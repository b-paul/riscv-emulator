use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::Read;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;

mod csr;
pub mod device;
mod elf;
mod instructions;
mod interpret;
mod mem;
pub mod tester;
mod trap;

use csr::MachineCsrs;
use device::{Device, DeviceRegister};
use mem::Memory;
use trap::Trap;

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

pub struct Emulator {
    memory: Memory,

    x: [u64; 32],

    trap: Option<u64>,

    machine_csrs: MachineCsrs,

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
    pub fn new(mem_size: usize) -> Self {
        Emulator {
            memory: Memory::new(mem_size),

            x: [0; 32],

            trap: None,

            machine_csrs: MachineCsrs::default(),

            waiting: false,

            privilege: Privilege::Machine,

            pc: 0,

            reservation: AtomicUsize::new(0),

            devices: Vec::new(),
            device_map: BTreeMap::new(),
        }
    }

    pub fn load_binary(&mut self, file_name: &str, entry_point: u64) -> std::io::Result<()> {
        let mut file = std::fs::File::open(file_name)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let elf = elf::Elf::read_elf(&buf);
        println!("Elf: {elf:?}");
        self.memory.write_bytes(0, &buf).unwrap();
        self.pc = entry_point;
        Ok(())
    }

    pub fn add_device(&mut self, device: Rc<RefCell<dyn Device>>) {
        let idx = self.devices.len();
        for register in device.borrow().get_registers() {
            self.device_map.insert(register.addr, (idx, register));
        }

        self.devices.push(device);
    }

    fn handle_traps(&mut self, pc: u64) {
        if let Some(trap) = self.trap {
            self.machine_csrs.mepc = pc;
            self.machine_csrs.mcause = trap;
            self.pc = self.machine_csrs.mtvec;
            self.trap = None;
            // set MPP to the current privilege level;
            self.machine_csrs.mstatus =
                (self.machine_csrs.mstatus & !(3 << 11)) | u64::from(self.privilege) << 11;
            // set MPIE to MIE
            self.machine_csrs.mstatus =
                (self.machine_csrs.mstatus & !(0x80)) | (self.machine_csrs.mstatus & 0x8) << 4;
            // Set MIE to 0
            self.machine_csrs.mstatus &= !0x8;
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
            Trap::InstrAddrMisaligned => self.machine_csrs.mtval = 0,
            Trap::InstrAccessFault => self.machine_csrs.mtval = 0,
            Trap::IllegalInstruction => self.machine_csrs.mtval = opcode,
            Trap::Breakpoint => self.machine_csrs.mtval = 0,
            Trap::LoadAccessFault => self.machine_csrs.mtval = 0,
            Trap::StoreAccessFault => self.machine_csrs.mtval = 0,
            Trap::ECallU => self.machine_csrs.mtval = 0,
            Trap::ECallM => self.machine_csrs.mtval = 0,
        }
    }

    fn increment_counters(&mut self) {
        if self.machine_csrs.mcountinhibit & 1 == 0 {
            self.machine_csrs.mcycle += 1;
        }
        if self.machine_csrs.mcountinhibit & 4 == 0 {
            self.machine_csrs.minstret += 1;
        }
    }

    pub fn cycle(&mut self) {
        let mut offset = 0;
        if let Ok(opcode) = self.read_u16(self.pc as usize) {
            if opcode & 0b11 == 0b11 {
                let Ok(opcode) = self.read_u32(self.pc as usize) else {
                    self.set_trap(Trap::InstrAccessFault, 0);
                    self.machine_csrs.mtval = 0;
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
