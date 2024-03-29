use std::io::{Read, Write};

struct Cpu {
    x: [u64; 32],
    pc: u64,
}

impl Cpu {
    fn new() -> Self {
        Cpu { x: [0; 32], pc: 0 }
    }
}

struct Memory {
    bytes: Box<[u8]>,
    size: usize,
}

impl Memory {
    fn new(size: usize) -> Memory {
        Memory {
            bytes: vec![0; size].into(),
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

struct Computer {
    cpu: Cpu,
    memory: Memory,
}

impl Computer {
    fn new(mem_size: usize) -> Self {
        Computer {
            cpu: Cpu::new(),
            memory: Memory::new(mem_size),
        }
    }

    fn load_binary(&mut self, file_name: &str, entry_point: u64) -> std::io::Result<()> {
        let mut file = std::fs::File::open(file_name)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        self.memory.write_bytes(0, &buf);
        self.cpu.pc = entry_point;
        Ok(())
    }

    fn syscall(&mut self) {
        match self.cpu.x[17] {
            0x01 => {
                // print
                let mut i = self.cpu.x[10] as usize;
                'print: loop {
                    let [byte] = self.memory.read_bytes(i);
                    if byte == 0 {
                        break 'print;
                    }
                    i = (i + 1) % self.memory.size;
                    print!("{}", byte as char);
                    std::io::stdout().flush().unwrap();
                }
            }
            0x04 => {
                // exit
                std::process::exit(self.cpu.x[10] as i32)
            }
            _ => (), // unimplemented syscall
        }
    }

    fn run_instruction(&mut self) {
        // the x0 register should always be 0 (hopefully it doesn't get written to and then used)
        self.cpu.x[0] = 0;

        let instruction =
            u32::from_le_bytes(self.memory.read_bytes::<4>(self.cpu.pc as usize));
        let opcode = instruction & 0x7f;
        let rd = ((instruction >> 7) & 0x1f) as usize;
        let rs1 = ((instruction >> 15) & 0x1f) as usize;
        let rs2 = ((instruction >> 20) & 0x1f) as usize;
        let funct3 = (instruction >> 12) & 0x7;
        let funct7 = (instruction >> 25) & 0x7f;

        // We implement RV64I

        match opcode {
            // Immediate instructions
            0b0010011 => {
                // note that we convert instruction to an i32 for sign extension.
                let uinp = self.cpu.x[rs1];
                let iinp = uinp as i64;
                let uimm = (((instruction as i32) >> 20) % 0x1000) as u64;
                let iimm = uimm as i64;
                match funct3 {
                    // ADDI
                    0b000 => self.cpu.x[rd] = (iinp + iimm) as u64,
                    // SLTI
                    0b001 => self.cpu.x[rd] = if iinp < iimm { 1 } else { 0 },
                    // SLTIU
                    0b010 => self.cpu.x[rd] = if uinp < uimm { 1 } else { 0 },
                    // XORI
                    0b011 => self.cpu.x[rd] = uimm ^ uinp,
                    // ORI
                    0b100 => self.cpu.x[rd] = uimm | uinp,
                    // ANDI
                    0b101 => self.cpu.x[rd] = uimm & uinp,
                    0b110 => {
                        let upper = (instruction >> 26) & 0x3f;
                        let shamt = (instruction >> 20) & 0x3f;
                        match upper {
                            // SLLI
                            0b000000 => self.cpu.x[rd] = uinp << shamt,
                            _ => panic!("Unimplemented instruction {instruction:b}"),
                        }
                    }
                    0b111 => {
                        let upper = (instruction >> 26) & 0x3f;
                        let shamt = (instruction >> 20) & 0x3f;
                        match upper {
                            // SRLI
                            0b0000000 => self.cpu.x[rd] = uinp >> shamt,
                            // SRAI
                            0b010000 => self.cpu.x[rd] = (uinp >> shamt) as u64,
                            _ => panic!("Unimplemented instruction {instruction:b}"),
                        }
                    }
                    _ => unreachable!(),
                }
            }

            0b0011011 => {
                let uinp = (self.cpu.x[rs1] & 0xffffffff) as u32;
                let iinp = uinp as i32;
                match funct3 {
                    // ADDIW
                    0b000 => {
                        let imm = ((instruction as i32) >> 20) & 0xfff;
                        self.cpu.x[rd] = (imm + iinp) as i64 as u64
                    }
                    0b001 => {
                        let upper = (instruction >> 25) & 0x1f;
                        match upper {
                            // SLLIW
                            0b000000 => {
                                let shamt = (instruction >> 20) & 0x1f;
                                self.cpu.x[rd] = (uinp << shamt) as u64;
                            }
                            _ => panic!("Unimplemented instruction {instruction:b}"),
                        }
                    }
                    0b101 => {
                        let upper = (instruction >> 25) & 0x1f;
                        match upper {
                            // SRLIW
                            0b000000 => {
                                let shamt = (instruction >> 20) & 0x1f;
                                self.cpu.x[rd] = (uinp >> shamt) as u64;
                            }
                            // SRAIW
                            0b010000 => {
                                let shamt = (instruction >> 20) & 0x1f;
                                self.cpu.x[rd] = (iinp >> shamt) as u64;
                            }
                            _ => panic!("Unimplemented instruction {instruction:b}"),
                        }
                    }
                    _ => panic!("Unimplemented instruction {instruction:b}"),
                }
            }

            // LUI
            0b0110111 => {
                let imm = (instruction) & (0xfffff << 12);
                self.cpu.x[rd] = imm as u64;
            }

            // AUIPC
            0b0010111 => {
                let imm = (instruction) & (0xfffff << 12);
                self.cpu.x[rd] = imm as u64 + self.cpu.pc;
            }

            // Register instructions
            0b0110011 => {
                match (funct3, funct7) {
                    // ADD
                    (0b000, 0b0000000) => {
                        self.cpu.x[rd] = self.cpu.x[rs1].wrapping_add(self.cpu.x[rs2])
                    }
                    // SUB
                    (0b000, 0b0100000) => {
                        self.cpu.x[rd] = self.cpu.x[rs1].wrapping_sub(self.cpu.x[rs2])
                    }
                    // SLT
                    (0b010, 0b0000000) => {
                        self.cpu.x[rd] = if (self.cpu.x[rs1] as i64) < (self.cpu.x[rs2] as i64) {
                            1
                        } else {
                            0
                        }
                    }
                    // SLTU
                    (0b011, 0b0000000) => {
                        self.cpu.x[rd] = if self.cpu.x[rs1] < self.cpu.x[rs2] {
                            1
                        } else {
                            0
                        }
                    }
                    // XOR
                    (0b100, 0b0000000) => self.cpu.x[rd] = self.cpu.x[rs1] ^ self.cpu.x[rs2],
                    // OR
                    (0b110, 0b0000000) => self.cpu.x[rd] = self.cpu.x[rs1] | self.cpu.x[rs2],
                    // AND
                    (0b111, 0b0000000) => self.cpu.x[rd] = self.cpu.x[rs1] & self.cpu.x[rs2],
                    // SLL
                    (0b001, 0b0000000) => {
                        self.cpu.x[rd] = self.cpu.x[rs1] << (self.cpu.x[rs2] & 0x3f)
                    }
                    // SRL
                    (0b101, 0b0000000) => {
                        self.cpu.x[rd] = self.cpu.x[rs1] >> (self.cpu.x[rs2] & 0x3f)
                    }
                    // SRA
                    (0b101, 0b0100000) => {
                        self.cpu.x[rd] =
                            ((self.cpu.x[rs1] as i64) >> (self.cpu.x[rs2] & 0x3f)) as u64
                    }
                    _ => panic!("Unimplemented instruction {instruction:b}"),
                }
            }

            0b0111011 => {
                match (funct3, funct7) {
                    // ADDW
                    (0b000, 0b0000000) => {
                        self.cpu.x[rd] =
                            (self.cpu.x[rs1] as u32).wrapping_add(self.cpu.x[rs2] as u32) as u64
                    }
                    // SUBW
                    (0b000, 0b0100000) => {
                        self.cpu.x[rd] =
                            (self.cpu.x[rs1] as u32).wrapping_sub(self.cpu.x[rs2] as u32) as u64
                    }
                    // SLLW
                    (0b001, 0b0000000) => {
                        self.cpu.x[rd] =
                            ((self.cpu.x[rs1] as u32) << (self.cpu.x[rs2] as u32 & 0x1f)) as u64
                    }
                    // SRLW
                    (0b101, 0b0000000) => {
                        self.cpu.x[rd] =
                            ((self.cpu.x[rs1] as u32) >> (self.cpu.x[rs2] as u32 & 0x1f)) as u64
                    }
                    // SRAW
                    (0b101, 0b0100000) => {
                        self.cpu.x[rd] = ((self.cpu.x[rs1] as i32)
                            >> (self.cpu.x[rs2] as u32 & 0x1f))
                            as i64 as u64
                    }
                    _ => panic!("Unimplemented instruction {instruction:b}"),
                }
            }

            // Control transfer instructions

            // JAL
            0b1101111 => {
                // i don't trust this TODO double check
                let offset = (instruction >> 21) & 0x3ff
                    | (instruction >> 10) & 0x400
                    | (instruction >> 1) & 0x7f800
                    | (instruction >> 12) & 0x80000;
                let offset = ((offset as i32) << 12 >> 12) as i64;
                let offset = offset * 2;
                self.cpu.x[rd] = self.cpu.pc;
                self.cpu.pc = self.cpu.pc.wrapping_add(offset as u64).wrapping_sub(4);
                if self.cpu.pc % 4 != 0 {
                    panic!("Jumped to misaligned instruction")
                }
            }
            0b1100111 => {
                // JALR
                if funct3 == 0b000 {
                    let offset = ((instruction >> 20) & 0xfff) as u64;
                    self.cpu.x[rd] = self.cpu.pc;
                    self.cpu.pc = self.cpu.x[rs1].wrapping_add(offset);
                    if self.cpu.pc % 4 != 0 {
                        panic!("Jumped to misaligned instruction")
                    }
                } else {
                    panic!("Unimplemented instruction {instruction:b}");
                }
            }
            0b1100011 => {
                // i don't trust this TODO double check
                let offset = (instruction >> 8) & 0xf
                    | (instruction >> 21) & 0x1f0
                    | (instruction << 2) & 0x200
                    | (instruction >> 20) & 0x400;
                let offset = ((offset as i32) << 20 >> 20) as u64;
                let offset = (offset * 2).wrapping_sub(4096);
                match funct3 {
                    // BEQ
                    0b000 => {
                        if self.cpu.x[rs1] == self.cpu.x[rs2] {
                            self.cpu.pc = self.cpu.pc.wrapping_add(offset).wrapping_sub(4);
                        }
                    }
                    // BNE
                    0b001 => {
                        if self.cpu.x[rs1] != self.cpu.x[rs2] {
                            self.cpu.pc = self.cpu.pc.wrapping_add(offset).wrapping_sub(4);
                        }
                    }
                    // BLT
                    0b100 => {
                        if (self.cpu.x[rs1] as i64) < (self.cpu.x[rs2] as i64) {
                            self.cpu.pc = self.cpu.pc.wrapping_add(offset).wrapping_sub(4);
                        }
                    }
                    // BLTU
                    0b110 => {
                        if self.cpu.x[rs1] < self.cpu.x[rs2] {
                            self.cpu.pc = self.cpu.pc.wrapping_add(offset).wrapping_sub(4);
                        }
                    }
                    // BGE
                    0b101 => {
                        if (self.cpu.x[rs1] as i64) >= (self.cpu.x[rs2] as i64) {
                            self.cpu.pc = self.cpu.pc.wrapping_add(offset).wrapping_sub(4);
                        }
                    }
                    // BGTU
                    0b111 => {
                        if self.cpu.x[rs1] >= self.cpu.x[rs2] {
                            self.cpu.pc = self.cpu.pc.wrapping_add(offset).wrapping_sub(4);
                        }
                    }
                    _ => panic!("Unimplemented instruction {instruction:b}"),
                }
                if self.cpu.pc % 4 != 0 {
                    panic!("Jumped to misaligned instruction")
                }
            }

            // Load and Store
            0b0000011 => {
                let imm = (((instruction as i32) >> 20) & 0xfff) as u64;
                let addr = self.cpu.x[rs1].wrapping_add(imm) as usize;
                match funct3 {
                    // LB
                    0b000 => {
                        self.cpu.x[rd] =
                            i8::from_le_bytes(self.memory.read_bytes(addr)) as i64 as u64
                    }
                    // LH
                    0b001 => {
                        self.cpu.x[rd] =
                            i16::from_le_bytes(self.memory.read_bytes(addr)) as i64 as u64
                    }
                    // LW
                    0b010 => {
                        self.cpu.x[rd] =
                            i32::from_le_bytes(self.memory.read_bytes(addr)) as i64 as u64
                    }
                    // LBU
                    0b100 => {
                        self.cpu.x[rd] = u8::from_le_bytes(self.memory.read_bytes(addr)) as u64
                    }
                    // LHU
                    0b101 => {
                        self.cpu.x[rd] =
                            u16::from_le_bytes(self.memory.read_bytes(addr)) as u64
                    }
                    // LWU
                    0b110 => {
                        self.cpu.x[rd] =
                            u32::from_le_bytes(self.memory.read_bytes(addr)) as u64
                    }
                    // LD
                    0b011 => {
                        self.cpu.x[rd] =
                            u64::from_le_bytes(self.memory.read_bytes(addr)) as u64
                    }
                    _ => panic!("Unimplemented instruction {instruction:b}"),
                }
            }
            0b0100011 => {
                let imm = (((instruction as i64) >> 7) & 0x1f
                    | ((instruction as i64) >> 20) & 0xfe0) as u64;
                let addr = self.cpu.x[rs1].wrapping_add(imm) as usize;
                match funct3 {
                    // SB
                    0b000 => self
                        .memory
                        .write_bytes(addr, &(self.cpu.x[rs2] as u8).to_le_bytes()),
                    // SH
                    0b001 => self
                        .memory
                        .write_bytes(addr, &(self.cpu.x[rs2] as u16).to_le_bytes()),
                    // SW
                    0b010 => self
                        .memory
                        .write_bytes(addr, &(self.cpu.x[rs2] as u32).to_le_bytes()),
                    // SD
                    0b011 => self
                        .memory
                        .write_bytes(addr, &self.cpu.x[rs2].to_le_bytes()),
                    _ => panic!("Unimplemented instruction {instruction:b}"),
                }
            }

            // Memory ordering instructions

            // FENCE
            0b0001111 => {
                todo!("FENCE");
            }

            // Environment call and breakpoints
            0b1110011 => match instruction {
                // ECALL
                0b00000000000000000000000001110011 => self.syscall(),
                // EBREAK
                0b00000000000100000000000001110011 => {
                    todo!("EBREAK");
                }
                _ => panic!("Unimplemented instruction {instruction:b}"),
            },

            _ => panic!("Unimplemented instruction {instruction:b}"),
        }

        self.cpu.pc += 4;
    }
}

fn main() {
    let mut computer = Computer::new(128 * 1024 * 1024);

    computer.load_binary("a.out", 0x1000).unwrap();

    loop {
        computer.run_instruction();
    }
}
