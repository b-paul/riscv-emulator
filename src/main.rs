use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Memory {
    bytes: Box<[u8]>,
    size: usize,
}

impl Memory {
    fn new(size: usize) -> Memory {
        Memory {
            bytes: vec![0; size].into_boxed_slice(),
            size,
        }
    }

    // slightly confusing api with this being arrays and write_bytes being slices but whatever
    fn read_bytes<const COUNT: usize>(&self, mut addr: usize) -> [u8; COUNT] {
        let mut out = [0; COUNT];
        for b in out.iter_mut() {
            *b = self.bytes[addr];
            addr += 1;
            addr %= self.size;
        }
        out
    }

    fn write_bytes(&mut self, mut addr: usize, bytes: &[u8]) {
        for &b in bytes {
            self.bytes[addr] = b;
            addr += 1;
            addr %= self.size;
        }
    }
}

struct Emulator {
    memory: Memory,

    x: [u64; 32],

    trap: Option<u64>,

    f: [f64; 32],
    fcsr: u32,

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
    _mtime: u64,
    _mtimecmp: u64,

    pc: u64,

    // A valid reservation will always have the bottom 2 bits set to 0, since they must be aligned
    // to a 4 byte boundary. This means we can encode information in these bottom bits!
    // 00 : No reservation
    // 01 : Word reservation
    // 10 : Double word reservation
    // 11 : unused
    reservation: AtomicUsize,
}

impl Emulator {
    fn new(mem_size: usize) -> Self {
        Emulator {
            memory: Memory::new(mem_size),

            x: [0; 32],

            trap: None,

            f: [0.; 32],
            fcsr: 0,

            misa: 2 << 62 | 0b00000000000001000100101101,
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
            _mtime: 0,
            _mtimecmp: 0,

            pc: 0,

            reservation: AtomicUsize::new(0),
        }
    }

    fn load_binary(&mut self, file_name: &str, entry_point: u64) -> std::io::Result<()> {
        let mut file = std::fs::File::open(file_name)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        self.memory.write_bytes(0, &buf);
        self.pc = entry_point;
        Ok(())
    }

    fn get_csr(&mut self, csr: u32, _read: bool) -> Option<u64> {
        match csr {
            // Machine csrs
            0x301 => Some(self.misa),                 // misa
            0xF11 => Some(0),                         // mvendorid
            0xF12 => Some(0),                         // marchid
            0xF13 => Some(0),                         // mimpid
            0xF14 => Some(0),                         // mhartid
            0x300 => Some(self.mstatus),              // mstatus
            0x305 => Some(self.mtvec),                // mtvec
            0x344 => Some(0),                         // mip
            0x304 => Some(0),                         // mie
            0xB00 => Some(self.mcycle),               // mcycle
            0xB02 => Some(self.minstret),             // minstret
            0xB04..=0xB1F => Some(0),                 // mhpmcounterN (unimplemented)
            0x323..=0x33F => Some(0),                 // mhpmeventN (unimplemented)
            0x306 => Some(self.mcounteren as u64),    // mcounteren (check when implementing U or S)
            0x320 => Some(self.mcountinhibit as u64), // mcountinhibit
            0x340 => Some(self.mscratch),             // mscratch
            0x341 => Some(self.mepc),                 // mepc
            0x342 => Some(self.mcause),               // mcause
            0x343 => Some(self.mtval),                // mtval
            0xF15 => Some(0),                         // mconfigptr
            0x30A => Some(self.menvcfg),              // menvcfg
            0x747 => Some(self.mseccfg),              // mseccfg

            0x003 => Some(self.fcsr as u64),
            _ => None,
        }
    }

    fn set_csr(&mut self, csr: u32, val: u64, write: bool) {
        if !write {
            return;
        }
        match csr {
            // Machine csrs

            // misa
            // Don't allow modification of allowed extensions for simplicity
            // (might change later)
            0x301 => {}
            // mstatus (0x7fffffc0ff800015 is the WPRI mask)
            0x300 => {
                self.mstatus = val & !0x7fffffc0ff800015;
                // when privledge level x is not implemented xPP is read only 0.
                self.mstatus &= !0x100;
                // when privledge level x is not implemented xXL is read only 0.
                self.mstatus &= !(3 << 30);
                self.mstatus &= !(3 << 32);
                // MPRIV is read only 0 if U is not implemented
                self.mstatus &= !(1 << 17);
                // MXR is read only 0 if S is not implemented
                self.mstatus &= !(1 << 19);
                // SUM is read only 0 if S is not implemented
                self.mstatus &= !(1 << 18);
                // We only support little endian, so MBE, SBE and UBE are effectively read only 0.
                self.mstatus &= !(1 << 37);
                self.mstatus &= !(1 << 36);
                self.mstatus &= !(1 << 6);
                // TVM is read only 0 if S is not implemented
                self.mstatus &= !(1 << 20);
                // TW is read only 0 if there are no modes less than M implemented
                self.mstatus &= !(1 << 21);
                // TSR is read only 0 if S is not implemented
                self.mstatus &= !(1 << 22);
                // For simplicity FS will always say dirty
                self.mstatus |= 3 << 13;
                // VS and XS are read only zero as neither V nor X are implemented
                self.mstatus &= !(3 << 9);
                self.mstatus &= !(3 << 15);
            }
            0x305 => self.mtvec = val & !3, // mtvec (we assume always direct)
            // mip
            0x344 => {
                // For us everything in the bottom 16 bites of mip is read only
                self.mip = val & !0xffff;
            }
            // mie
            0x304 => {
                // Zero out the zero bits of mie.
                self.mie = val & !0xd555;
                // SEIP, STIP and SSIP are read only zero since S is not implemented
                self.mie &= !(1 << 1);
                self.mie &= !(1 << 5);
                self.mie &= !(1 << 9);
                // LCOFIE is read only zero since Sscofpmf is not implemented
                self.mie &= !(1 << 13);
            }
            0xB00 => self.mcycle = val,            // mcycle
            0xB02 => self.minstret = val,          // minstret
            0x306 => self.mcounteren = val as u32, // mcounteren
            0x340 => self.mscratch = val,          // mscratch
            0x341 => self.mepc = val,              // mepc
            0x342 => self.mcause = val,            // mcause
            0x343 => self.mtval = val,             // mtval
            0x30A => self.menvcfg = val,           // menvcfg TODO
            0x747 => self.mseccfg = val,           // mseccfg TODO?

            0x003 => self.fcsr = val as u32,
            _ => self.illegal_instruction(),
        }
    }

    fn handle_traps(&mut self) {
        if let Some(trap) = self.trap {
            self.mepc = self.pc;
            self.mcause = trap;
            self.pc = self.mtvec.wrapping_sub(4);
            self.trap = None;

            match trap {
                11 => {
                    println!("Stage {}", self.x[10] / 2);
                }
                _ => (),
            }
        }
    }

    fn set_mtrap(&mut self, cause: u64) {
        self.trap = Some(cause);
    }

    fn illegal_instruction(&mut self) {
        self.set_mtrap(2);
        let mut instruction = u32::from_le_bytes(self.memory.read_bytes(self.pc as usize));
        if instruction & 3 != 3 {
            instruction &= 0xffff;
        }
        self.mtval = instruction as u64;
    }

    fn increment_counters(&mut self) {
        if self.mcountinhibit & 1 == 0 {
            self.mcycle += 1;
        }
        if self.mcountinhibit & 4 == 0 {
            self.minstret += 1;
        }
    }

    fn try_16bit_instruction(&mut self) -> bool {
        let instruction = u16::from_le_bytes(self.memory.read_bytes(self.pc as usize));
        let opcode = instruction & 0x3;
        let funct3 = (instruction >> 13) & 0x7;

        if opcode == 0b11 {
            return false;
        }

        if instruction == 0 {
            self.illegal_instruction();
        }

        match opcode {
            0b00 => {
                let rd = (instruction >> 2 & 0x7) as usize + 8;
                let rs1 = (instruction >> 7 & 0x7) as usize + 8;
                let rs2 = (instruction >> 2 & 0x7) as usize + 8;
                match funct3 {
                    // C.LW
                    0b010 => {
                        let offset = instruction >> 4 & 0x4
                            | instruction >> 7 & 0x38
                            | instruction << 1 & 0x40;
                        let addr = self.x[rs1] as usize + offset as usize;
                        self.x[rd] = u32::from_le_bytes(self.memory.read_bytes(addr)) as u64;
                    }
                    // C.LD
                    0b011 => {
                        let offset = instruction >> 7 & 0x38 | instruction << 1 & 0xc0;
                        let addr = self.x[rs1] as usize + offset as usize;
                        self.x[rd] = u64::from_le_bytes(self.memory.read_bytes(addr));
                    }
                    // C.SW
                    0b110 => {
                        let offset = instruction >> 4 & 0x4
                            | instruction >> 7 & 0x38
                            | instruction << 1 & 0x40;
                        let addr = self.x[rs1] as usize + offset as usize;
                        self.memory
                            .write_bytes(addr, &(self.x[rs2] as u32).to_le_bytes());
                    }
                    // C.SD
                    0b111 => {
                        let offset = instruction >> 7 & 0x38 | instruction << 1 & 0xc0;
                        let addr = self.x[rs1] as usize + offset as usize;
                        self.memory.write_bytes(addr, &(self.x[rs2]).to_le_bytes());
                    }
                    // C.ADDI4SPN
                    0b000 => {
                        let rd = (instruction >> 2 & 0x7) as usize + 8;
                        let nzuimm = (instruction >> 4 & 0x4
                            | instruction >> 2 & 0x8
                            | instruction >> 7 & 0x30
                            | instruction >> 1 & 0x3c) as u64;
                        self.x[2] = self.x[2].wrapping_add(nzuimm);
                        self.x[rd] = self.x[2];
                    }
                    _ => self.illegal_instruction(),
                }
            }

            0b01 => {
                match funct3 {
                    // C.J
                    0b101 => {
                        let offset = instruction >> 2 & 0x6
                            | instruction >> 7 & 0x10
                            | instruction << 3 & 0x20
                            | instruction >> 1 & 0x40
                            | instruction << 1 & 0x80
                            | instruction >> 1 & 0x300
                            | instruction << 2 & 0x400
                            | instruction >> 1 & 0x800;
                        self.pc = self.pc.wrapping_add(offset as u64).wrapping_sub(2);
                    }
                    // C.BEQZ
                    0b110 => {
                        let rs1 = (instruction >> 7 & 0x7) as usize + 8;
                        let offset = instruction >> 2 & 0x6
                            | instruction >> 7 & 0x18
                            | instruction << 3 & 0x20
                            | instruction << 1 & 0xc0
                            | instruction >> 4 & 0x100;
                        let offset = offset as i8 as i64 as u64;
                        if self.x[rs1] == 0 {
                            self.pc = self.pc.wrapping_add(offset).wrapping_sub(2);
                        }
                    }
                    // C.BNEZ
                    0b111 => {
                        let rs1 = (instruction >> 7 & 0x7) as usize + 8;
                        let offset = instruction >> 2 & 0x6
                            | instruction >> 7 & 0x18
                            | instruction << 3 & 0x20
                            | instruction << 1 & 0xc0
                            | instruction >> 4 & 0x100;
                        let offset = offset as i8 as i64 as u64;
                        if self.x[rs1] != 0 {
                            self.pc = self.pc.wrapping_add(offset).wrapping_sub(2);
                        }
                    }
                    // C.LI
                    0b010 => {
                        let rd = (instruction >> 7 & 0x1f) as usize;
                        let imm = (((instruction >> 2 & 0x1f | instruction >> 7 & 0x20) as i8) << 2
                            >> 2) as i64 as u64;
                        self.x[rd] = imm;
                    }
                    0b011 => {
                        let rd = (instruction >> 7 & 0x1f) as usize;
                        // C.ADDI16SP
                        if rd == 2 {
                            let nzimm = (((instruction >> 2 & 0x10
                                | instruction << 3 & 0x20
                                | instruction << 1 & 0x40
                                | instruction << 4 & 0x180
                                | instruction >> 3 & 0x200)
                                as i16)
                                << 6
                                >> 6) as i64 as u64;
                            if nzimm == 0 {
                                self.illegal_instruction();
                            }
                            self.x[2] = self.x[2].wrapping_add(nzimm);
                        } else {
                            // C.LUI
                            let imm = (((instruction as i32) << 10 & 0x1f000
                                | (instruction as i32) << 5 & 0x20000)
                                << 14
                                >> 14) as i64 as u64;
                            if imm == 0 {
                                self.illegal_instruction();
                            }
                            self.x[rd] = imm;
                        }
                    }
                    // C.ADDI
                    0b000 => {
                        let rd = (instruction >> 7 & 0x1f) as usize;
                        let nzimm = (((instruction >> 2 & 0x1f | instruction >> 7 & 0x20) as i8)
                            << 2
                            >> 2) as i64 as u64;
                        self.x[rd] = (self.x[rd]).wrapping_add(nzimm);
                    }
                    // C.ADDIW
                    0b001 => {
                        let rd = (instruction >> 7 & 0x1f) as usize;
                        let imm = (((instruction >> 2 & 0x1f | instruction >> 7 & 0x20) as i8) << 2
                            >> 2) as i32;
                        self.x[rd] = (self.x[rd] as i32).wrapping_add(imm) as i64 as u64;
                    }
                    0b100 => {
                        let rd = (instruction >> 7 & 0x7) as usize + 8;
                        let funct2 = instruction >> 10 & 0x3;
                        match funct2 {
                            // C.SRLI
                            0b00 => {
                                let shamt =
                                    (instruction >> 2 & 0x1f | instruction >> 7 & 0x20) as u32;
                                self.x[rd] = self.x[rd].wrapping_shr(shamt);
                            }
                            // C.SRAI
                            0b01 => {
                                let shamt =
                                    (instruction >> 2 & 0x1f | instruction >> 7 & 0x20) as u32;
                                self.x[rd] = (self.x[rd] as i64).wrapping_shr(shamt) as u64;
                            }
                            // C.ANDI
                            0b10 => {
                                let imm =
                                    (instruction >> 2 & 0x1f | instruction >> 7 & 0x20) as u64;
                                self.x[rd] &= imm;
                            }
                            0b11 => {
                                let rs2 = (instruction >> 2 & 0x7) as usize + 8;
                                let funct3 = instruction >> 5 & 0x3 | instruction >> 8 & 0x4;
                                match funct3 {
                                    // C.AND
                                    0b011 => self.x[rd] &= self.x[rs2],
                                    // C.OR
                                    0b010 => self.x[rd] |= self.x[rs2],
                                    // C.XOR
                                    0b001 => self.x[rd] &= self.x[rs2],
                                    // C.SUB
                                    0b000 => self.x[rd] = self.x[rd].wrapping_sub(self.x[rs2]),
                                    // C.ADDW
                                    0b101 => {
                                        self.x[rd] = self.x[rd].wrapping_add(self.x[rs2]) as i32
                                            as i64
                                            as u64
                                    }
                                    // C.SUBW
                                    0b100 => {
                                        self.x[rd] = self.x[rd].wrapping_sub(self.x[rs2]) as i32
                                            as i64
                                            as u64
                                    }
                                    _ => self.illegal_instruction(),
                                }
                            }
                            _ => self.illegal_instruction(),
                        }
                    }
                    _ => self.illegal_instruction(),
                }
            }

            0b10 => {
                match funct3 {
                    // C.LWSP
                    0b010 => {
                        let rd = (instruction >> 7 & 0x1f) as usize;
                        if rd == 0 {
                            self.illegal_instruction();
                        }
                        let offset = instruction >> 2 & 0x1c
                            | instruction >> 7 & 0x20
                            | instruction << 4 & 0xc0;
                        let addr = self.x[2] as usize + offset as usize;
                        self.x[rd] = u32::from_le_bytes(self.memory.read_bytes(addr)) as u64;
                    }
                    // C.LDSP
                    0b011 => {
                        let rd = (instruction >> 7 & 0x1f) as usize;
                        if rd == 0 {
                            self.illegal_instruction();
                        }
                        let offset = instruction >> 2 & 0x18
                            | instruction >> 7 & 0x20
                            | instruction << 4 & 0x1c0;
                        let addr = self.x[2] as usize + offset as usize;
                        self.x[rd] = u64::from_le_bytes(self.memory.read_bytes(addr));
                    }
                    // C.SWSP
                    0b110 => {
                        let rs2 = (instruction >> 2 & 0x1f) as usize;
                        let offset = instruction >> 7 & 0x7c | instruction >> 1 & 0x180;
                        let addr = self.x[2] as usize + offset as usize;
                        self.memory
                            .write_bytes(addr, &u32::to_le_bytes(self.x[rs2] as u32));
                    }
                    // C.SDSP
                    0b111 => {
                        let rs2 = (instruction >> 2 & 0x1f) as usize;
                        let offset = instruction >> 7 & 0x78 | instruction >> 1 & 0x380;
                        let addr = self.x[2] as usize + offset as usize;
                        self.memory
                            .write_bytes(addr, &u64::to_le_bytes(self.x[rs2]));
                    }
                    0b100 => {
                        let rs1 = (instruction >> 7 & 0x1f) as usize;
                        let rs2 = (instruction >> 2 & 0x1f) as usize;
                        let funct4 = (instruction >> 12 & 0x1) == 1;
                        if rs1 == 0 {
                            self.illegal_instruction();
                        }
                        match (rs1 == 0, rs2 == 0, funct4) {
                            // C.JR
                            (false, true, false) => {
                                self.pc = self.x[rs1].wrapping_sub(2);
                            }
                            // C.JALR
                            (false, true, true) => {
                                let tmp = self.x[rs1];
                                self.x[1] = self.pc;
                                self.pc = tmp.wrapping_sub(2);
                            }
                            // C.MV
                            (false, false, false) => {
                                self.x[rs1] = self.x[rs2];
                            }
                            (false, false, true) => {
                                self.x[rs1] = self.x[rs1].wrapping_add(self.x[rs2]);
                            }
                            // C.EBREAK
                            (true, true, true) => {
                                todo!("C.EBREAK")
                            }
                            _ => self.illegal_instruction(),
                        }
                    }
                    // C.SLLI
                    0b000 => {
                        let rd = (instruction >> 7 & 0x1f) as usize;
                        let shamt = (instruction >> 2 & 0x1f | instruction >> 7 & 0x20) as u32;
                        self.x[rd] = self.x[rd].wrapping_shl(shamt);
                    }
                    _ => self.illegal_instruction(),
                }
            }

            _ => self.illegal_instruction(),
        }

        self.pc = self.pc.wrapping_add(2);

        self.increment_counters();

        self.handle_traps();

        true
    }

    fn run_instruction(&mut self) {
        // the x0 register should always be 0 (hopefully it doesn't get written to and then used)
        self.x[0] = 0;

        if self.try_16bit_instruction() {
            return;
        }

        let instruction = u32::from_le_bytes(self.memory.read_bytes(self.pc as usize));
        let opcode = instruction & 0x7f;
        let rd = (instruction >> 7 & 0x1f) as usize;
        let rs1 = (instruction >> 15 & 0x1f) as usize;
        let rs2 = (instruction >> 20 & 0x1f) as usize;
        let funct3 = instruction >> 12 & 0x7;
        let funct7 = instruction >> 25 & 0x7f;

        match opcode {
            // Immediate instructions
            0b0010011 => {
                // note that we convert instruction to an i32 for sign extension.
                let uinp = self.x[rs1];
                let iinp = uinp as i64;
                let uimm = ((instruction as i32 >> 20) % 0x1000) as u64;
                let iimm = uimm as i64;
                match funct3 {
                    // ADDI
                    0b000 => self.x[rd] = (iinp.wrapping_add(iimm)) as u64,
                    // SLTI
                    0b010 => self.x[rd] = if iinp < iimm { 1 } else { 0 },
                    // SLTIU
                    0b011 => self.x[rd] = if uinp < uimm { 1 } else { 0 },
                    // XORI
                    0b100 => self.x[rd] = uimm ^ uinp,
                    // ORI
                    0b110 => self.x[rd] = uimm | uinp,
                    // ANDI
                    0b111 => self.x[rd] = uimm & uinp,
                    0b001 => {
                        let upper = instruction >> 26 & 0x3f;
                        let shamt = instruction >> 20 & 0x3f;
                        match upper {
                            // SLLI
                            0b000000 => self.x[rd] = uinp.wrapping_shl(shamt),
                            _ => self.illegal_instruction(),
                        }
                    }
                    0b101 => {
                        let upper = instruction >> 26 & 0x3f;
                        let shamt = instruction >> 20 & 0x3f;
                        match upper {
                            // SRLI
                            0b0000000 => self.x[rd] = uinp.wrapping_shr(shamt),
                            // SRAI
                            0b010000 => self.x[rd] = (iinp.wrapping_shr(shamt)) as u64,
                            _ => self.illegal_instruction(),
                        }
                    }
                    _ => unreachable!(),
                }
            }

            0b0011011 => {
                let uinp = self.x[rs1] as u32;
                let iinp = uinp as i32;
                match funct3 {
                    // ADDIW
                    0b000 => {
                        let imm = instruction as i32 >> 20;
                        self.x[rd] = (imm.wrapping_add(iinp)) as i64 as u64;
                    }
                    0b001 => {
                        let upper = instruction >> 25 & 0x7f;
                        match upper {
                            // SLLIW
                            0b0000000 => {
                                let shamt = instruction >> 20 & 0x1f;
                                self.x[rd] = (uinp << shamt) as i32 as i64 as u64;
                            }
                            _ => self.illegal_instruction(),
                        }
                    }
                    0b101 => {
                        let upper = instruction >> 25 & 0x7f;
                        match upper {
                            // SRLIW
                            0b0000000 => {
                                let shamt = instruction >> 20 & 0x1f;
                                self.x[rd] = (uinp >> shamt) as i32 as i64 as u64;
                            }
                            // SRAIW
                            0b0100000 => {
                                let shamt = instruction >> 20 & 0x1f;
                                self.x[rd] = (iinp >> shamt) as i64 as u64;
                            }
                            _ => self.illegal_instruction(),
                        }
                    }
                    _ => self.illegal_instruction(),
                }
            }

            // LUI
            0b0110111 => {
                let imm = instruction & 0xfffff << 12;
                self.x[rd] = imm as i32 as i64 as u64;
            }

            // AUIPC
            0b0010111 => {
                let imm = instruction & 0xfffff << 12;
                self.x[rd] = (imm as i32 as i64 as u64).wrapping_add(self.pc);
            }

            // Register instructions
            0b0110011 => {
                let a = self.x[rs1];
                let b = self.x[rs2];
                match (funct3, funct7) {
                    // ADD
                    (0b000, 0b0000000) => self.x[rd] = a.wrapping_add(b),
                    // SUB
                    (0b000, 0b0100000) => self.x[rd] = a.wrapping_sub(b),
                    // SLT
                    (0b010, 0b0000000) => self.x[rd] = if (a as i64) < (b as i64) { 1 } else { 0 },
                    // SLTU
                    (0b011, 0b0000000) => self.x[rd] = if a < b { 1 } else { 0 },
                    // XOR
                    (0b100, 0b0000000) => self.x[rd] = a ^ b,
                    // OR
                    (0b110, 0b0000000) => self.x[rd] = a | b,
                    // AND
                    (0b111, 0b0000000) => self.x[rd] = a & b,
                    // SLL
                    (0b001, 0b0000000) => self.x[rd] = a << (b & 0x3f),
                    // SRL
                    (0b101, 0b0000000) => self.x[rd] = a >> (b & 0x3f),
                    // SRA
                    (0b101, 0b0100000) => self.x[rd] = (a as i64 >> (b & 0x3f)) as u64,
                    // MUL
                    (0b000, 0b0000001) => self.x[rd] = a.wrapping_mul(b),
                    // MULH
                    (0b001, 0b0000001) => {
                        self.x[rd] =
                            ((a as i64 as i128).wrapping_mul(b as i64 as i128) >> 64) as u64
                    }
                    // MULHU
                    (0b011, 0b0000001) => {
                        self.x[rd] = ((a as u128).wrapping_mul(b as u128) >> 64) as u64
                    }
                    // MULHSU
                    (0b010, 0b0000001) => {
                        self.x[rd] =
                            ((a as i64 as i128).wrapping_mul(b as u128 as i128) >> 64) as u64
                    }
                    // DIV
                    (0b100, 0b0000001) => {
                        self.x[rd] = if a as i64 == i64::MIN && b as i64 == -1 {
                            a
                        } else if b == 0 {
                            u64::MAX
                        } else {
                            (a as i64).wrapping_div(b as i64) as u64
                        }
                    }
                    // DIVU
                    (0b101, 0b0000001) => {
                        self.x[rd] = if b == 0 {
                            u64::MAX
                        } else {
                            (a).wrapping_div(b)
                        }
                    }
                    // REM
                    (0b110, 0b0000001) => {
                        self.x[rd] = if a as i64 == i64::MIN && b as i64 == -1 {
                            0
                        } else if b == 0 {
                            a
                        } else {
                            (a as i64).wrapping_rem(b as i64) as u64
                        }
                    }
                    // REMU
                    (0b111, 0b0000001) => self.x[rd] = if b == 0 { a } else { a.wrapping_rem(b) },
                    _ => self.illegal_instruction(),
                }
            }

            0b0111011 => {
                let a = self.x[rs1] as u32;
                let b = self.x[rs2] as u32;
                match (funct3, funct7) {
                    // ADDW
                    (0b000, 0b0000000) => {
                        self.x[rd] = (a as i32).wrapping_add(b as i32) as i64 as u64
                    }
                    // SUBW
                    (0b000, 0b0100000) => {
                        self.x[rd] = (a as i32).wrapping_sub(b as i32) as i64 as u64
                    }
                    // SLLW
                    (0b001, 0b0000000) => self.x[rd] = (a << (b & 0x1f)) as i32 as i64 as u64,
                    // SRLW
                    (0b101, 0b0000000) => self.x[rd] = (a >> (b & 0x1f)) as i32 as i64 as u64,
                    // SRAW
                    (0b101, 0b0100000) => self.x[rd] = (a as i32 >> (b & 0x1f)) as i64 as u64,
                    // MULW
                    (0b000, 0b0000001) => {
                        self.x[rd] = (a as i32).wrapping_mul(b as i32) as i64 as u64
                    }
                    // DIVW
                    (0b100, 0b0000001) => {
                        self.x[rd] = if a as i32 == i32::MIN && b as i32 == -1 {
                            i32::MIN as i64 as u64
                        } else if b == 0 {
                            u64::MAX
                        } else {
                            (a as i32).wrapping_div(b as i32) as i64 as u64
                        }
                    }
                    // DIVUW
                    (0b101, 0b0000001) => {
                        self.x[rd] = if b == 0 {
                            u64::MAX
                        } else {
                            a.wrapping_div(b as u32) as i32 as i64 as u64
                        }
                    }
                    // REMW
                    (0b110, 0b0000001) => {
                        self.x[rd] = if a as i32 == i32::MIN && b as i32 == -1 {
                            0
                        } else if b == 0 {
                            a as i32 as i64 as u64
                        } else {
                            (a as i32).wrapping_rem(b as i32) as i64 as u64
                        }
                    }
                    // REMUW
                    (0b111, 0b0000001) => {
                        self.x[rd] = if b == 0 {
                            a as i32 as i64 as u64
                        } else {
                            a.wrapping_rem(b) as i32 as i64 as u64
                        }
                    }
                    _ => self.illegal_instruction(),
                }
            }

            // Control transfer instructions

            // JAL
            0b1101111 => {
                let offset = instruction >> 21 & 0x3ff
                    | instruction >> 10 & 0x400
                    | instruction >> 1 & 0x7f800
                    | instruction >> 12 & 0x80000;
                let offset = ((offset as i32) << 12 >> 11) as i64;
                self.x[rd] = self.pc.wrapping_add(4);
                self.pc = self.pc.wrapping_add(offset as u64).wrapping_sub(4);
            }
            0b1100111 => {
                // JALR
                if funct3 == 0b000 {
                    let offset = instruction >> 20 & 0xfff;
                    let offset = ((offset as i32) << 20 >> 20) as i64 as u64;
                    let tmp = self.x[rs1];
                    self.x[rd] = self.pc.wrapping_add(4);
                    self.pc = tmp.wrapping_add(offset).wrapping_sub(4);
                } else {
                    self.illegal_instruction();
                }
            }
            0b1100011 => {
                let offset = instruction >> 8 & 0xf
                    | instruction >> 21 & 0x3f0
                    | instruction << 3 & 0x400
                    | instruction >> 20 & 0x800;
                let offset = ((offset as i32) << 20 >> 19) as u64;
                let offset = offset;
                match funct3 {
                    // BEQ
                    0b000 => {
                        if self.x[rs1] == self.x[rs2] {
                            self.pc = self.pc.wrapping_add(offset).wrapping_sub(4);
                        }
                    }
                    // BNE
                    0b001 => {
                        if self.x[rs1] != self.x[rs2] {
                            self.pc = self.pc.wrapping_add(offset).wrapping_sub(4);
                        }
                    }
                    // BLT
                    0b100 => {
                        if (self.x[rs1] as i64) < (self.x[rs2] as i64) {
                            self.pc = self.pc.wrapping_add(offset).wrapping_sub(4);
                        }
                    }
                    // BLTU
                    0b110 => {
                        if self.x[rs1] < self.x[rs2] {
                            self.pc = self.pc.wrapping_add(offset).wrapping_sub(4);
                        }
                    }
                    // BGE
                    0b101 => {
                        if (self.x[rs1] as i64) >= (self.x[rs2] as i64) {
                            self.pc = self.pc.wrapping_add(offset).wrapping_sub(4);
                        }
                    }
                    // BGTU
                    0b111 => {
                        if self.x[rs1] >= self.x[rs2] {
                            self.pc = self.pc.wrapping_add(offset).wrapping_sub(4);
                        }
                    }
                    _ => self.illegal_instruction(),
                }
            }

            // Load and Store
            0b0000011 => {
                let imm = (instruction as i32 >> 20) as u64;
                let addr = self.x[rs1].wrapping_add(imm) as usize;
                match funct3 {
                    // LB
                    0b000 => {
                        self.x[rd] = i8::from_le_bytes(self.memory.read_bytes(addr)) as i64 as u64
                    }
                    // LH
                    0b001 => {
                        self.x[rd] = i16::from_le_bytes(self.memory.read_bytes(addr)) as i64 as u64
                    }
                    // LW
                    0b010 => {
                        self.x[rd] = i32::from_le_bytes(self.memory.read_bytes(addr)) as i64 as u64
                    }
                    // LBU
                    0b100 => self.x[rd] = u8::from_le_bytes(self.memory.read_bytes(addr)) as u64,
                    // LHU
                    0b101 => self.x[rd] = u16::from_le_bytes(self.memory.read_bytes(addr)) as u64,
                    // LWU
                    0b110 => self.x[rd] = u32::from_le_bytes(self.memory.read_bytes(addr)) as u64,
                    // LD
                    0b011 => self.x[rd] = u64::from_le_bytes(self.memory.read_bytes(addr)),
                    _ => self.illegal_instruction(),
                }
            }
            0b0100011 => {
                let imm = instruction >> 7 & 0x1f | instruction >> 20 & 0xfe0;
                let imm = ((imm as i32) << 20 >> 20) as i64 as u64;
                let addr = self.x[rs1].wrapping_add(imm) as usize;
                match funct3 {
                    // SB
                    0b000 => self
                        .memory
                        .write_bytes(addr, &(self.x[rs2] as u8).to_le_bytes()),
                    // SH
                    0b001 => self
                        .memory
                        .write_bytes(addr, &(self.x[rs2] as u16).to_le_bytes()),
                    // SW
                    0b010 => self
                        .memory
                        .write_bytes(addr, &(self.x[rs2] as u32).to_le_bytes()),
                    // SD
                    0b011 => self.memory.write_bytes(addr, &self.x[rs2].to_le_bytes()),
                    _ => self.illegal_instruction(),
                }
            }

            // Memory ordering instructions

            // FENCE
            0b0001111 => {
                // i dont think anything needs to be done here until some sort of instruction
                // reordering is implemented
            }

            // System
            0b1110011 => {
                if funct3 == 0 {
                    match instruction {
                        // ECALL
                        0b00000000000000000000000001110011 => {
                            self.set_mtrap(11); // Environment call from M mode
                            self.minstret = self.minstret.wrapping_sub(1);
                        }
                        // EBREAK
                        0b00000000000100000000000001110011 => {
                            self.set_mtrap(3); // Breakpoint
                            self.minstret = self.minstret.wrapping_sub(1);
                        }
                        // MRET
                        0b00110000001000000000000001110011 => {
                            self.pc = self.mepc.wrapping_sub(4);
                        }
                        _ => self.illegal_instruction(),
                    }
                } else {
                    // We must be doing a Zicsr instruction
                    let csr = instruction >> 20 & 0xfff;
                    let uimm = rs1 as u64;
                    let read = !(funct3 & 3 == 0b01 && rs1 == 0);
                    let write = !(funct3 & 2 == 0b10 && rs1 == 0);
                    if let Some(csr_val) = self.get_csr(csr, read) {
                        if let Some(new_val) = match funct3 {
                            // CSRRW
                            0b001 => Some(self.x[rs1]),
                            // CSRRS
                            0b010 => Some(csr_val | self.x[rs1]),
                            // CSRRC
                            0b011 => Some(csr_val & !self.x[rs1]),
                            // CSRRWI
                            0b101 => Some(uimm),
                            // CSRRSI
                            0b110 => Some(csr_val | uimm),
                            // CSRRCI
                            0b111 => Some(csr_val & !uimm),
                            _ => None,
                        } {
                            self.set_csr(csr, new_val, write);
                            self.x[rd] = csr_val;
                        } else {
                            self.illegal_instruction();
                        }
                    } else {
                        self.illegal_instruction();
                    }
                }
            }

            0b0101111 => {
                let funct5 = instruction >> 27 & 0x1f;
                let _aq = instruction >> 26 & 1 != 0;
                let _rl = instruction >> 25 & 1 != 0;
                let addr = self.x[rs1] as usize;
                if funct3 == 0b010 && addr % 4 != 0 || funct3 == 0b011 && addr % 8 != 0 {
                    panic!("Misaligned memory addres {addr:x}");
                }
                match (funct5, funct3) {
                    // LR.W
                    (0b00010, 0b010) => {
                        if rs2 != 0 {
                            self.illegal_instruction();
                        }
                        self.x[rd] = i32::from_le_bytes(self.memory.read_bytes(addr)) as i64 as u64;

                        self.reservation.store(addr | 0b01, Ordering::Relaxed);
                    }
                    // LR.D
                    (0b00010, 0b011) => {
                        if rs2 != 0 {
                            self.illegal_instruction();
                        }
                        self.x[rd] = u64::from_le_bytes(self.memory.read_bytes(addr));

                        self.reservation.store(addr | 0b10, Ordering::Relaxed);
                    }
                    // SC.W
                    (0b00011, 0b010) => {
                        if self.reservation.load(Ordering::Acquire) == addr | 0b01 {
                            let bytes = (self.x[rs2] as u32).to_le_bytes();
                            self.memory.write_bytes(addr, &bytes);
                            self.reservation.store(0, Ordering::Release);
                            self.x[rd] = 0;
                        } else {
                            self.x[rd] = 1;
                        }
                    }
                    // SC.D
                    (0b00011, 0b011) => {
                        if self.reservation.load(Ordering::Acquire) == addr | 0b10 {
                            let bytes = self.x[rs2].to_le_bytes();
                            self.memory.write_bytes(addr, &bytes);
                            self.reservation.store(0, Ordering::Release);
                            self.x[rd] = 0;
                        } else {
                            self.x[rd] = 1;
                        }
                    }
                    // AMOSWAP.W
                    (0b00001, 0b010) => {
                        let inp = i32::from_le_bytes(self.memory.read_bytes(addr)) as i64 as u64;
                        let out = (self.x[rs2] as u32).to_le_bytes();
                        self.x[rd] = inp;
                        self.memory.write_bytes(addr, &out);
                    }
                    // AMOSWAP.D
                    (0b00001, 0b011) => {
                        let inp = u64::from_le_bytes(self.memory.read_bytes(addr));
                        let out = self.x[rs2].to_le_bytes();
                        self.x[rd] = inp;
                        self.memory.write_bytes(addr, &out);
                    }
                    // AMOADD.W
                    (0b00000, 0b010) => {
                        let inp = i32::from_le_bytes(self.memory.read_bytes(addr)) as i64 as u64;
                        let out = (inp.wrapping_add(self.x[rs2]) as u32).to_le_bytes();
                        self.x[rd] = inp;
                        self.memory.write_bytes(addr, &out);
                    }
                    // AMOADD.D
                    (0b00000, 0b011) => {
                        let inp = u64::from_le_bytes(self.memory.read_bytes(addr));
                        let out = inp.wrapping_add(self.x[rs2]).to_le_bytes();
                        self.x[rd] = inp;
                        self.memory.write_bytes(addr, &out);
                    }
                    // AMOAND.W
                    (0b01100, 0b010) => {
                        let inp = i32::from_le_bytes(self.memory.read_bytes(addr)) as i64 as u64;
                        let out = ((inp & self.x[rs2]) as u32).to_le_bytes();
                        self.x[rd] = inp;
                        self.memory.write_bytes(addr, &out);
                    }
                    // AMOAND.D
                    (0b01100, 0b011) => {
                        let inp = u64::from_le_bytes(self.memory.read_bytes(addr));
                        let out = (inp & self.x[rs2]).to_le_bytes();
                        self.x[rd] = inp;
                        self.memory.write_bytes(addr, &out);
                    }
                    // AMOOR.W
                    (0b01000, 0b010) => {
                        let inp = i32::from_le_bytes(self.memory.read_bytes(addr)) as i64 as u64;
                        let out = ((inp | self.x[rs2]) as u32).to_le_bytes();
                        self.x[rd] = inp;
                        self.memory.write_bytes(addr, &out);
                    }
                    // AMOOR.D
                    (0b01000, 0b011) => {
                        let inp = u64::from_le_bytes(self.memory.read_bytes(addr));
                        let out = (inp | self.x[rs2]).to_le_bytes();
                        self.x[rd] = inp;
                        self.memory.write_bytes(addr, &out);
                    }
                    // AMOXOR.W
                    (0b00100, 0b010) => {
                        let inp = i32::from_le_bytes(self.memory.read_bytes(addr)) as i64 as u64;
                        let out = ((inp ^ self.x[rs2]) as u32).to_le_bytes();
                        self.x[rd] = inp;
                        self.memory.write_bytes(addr, &out);
                    }
                    // AMOXOR.D
                    (0b00100, 0b011) => {
                        let inp = u64::from_le_bytes(self.memory.read_bytes(addr));
                        let out = (inp ^ self.x[rs2]).to_le_bytes();
                        self.x[rd] = inp;
                        self.memory.write_bytes(addr, &out);
                    }
                    // AMOMAX.W
                    (0b10100, 0b010) => {
                        let inp = i32::from_le_bytes(self.memory.read_bytes(addr));
                        let out = inp.max(self.x[rs2] as i32).to_le_bytes();
                        self.x[rd] = inp as u64;
                        self.memory.write_bytes(addr, &out);
                    }
                    // AMOMAX.D
                    (0b10100, 0b011) => {
                        let inp = i64::from_le_bytes(self.memory.read_bytes(addr));
                        let out = inp.max(self.x[rs2] as i64).to_le_bytes();
                        self.x[rd] = inp as u64;
                        self.memory.write_bytes(addr, &out);
                    }
                    // AMOMIN.W
                    (0b10000, 0b010) => {
                        let inp = i32::from_le_bytes(self.memory.read_bytes(addr));
                        let out = inp.min(self.x[rs2] as i32).to_le_bytes();
                        self.x[rd] = inp as u64;
                        self.memory.write_bytes(addr, &out);
                    }
                    // AMOMIN.D
                    (0b10000, 0b011) => {
                        let inp = i64::from_le_bytes(self.memory.read_bytes(addr));
                        let out = inp.min(self.x[rs2] as i64).to_le_bytes();
                        self.x[rd] = inp as u64;
                        self.memory.write_bytes(addr, &out);
                    }
                    // AMOMAXU.W
                    (0b11100, 0b010) => {
                        let inp = u32::from_le_bytes(self.memory.read_bytes(addr));
                        let out = inp.max(self.x[rs2] as u32).to_le_bytes();
                        self.x[rd] = inp as u64;
                        self.memory.write_bytes(addr, &out);
                    }
                    // AMOMAXU.D
                    (0b11100, 0b011) => {
                        let inp = u64::from_le_bytes(self.memory.read_bytes(addr));
                        let out = inp.max(self.x[rs2]).to_le_bytes();
                        self.x[rd] = inp;
                        self.memory.write_bytes(addr, &out);
                    }
                    // AMOMINU.W
                    (0b11000, 0b010) => {
                        let inp = u32::from_le_bytes(self.memory.read_bytes(addr));
                        let out = inp.min(self.x[rs2] as u32).to_le_bytes();
                        self.x[rd] = inp as u64;
                        self.memory.write_bytes(addr, &out);
                    }
                    // AMOMINU.D
                    (0b11000, 0b011) => {
                        let inp = u64::from_le_bytes(self.memory.read_bytes(addr));
                        let out = inp.min(self.x[rs2]).to_le_bytes();
                        self.x[rd] = inp;
                        self.memory.write_bytes(addr, &out);
                    }
                    _ => self.illegal_instruction(),
                }
            }

            // TODO floats/doubles:
            // - Set csr flags on illegal instruction etc
            // - Canonicalize NaNs (whatever that means)
            // - Actually represent 32 bit floats properly (lower bits of 64 bit NaN)
            0b0000111 => {
                let offset = (instruction >> 20 & 0xfff) as usize;
                let addr = (self.x[rs1] as usize).wrapping_add(offset);
                match funct3 {
                    // FLW
                    0b010 => {
                        self.f[rd] = f32::from_le_bytes(self.memory.read_bytes(addr)) as f64;
                    }
                    // FLD
                    0b011 => {
                        self.f[rd] = f64::from_le_bytes(self.memory.read_bytes(addr));
                    }
                    _ => self.illegal_instruction(),
                }
            }

            0b0100111 => {
                let offset = (instruction >> 7 & 0x1f | instruction >> 25 & 0xfe0) as usize;
                let addr = (self.x[rs1] as usize).wrapping_add(offset);
                match funct3 {
                    // FSW
                    0b010 => {
                        self.memory
                            .write_bytes(addr, &(self.f[rs1] as f32).to_le_bytes());
                    }
                    // FSD
                    0b011 => {
                        self.memory.write_bytes(addr, &(self.f[rs1]).to_le_bytes());
                    }
                    _ => self.illegal_instruction(),
                }
            }

            0b1010011 => {
                let funct5 = instruction >> 27 & 0x1f;
                let fmt = instruction >> 25 & 0x3;
                let rm = instruction >> 12 & 0x7;
                let a = self.f[rs1];
                let b = self.f[rs2];

                match (funct5, fmt, rm, rs2) {
                    // FADD.S
                    (0b00000, 0b00, _, _) => self.f[rd] = (a as f32 + b as f32) as f64,
                    // FADD.D
                    (0b00000, 0b01, _, _) => self.f[rd] = a + b,
                    // FSUB.S
                    (0b00001, 0b00, _, _) => self.f[rd] = (a as f32 - b as f32) as f64,
                    // FSUB.D
                    (0b00001, 0b01, _, _) => self.f[rd] = a - b,
                    // FMUL.S
                    (0b00010, 0b00, _, _) => self.f[rd] = (a as f32 * b as f32) as f64,
                    // FMUL.D
                    (0b00010, 0b01, _, _) => self.f[rd] = a * b,
                    // FDIV.S
                    (0b00011, 0b00, _, _) => self.f[rd] = (a as f32 / b as f32) as f64,
                    // FDIV.D
                    (0b00011, 0b01, _, _) => self.f[rd] = a / b,
                    // FMIN.S
                    (0b00101, 0b00, 0, _) => self.f[rd] = (a as f32).min(b as f32) as f64,
                    // FMIN.D
                    (0b00101, 0b01, 0, _) => self.f[rd] = a.min(b),
                    // FMAX.S
                    (0b00101, 0b00, 1, _) => self.f[rd] = (a as f32).max(b as f32) as f64,
                    // FMAX.D
                    (0b00101, 0b01, 1, _) => self.f[rd] = a.max(b),
                    // FSQRT.S
                    (0b00101, 0b00, _, 0) => self.f[rd] = (a as f32).sqrt() as f64,
                    // FSQRT.D
                    (0b00101, 0b01, _, 0) => self.f[rd] = a.sqrt(),
                    // FCVT.W.S
                    (0b11000, 0b00, _, 0) => self.x[rd] = self.f[rs1] as f32 as i32 as i64 as u64,
                    // FCVT.L.S
                    (0b11000, 0b00, _, 1) => self.x[rd] = self.f[rs1] as f32 as i64 as u64,
                    // FCVT.WU.S
                    (0b11000, 0b00, _, 2) => self.x[rd] = self.f[rs1] as f32 as u32 as u64,
                    // FCVT.LU.S
                    (0b11000, 0b00, _, 3) => self.x[rd] = self.f[rs1] as f32 as u64,
                    // FCVT.W.D
                    (0b11001, 0b01, _, 0) => self.x[rd] = self.f[rs1] as i32 as i64 as u64,
                    // FCVT.L.D
                    (0b11000, 0b01, _, 1) => self.x[rd] = self.f[rs1] as i64 as u64,
                    // FCVT.WU.D
                    (0b11000, 0b01, _, 2) => self.x[rd] = self.f[rs1] as u32 as u64,
                    // FCVT.LU.D
                    (0b11000, 0b01, _, 3) => self.x[rd] = self.f[rs1] as u64,
                    // FCVT.S.W
                    (0b11010, 0b00, _, 0) => self.f[rd] = self.x[rs1] as i32 as f32 as f64,
                    // FCVT.S.L
                    (0b11010, 0b00, _, 1) => self.f[rd] = self.x[rs1] as i64 as f32 as f64,
                    // FCVT.S.WU
                    (0b11010, 0b00, _, 2) => self.f[rd] = self.x[rs1] as f32 as f64,
                    // FCVT.S.LU
                    (0b11010, 0b00, _, 3) => self.f[rd] = self.x[rs1] as f32 as f64,
                    // FCVT.D.W
                    (0b11010, 0b01, _, 0) => self.f[rd] = self.x[rs1] as i32 as f64,
                    // FCVT.D.L
                    (0b11010, 0b01, _, 1) => self.f[rd] = self.x[rs1] as i64 as f64,
                    // FCVT.D.WU
                    (0b11010, 0b01, _, 2) => self.f[rd] = self.x[rs1] as f64,
                    // FCVT.D.LU
                    (0b11010, 0b01, _, 3) => self.f[rd] = self.x[rs1] as f64,
                    // FCVT.S.D
                    (0b01000, 0b00, _, 1) => self.f[rd] = self.f[rs1] as f32 as f64,
                    // FCVT.D.S
                    (0b01000, 0b01, _, 0) => self.f[rd] = self.f[rs1] as f32 as f64,
                    // FSGNJ.S
                    (0b00100, 0b00, 0, _) => self.f[rd] = (a as f32).copysign(b as f32) as f64,
                    // FSGNJN.S
                    (0b00100, 0b00, 1, _) => self.f[rd] = (a as f32).copysign(-b as f32) as f64,
                    // FSGNJX.S
                    (0b00100, 0b00, 2, _) => {
                        self.f[rd] = ((a as f32) * (-b as f32).signum()) as f64
                    }
                    // FSGNJ.D
                    (0b00100, 0b01, 0, _) => self.f[rd] = (a).copysign(b),
                    // FSGNJN.D
                    (0b00100, 0b01, 1, _) => self.f[rd] = (a).copysign(-b),
                    // FSGNJX.D
                    (0b00100, 0b01, 2, _) => self.f[rd] = a * -b.signum(),
                    // FMV.X.W
                    (0b11100, 0b00, 0, 0) => self.x[rd] = (self.f[rs1] as f32).to_bits() as u64,
                    // FMV.W.X
                    (0b11110, 0b00, 0, 0) => self.f[rd] = f32::from_bits(self.x[rs1] as u32) as f64,
                    // FMV.X.D
                    (0b11100, 0b01, 0, 0) => self.x[rd] = self.f[rs1].to_bits(),
                    // FMV.D.X
                    (0b11110, 0b01, 0, 0) => self.f[rd] = f64::from_bits(self.x[rs1]),
                    // FEQ.S
                    (0b10100, 0b00, 0b010, _) => self.x[rd] = ((a as f32) == (b as f32)) as u64,
                    // FEQ.D
                    (0b10100, 0b01, 0b010, _) => self.x[rd] = (a == b) as u64,
                    // FLT.S
                    (0b10100, 0b00, 0b001, _) => self.x[rd] = ((a as f32) < (b as f32)) as u64,
                    // FLT.D
                    (0b10100, 0b01, 0b001, _) => self.x[rd] = (a < b) as u64,
                    // FLE.S
                    (0b10100, 0b00, 0b000, _) => self.x[rd] = ((a as f32) <= (b as f32)) as u64,
                    // FLE.D
                    (0b10100, 0b01, 0b000, _) => self.x[rd] = (a <= b) as u64,
                    // FCLASS.S
                    (0b11100, 0b00, 0b001, 0) => {
                        use std::num::FpCategory;
                        self.x[rd] = match ((a as f32).classify(), a >= 0.) {
                            (FpCategory::Infinite, false) => 1 << 0,
                            (FpCategory::Normal, false) => 1 << 1,
                            (FpCategory::Subnormal, false) => 1 << 2,
                            (FpCategory::Zero, false) => 1 << 3,
                            (FpCategory::Zero, true) => 1 << 4,
                            (FpCategory::Subnormal, true) => 1 << 5,
                            (FpCategory::Normal, true) => 1 << 6,
                            (FpCategory::Infinite, true) => 1 << 7,
                            (FpCategory::Nan, false) => 1 << 8,
                            (FpCategory::Nan, true) => 1 << 9,
                        }
                    }
                    // FCLASS.D
                    (0b11100, 0b01, 0b001, 0) => {
                        use std::num::FpCategory;
                        self.x[rd] = match (a.classify(), a >= 0.) {
                            (FpCategory::Infinite, false) => 1 << 0,
                            (FpCategory::Normal, false) => 1 << 1,
                            (FpCategory::Subnormal, false) => 1 << 2,
                            (FpCategory::Zero, false) => 1 << 3,
                            (FpCategory::Zero, true) => 1 << 4,
                            (FpCategory::Subnormal, true) => 1 << 5,
                            (FpCategory::Normal, true) => 1 << 6,
                            (FpCategory::Infinite, true) => 1 << 7,
                            (FpCategory::Nan, false) => 1 << 8,
                            (FpCategory::Nan, true) => 1 << 9,
                        }
                    }
                    _ => self.illegal_instruction(),
                }
            }

            // TODO raise exceptions on 0 * infinity or something like that
            0b1000011 => {
                let rs3 = (instruction >> 27 & 0x1f) as usize;
                let fmt = instruction >> 25 & 0x3;
                let a = self.f[rs1];
                let b = self.f[rs2];
                let c = self.f[rs3];
                match fmt {
                    // FMADD.S
                    0b00 => self.f[rd] = ((a as f32) * (b as f32) + (c as f32)) as f64,
                    // FMADD.D
                    0b01 => self.f[rd] = a * b + c,
                    _ => self.illegal_instruction(),
                }
            }

            0b1000111 => {
                let rs3 = (instruction >> 27 & 0x1f) as usize;
                let fmt = instruction >> 25 & 0x3;
                let a = self.f[rs1];
                let b = self.f[rs2];
                let c = self.f[rs3];
                match fmt {
                    // FMSUB.S
                    0b00 => self.f[rd] = ((a as f32) * (b as f32) - (c as f32)) as f64,
                    // FMSUB.D
                    0b01 => self.f[rd] = a * b - c,
                    _ => self.illegal_instruction(),
                }
            }

            0b1001011 => {
                let rs3 = (instruction >> 27 & 0x1f) as usize;
                let fmt = instruction >> 25 & 0x3;
                let a = self.f[rs1];
                let b = self.f[rs2];
                let c = self.f[rs3];
                match fmt {
                    // FNMADD.S
                    0b00 => self.f[rd] = (-((a as f32) * (b as f32)) + (c as f32)) as f64,
                    // FNMADD.D
                    0b01 => self.f[rd] = -(a * b) + c,
                    _ => self.illegal_instruction(),
                }
            }

            0b1001111 => {
                let rs3 = (instruction >> 27 & 0x1f) as usize;
                let fmt = instruction >> 25 & 0x3;
                let a = self.f[rs1];
                let b = self.f[rs2];
                let c = self.f[rs3];
                match fmt {
                    // FNMSUB.S
                    0b00 => self.f[rd] = (-((a as f32) * (b as f32)) - (c as f32)) as f64,
                    // FNMSUB.D
                    0b01 => self.f[rd] = -(a * b) - c,
                    _ => self.illegal_instruction(),
                }
            }

            _ => self.illegal_instruction(),
        }

        self.increment_counters();

        self.handle_traps();

        self.pc = self.pc.wrapping_add(4);
    }
}

fn main() {
    let mut computer = Emulator::new(128 * 1024 * 1024);

    //computer.load_binary("a.out", 0x1000).unwrap();
    computer
        .load_binary("../riscv-tests/isa/rv64um-p-remw", 0x1000)
        .unwrap();

    // TODO
    // - M
    // - A
    // - C
    // - F
    // - D
    // - Zicsr

    loop {
        computer.run_instruction();
    }
}
