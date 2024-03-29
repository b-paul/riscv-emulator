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

    fn read_bytes<const COUNT: usize>(&self, mut addr: usize) -> [u8; COUNT] {
        let mut out = [0; COUNT];
        for b in out.iter_mut() {
            *b = self.bytes[addr];
            addr += 1;
            addr %= self.size;
        }
        out
    }

    fn write_bytes<const COUNT: usize>(&mut self, mut addr: usize, bytes: [u8; COUNT]) {
        for b in bytes {
            self.bytes[addr] = b;
            addr += 1;
            addr %= self.size;
        }
    }

    fn write_bytes_var(&mut self, mut addr: usize, bytes: &[u8]) {
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
        self.memory.write_bytes_var(0, &buf);
        self.cpu.pc = entry_point;
        Ok(())
    }

    fn syscall(&mut self) {
        match self.cpu.x[17] {
            0x01 => {
                // print
                let mut i = self.cpu.x[10] as usize;
                'print: loop {
                    let bytes = self.memory.read_bytes::<8>(i);
                    i += 8;
                    i %= self.memory.size;

                    for c in bytes {
                        if c == 0 {
                            break 'print;
                        }
                        let c: char = c.into();
                        print!("{}", c);
                        std::io::stdout().flush().unwrap();
                    }
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

        let instruction = u32::from_le_bytes(self.memory.read_bytes::<4>(self.cpu.pc as usize));
        let opcode = instruction & 0x7f;

        // We implement RV64I

        match opcode {
            // Immediate instructions
            0b0010011 => {
                let rd = ((instruction >> 7) & 0x1f) as usize;
                let rs1 = ((instruction >> 15) & 0x1f) as usize;
                let funct3 = (instruction >> 12) & 0x7;

                // note that we convert instruction to an i32 for sign extension.
                match funct3 {
                    0b000 => {
                        // ADDI
                        let imm = (((instruction as i32) >> 20) % 0x1000) as i64;
                        let inp = self.cpu.x[rs1] as i64;
                        self.cpu.x[rd] = (inp + imm) as u64;
                    }
                    0b001 => {
                        // SLTI
                        let imm = (((instruction as i32) >> 20) % 0x1000) as i64;
                        let inp = self.cpu.x[rs1] as i64;
                        self.cpu.x[rd] = if inp < imm { 1 } else { 0 };
                    }
                    0b010 => {
                        // SLTIU
                        let imm = (((instruction as i32) >> 20) % 0x1000) as u64;
                        let inp = self.cpu.x[rs1];
                        self.cpu.x[rd] = if inp < imm { 1 } else { 0 };
                    }
                    0b011 => {
                        // XORI
                        let imm = (((instruction as i32) >> 20) % 0x1000) as u64;
                        let inp = self.cpu.x[rs1];
                        self.cpu.x[rd] = imm ^ inp;
                    }
                    0b100 => {
                        // ORI
                        let imm = (((instruction as i32) >> 20) % 0x1000) as u64;
                        let inp = self.cpu.x[rs1];
                        self.cpu.x[rd] = imm | inp;
                    }
                    0b101 => {
                        // ANDI
                        let imm = (((instruction as i32) >> 20) % 0x1000) as u64;
                        let inp = self.cpu.x[rs1];
                        self.cpu.x[rd] = imm & inp;
                    }
                    0b110 => {
                        let upper = (instruction >> 26) & 0x3f;
                        match upper {
                            0b000000 => {
                                // SLLI
                                let shamt = (instruction >> 20) & 0x3f;
                                let inp = self.cpu.x[rs1];
                                self.cpu.x[rd] = inp << shamt;
                            }
                            _ => panic!("Unimplemented instruction {instruction:b}"),
                        }
                    }
                    0b111 => {
                        let upper = (instruction >> 26) & 0x3f;
                        match upper {
                            0b0000000 => {
                                // SRLI
                                let shamt = (instruction >> 20) & 0x3f;
                                let inp = self.cpu.x[rs1];
                                self.cpu.x[rd] = inp >> shamt;
                            }
                            0b010000 => {
                                // SRAI
                                let shamt = (instruction >> 20) & 0x3f;
                                let inp = self.cpu.x[rs1] as i64;
                                self.cpu.x[rd] = (inp >> shamt) as u64;
                            }
                            _ => panic!("Unimplemented instruction {instruction:b}"),
                        }
                    }
                    _ => unreachable!(),
                }
            }

            0b0011011 => {
                let rd = ((instruction >> 7) & 0x1f) as usize;
                let rs1 = ((instruction >> 15) & 0x1f) as usize;
                let funct3 = (instruction >> 12) & 0x7;

                match funct3 {
                    0b000 => {
                        // ADDIW
                        let imm = ((instruction as i32) >> 20) & 0xfff;
                        let inp = (self.cpu.x[rs1] & 0xffffffff) as i32;
                        self.cpu.x[rd] = (imm + inp) as i64 as u64
                    }
                    0b001 => {
                        let upper = (instruction >> 25) & 0x1f;
                        match upper {
                            0b000000 => {
                                // SLLIW
                                let shamt = (instruction >> 20) & 0x1f;
                                let inp = (self.cpu.x[rs1] & 0xffffffff) as u32;
                                self.cpu.x[rd] = (inp << shamt) as u64;
                            }
                            _ => panic!("Unimplemented instruction {instruction:b}"),
                        }
                    }
                    0b101 => {
                        let upper = (instruction >> 25) & 0x1f;
                        match upper {
                            0b000000 => {
                                // SRLIW
                                let shamt = (instruction >> 20) & 0x1f;
                                let inp = (self.cpu.x[rs1] & 0xffffffff) as i32;
                                self.cpu.x[rd] = (inp >> shamt) as u64;
                            }
                            0b010000 => {
                                // SRAIW
                                let shamt = (instruction >> 20) & 0x1f;
                                let inp = (self.cpu.x[rs1] & 0xffffffff) as u32;
                                self.cpu.x[rd] = (inp >> shamt) as u64;
                            }
                            _ => panic!("Unimplemented instruction {instruction:b}"),
                        }
                    }
                    _ => panic!("Unimplemented instruction {instruction:b}"),
                }
            }

            0b0110111 => {
                // LUI
                let rd = ((instruction >> 7) & 0x1f) as usize;
                let imm = (instruction) & (0xfffff << 12);
                self.cpu.x[rd] = imm as u64;
            }

            0b0010111 => {
                // AUIPC
                let rd = ((instruction >> 7) & 0x1f) as usize;
                let imm = (instruction) & (0xfffff << 12);
                self.cpu.x[rd] = imm as u64 + self.cpu.pc;
            }

            // Register instructions
            0b0110011 => {
                let rd = ((instruction >> 7) & 0x1f) as usize;
                let rs1 = ((instruction >> 15) & 0x1f) as usize;
                let rs2 = ((instruction >> 20) & 0x1f) as usize;
                let funct3 = (instruction >> 12) & 0x7;
                let funct7 = (instruction >> 25) & 0x7f;

                match (funct3, funct7) {
                    (0b000, 0b0000000) => {
                        // ADD
                        self.cpu.x[rd] = self.cpu.x[rs1].wrapping_add(self.cpu.x[rs2]);
                    }
                    (0b000, 0b0100000) => {
                        // SUB
                        self.cpu.x[rd] = self.cpu.x[rs1].wrapping_sub(self.cpu.x[rs2]);
                    }
                    (0b010, 0b0000000) => {
                        // SLT
                        self.cpu.x[rd] = if (self.cpu.x[rs1] as i64) < (self.cpu.x[rs2] as i64) {
                            1
                        } else {
                            0
                        };
                    }
                    (0b011, 0b0000000) => {
                        // SLTU
                        self.cpu.x[rd] = if self.cpu.x[rs1] < self.cpu.x[rs2] {
                            1
                        } else {
                            0
                        };
                    }
                    (0b100, 0b0000000) => {
                        // XOR
                        self.cpu.x[rd] = self.cpu.x[rs1] ^ self.cpu.x[rs2];
                    }
                    (0b110, 0b0000000) => {
                        // OR
                        self.cpu.x[rd] = self.cpu.x[rs1] | self.cpu.x[rs2];
                    }
                    (0b111, 0b0000000) => {
                        // AND
                        self.cpu.x[rd] = self.cpu.x[rs1] & self.cpu.x[rs2];
                    }
                    (0b001, 0b0000000) => {
                        // SLL
                        self.cpu.x[rd] = self.cpu.x[rs1] << (self.cpu.x[rs2] & 0x3f);
                    }
                    (0b101, 0b0000000) => {
                        // SRL
                        self.cpu.x[rd] = self.cpu.x[rs1] >> (self.cpu.x[rs2] & 0x3f);
                    }
                    (0b101, 0b0100000) => {
                        // SRA
                        self.cpu.x[rd] =
                            ((self.cpu.x[rs1] as i64) >> (self.cpu.x[rs2] & 0x3f)) as u64;
                    }
                    _ => panic!("Unimplemented instruction {instruction:b}"),
                }
            }

            0b0111011 => {
                let rd = ((instruction >> 7) & 0x1f) as usize;
                let rs1 = ((instruction >> 15) & 0x1f) as usize;
                let rs2 = ((instruction >> 20) & 0x1f) as usize;
                let funct3 = (instruction >> 12) & 0x7;
                let funct7 = (instruction >> 25) & 0x7f;
                match (funct3, funct7) {
                    (0b000, 0b0000000) => {
                        // ADDW
                        self.cpu.x[rd] =
                            (self.cpu.x[rs1] as u32).wrapping_add(self.cpu.x[rs2] as u32) as u64;
                    }
                    (0b000, 0b0100000) => {
                        // SUBW
                        self.cpu.x[rd] =
                            (self.cpu.x[rs1] as u32).wrapping_sub(self.cpu.x[rs2] as u32) as u64;
                    }
                    (0b001, 0b0000000) => {
                        // SLLW
                        self.cpu.x[rd] =
                            ((self.cpu.x[rs1] as u32) << (self.cpu.x[rs2] as u32 & 0x1f)) as u64;
                    }
                    (0b101, 0b0000000) => {
                        // SRLW
                        self.cpu.x[rd] =
                            ((self.cpu.x[rs1] as u32) >> (self.cpu.x[rs2] as u32 & 0x1f)) as u64;
                    }
                    (0b101, 0b0100000) => {
                        // SRAW
                        self.cpu.x[rd] = ((self.cpu.x[rs1] as i32)
                            >> (self.cpu.x[rs2] as u32 & 0x1f))
                            as i64 as u64;
                    }
                    _ => panic!("Unimplemented instruction {instruction:b}"),
                }
            }

            // Control transfer instructions
            0b1101111 => {
                // JAL
                let rd = ((instruction >> 7) & 0x1f) as usize;
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
                let funct3 = (instruction >> 12) & 0x7;
                if funct3 == 0b000 {
                    // JALR
                    let rd = ((instruction >> 7) & 0x1f) as usize;
                    let rs1 = ((instruction >> 15) & 0x1f) as usize;
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
                let rs1 = ((instruction >> 15) & 0x1f) as usize;
                let rs2 = ((instruction >> 20) & 0x1f) as usize;
                // i don't trust this TODO double check
                let offset = (instruction >> 8) & 0xf
                    | (instruction >> 21) & 0x1f0
                    | (instruction << 2) & 0x200
                    | (instruction >> 20) & 0x400;
                let offset = ((offset as i32) << 20 >> 20) as u64;
                let offset = (offset * 2).wrapping_sub(4096);
                let funct3 = (instruction >> 12) & 0x7;
                match funct3 {
                    0b000 => {
                        // BEQ
                        if self.cpu.x[rs1] == self.cpu.x[rs2] {
                            self.cpu.pc = self.cpu.pc.wrapping_add(offset).wrapping_sub(4);
                            if self.cpu.pc % 4 != 0 {
                                panic!("Jumped to misaligned instruction")
                            }
                        }
                    }
                    0b001 => {
                        // BNE
                        if self.cpu.x[rs1] != self.cpu.x[rs2] {
                            self.cpu.pc = self.cpu.pc.wrapping_add(offset).wrapping_sub(4);
                            if self.cpu.pc % 4 != 0 {
                                panic!("Jumped to misaligned instruction")
                            }
                        }
                    }
                    0b100 => {
                        // BLT
                        if (self.cpu.x[rs1] as i64) < (self.cpu.x[rs2] as i64) {
                            self.cpu.pc = self.cpu.pc.wrapping_add(offset).wrapping_sub(4);
                            if self.cpu.pc % 4 != 0 {
                                panic!("Jumped to misaligned instruction")
                            }
                        }
                    }
                    0b110 => {
                        // BLTU
                        if self.cpu.x[rs1] < self.cpu.x[rs2] {
                            self.cpu.pc = self.cpu.pc.wrapping_add(offset).wrapping_sub(4);
                            if self.cpu.pc % 4 != 0 {
                                panic!("Jumped to misaligned instruction")
                            }
                        }
                    }
                    0b101 => {
                        // BGE
                        if (self.cpu.x[rs1] as i64) >= (self.cpu.x[rs2] as i64) {
                            self.cpu.pc = self.cpu.pc.wrapping_add(offset).wrapping_sub(4);
                            if self.cpu.pc % 4 != 0 {
                                panic!("Jumped to misaligned instruction")
                            }
                        }
                    }
                    0b111 => {
                        // BGTU
                        if self.cpu.x[rs1] >= self.cpu.x[rs2] {
                            self.cpu.pc = self.cpu.pc.wrapping_add(offset).wrapping_sub(4);
                            if self.cpu.pc % 4 != 0 {
                                panic!("Jumped to misaligned instruction")
                            }
                        }
                    }
                    _ => panic!("Unimplemented instruction {instruction:b}"),
                }
            }

            // Load and Store
            0b0000011 => {
                let rd = ((instruction >> 7) & 0x1f) as usize;
                let rs1 = ((instruction >> 15) & 0x1f) as usize;
                let funct3 = (instruction >> 12) & 0x7;
                let imm = (((instruction as i32) >> 20) & 0xfff) as u64;
                let addr = self.cpu.x[rs1].wrapping_add(imm) as usize;
                match funct3 {
                    0b000 => {
                        // LB
                        self.cpu.x[rd] =
                            i8::from_le_bytes(self.memory.read_bytes::<1>(addr)) as i64 as u64;
                    }
                    0b001 => {
                        // LH
                        self.cpu.x[rd] =
                            i16::from_le_bytes(self.memory.read_bytes::<2>(addr)) as i64 as u64;
                    }
                    0b010 => {
                        // LW
                        self.cpu.x[rd] =
                            i32::from_le_bytes(self.memory.read_bytes::<4>(addr)) as i64 as u64;
                    }
                    0b100 => {
                        // LBU
                        self.cpu.x[rd] =
                            u8::from_le_bytes(self.memory.read_bytes::<1>(addr)) as u64;
                    }
                    0b101 => {
                        // LHU
                        self.cpu.x[rd] =
                            u16::from_le_bytes(self.memory.read_bytes::<2>(addr)) as u64;
                    }
                    0b110 => {
                        // LWU
                        self.cpu.x[rd] =
                            u32::from_le_bytes(self.memory.read_bytes::<4>(addr)) as u64;
                    }
                    0b011 => {
                        // LD
                        self.cpu.x[rd] =
                            u64::from_le_bytes(self.memory.read_bytes::<8>(addr)) as u64;
                    }
                    _ => panic!("Unimplemented instruction {instruction:b}"),
                }
            }
            0b0100011 => {
                let rs1 = ((instruction >> 15) & 0x1f) as usize;
                let rs2 = ((instruction >> 20) & 0x1f) as usize;
                let funct3 = (instruction >> 12) & 0x7;
                let imm =
                    (((instruction as i64) >> 7) & 0x1f | ((instruction as i64) >> 20) & 0xfe0) as u64;
                let addr = self.cpu.x[rs1].wrapping_add(imm) as usize;
                match funct3 {
                    0b000 => {
                        // SB
                        self.memory
                            .write_bytes(addr, (self.cpu.x[rs2] as u8).to_le_bytes());
                    }
                    0b001 => {
                        // SH
                        self.memory
                            .write_bytes(addr, (self.cpu.x[rs2] as u16).to_le_bytes());
                    }
                    0b010 => {
                        // SW
                        self.memory
                            .write_bytes(addr, (self.cpu.x[rs2] as u32).to_le_bytes());
                    }
                    0b011 => {
                        // SD
                        self.memory.write_bytes(addr, self.cpu.x[rs2].to_le_bytes());
                    }
                    _ => panic!("Unimplemented instruction {instruction:b}"),
                }
            }

            // Memory ordering instructions
            0b0001111 => {
                // FENCE
                todo!("FENCE");
            }

            // Environment call and breakpoints
            0b1110011 => match instruction {
                0b00000000000000000000000001110011 => {
                    // ECALL
                    self.syscall()
                }
                0b00000000000100000000000001110011 => {
                    // EBREAK
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
