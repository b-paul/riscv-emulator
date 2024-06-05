use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::Read;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;

mod csr;
mod device;
mod instructions;
mod interpret;
mod mem;
mod tester;

use device::{Device, DeviceRegister};
use mem::Memory;

use instructions::Instruction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Privilege {
    User = 0b00,
    Machine = 0b11,
}

impl TryFrom<u64> for Privilege {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0b00 => Ok(Privilege::User),
            0b11 => Ok(Privilege::Machine),
            _ => Err(()),
        }
    }
}

impl From<Privilege> for u64 {
    fn from(value: Privilege) -> Self {
        match value {
            Privilege::User => 0b00,
            Privilege::Machine => 0b11,
        }
    }
}

// TODO
// enums for CSRs ?!

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum Trap {
    InstrAddrMisaligned,
    InstrAccessFault,
    IllegalInstruction,
    Breakpoint,
    LoadAccessFault,
    StoreAccessFault,
    ECallU,
    ECallM,
}

impl Trap {
    fn to_code(self) -> u64 {
        match self {
            Trap::InstrAddrMisaligned => 0,
            Trap::InstrAccessFault => 1,
            Trap::IllegalInstruction => 2,
            Trap::Breakpoint => 3,
            Trap::LoadAccessFault => 5,
            Trap::StoreAccessFault => 7,
            Trap::ECallU => 8,
            Trap::ECallM => 11,
        }
    }
}

struct Emulator {
    memory: Memory,

    x: [u64; 32],

    trap: Option<u64>,

    misa: u64,
    mstatus: u64,
    mtvec: u64,
    mip: u64,
    mie: u64,
    mcycle: u64,
    minstret: u64,
    mcounteren: u32,
    mcountinhibit: u32,
    mscratch: u64,
    mepc: u64,
    mcause: u64,
    mtval: u64,
    menvcfg: u64,
    mseccfg: u64,
    mtime: u64,
    mtimecmp: u64,

    waiting: bool,

    privilege: Privilege,

    pc: u64,

    // A valid reservation will always have the bottom 2 bits set to 0, since it must be aligned to
    // a 4 byte boundary. This means we can encode information in these bottom bits!
    // 00 : No reservation
    // 01 : Word reservation
    // 10 : Double word reservation
    // 11 : unused
    reservation: AtomicUsize,

    devices: Vec<Rc<RefCell<dyn Device>>>,
    device_map: BTreeMap<usize, (usize, DeviceRegister)>,
}

impl Emulator {
    fn new(mem_size: usize) -> Self {
        Emulator {
            memory: Memory::new(mem_size),

            x: [0; 32],

            trap: None,

            misa: 2 << 62 | 0b00000100000001000100000101,
            mstatus: 0,
            mtvec: 0,
            mip: 0,
            mie: 0,
            mcycle: 0,
            minstret: 0,
            mcounteren: 0,
            mcountinhibit: 0,
            mscratch: 0,
            mepc: 0,
            mcause: 0,
            mtval: 0,
            menvcfg: 0,
            mseccfg: 0,
            mtime: 0,
            mtimecmp: 0,
            waiting: false,

            privilege: Privilege::Machine,

            pc: 0,

            reservation: AtomicUsize::new(0),

            devices: Vec::new(),
            device_map: BTreeMap::new(),
        }
    }

    fn load_binary(&mut self, file_name: &str, entry_point: u64) -> std::io::Result<()> {
        let mut file = std::fs::File::open(file_name)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        self.memory.write_bytes(0, &buf).unwrap();
        self.pc = entry_point;
        Ok(())
    }

    fn add_device(&mut self, device: Rc<RefCell<dyn Device>>) {
        let idx = self.devices.len();
        for register in device.borrow().get_registers() {
            self.device_map.insert(register.addr, (idx, register));
        }

        self.devices.push(device);
    }

    fn handle_traps(&mut self, pc: u64) {
        if let Some(trap) = self.trap {
            self.mepc = pc;
            self.mcause = trap;
            self.pc = self.mtvec;
            self.trap = None;
            // set MPP to the current privilege level;
            self.mstatus = (self.mstatus & !(3 << 11)) | u64::from(self.privilege) << 11;
            // set MPIE to MIE
            self.mstatus = (self.mstatus & !(0x80)) | (self.mstatus & 0x8) << 4;
            // Set MIE to 0
            self.mstatus &= !0x8;
            // Traps by default are handled by M mode, but when S mode is implemented this must be
            // changed.
            self.privilege = Privilege::Machine;
        }
    }

    fn set_mtrap(&mut self, trap: Trap) {
        let cause = trap.to_code();
        self.trap = Some(cause);
    }

    fn set_trap(&mut self, trap: Trap, opcode: u64) {
        self.set_mtrap(trap);
        match trap {
            Trap::InstrAddrMisaligned => self.mtval = 0,
            Trap::InstrAccessFault => self.mtval = 0,
            Trap::IllegalInstruction => self.mtval = opcode,
            Trap::Breakpoint => self.mtval = 0,
            Trap::LoadAccessFault => self.mtval = 0,
            Trap::StoreAccessFault => self.mtval = 0,
            Trap::ECallU => self.mtval = 0,
            Trap::ECallM => self.mtval = 0,
        }
    }

    fn increment_counters(&mut self) {
        if self.mcountinhibit & 1 == 0 {
            self.mcycle += 1;
        }
        if self.mcountinhibit & 4 == 0 {
            self.minstret += 1;
        }
    }

    fn cycle(&mut self) {
        let mut offset = 0;
        if let Ok(opcode) = self.read_u16(self.pc as usize) {
            if opcode & 0b11 == 0b11 {
                let Ok(opcode) = self.read_u32(self.pc as usize) else {
                    self.set_trap(Trap::InstrAccessFault, 0);
                    self.mtval = 0;
                    return;
                };

                match Instruction::parse(opcode) {
                    Some(instruction) => self.execute(instruction, opcode as u64),
                    None => self.set_trap(Trap::IllegalInstruction, opcode as u64),
                }
                offset = 4;
            } else {
                match Instruction::parse_compressed(opcode) {
                    Some(instruction) => self.execute(instruction, opcode as u64),
                    None => self.set_trap(Trap::IllegalInstruction, opcode as u64),
                }

                offset = 2;
            }
            self.increment_counters();
        } else {
            self.set_trap(Trap::InstrAccessFault, 0);
        };
        let pc = self.pc;
        self.pc = self.pc.wrapping_add(offset);
        self.handle_traps(pc);
    }
}

fn main() {
    let path = std::env::args().nth(1).unwrap();

    for entry in std::fs::read_dir(path).unwrap().flatten() {
        let name = entry.file_name();
        let name = name.to_str().unwrap();

        if !name.starts_with("rv64mi-p-") || name.ends_with(".dump") {
            continue;
        }

        let path = entry.path();
        let path = path.to_str().unwrap();

        let mut emu = Emulator::new(128 * 1024 * 1024);

        let tester_addr = match name {
            "rv64ui-p-ma_data" => 0x3000,
            "rv64uc-p-rvc" => 0x4000,
            _ => 0x2000,
        };

        let tester = Rc::new(RefCell::new(tester::Tester::new(tester_addr)));

        emu.load_binary(path, 0x1000).unwrap();

        emu.add_device(tester.clone() as Rc<RefCell<dyn Device>>);

        loop {
            emu.cycle();
            if let Some(code) = tester.borrow().get_exit_code() {
                println!("{name}: {code}");
                break;
            }
        }
    }
}
