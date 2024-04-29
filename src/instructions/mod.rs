#![allow(dead_code)]

pub mod atomic;
pub mod base;
pub mod machine;
pub mod mul;
pub mod zicsr;

use atomic::AtomicInstruction;
use base::BaseInstruction;
use machine::MachineInstruction;
use mul::MulInstruction;
use zicsr::ZicsrInstruction;

pub struct RType {
    rd: u8,
    rs1: u8,
    rs2: u8,
}

pub struct IType {
    rd: u8,
    rs1: u8,
    imm: u16,
}

pub struct SType {
    rs1: u8,
    rs2: u8,
    imm: u16,
}

pub struct BType {
    rs1: u8,
    rs2: u8,
    imm: u16,
}

pub struct UType {
    rd: u8,
    imm: u32,
}

pub struct JType {
    rd: u8,
    imm: u32,
}

impl RType {
    fn new(instruction: u32) -> Self {
        RType {
            rd: (instruction >> 7 & 0x1f) as u8,
            rs1: (instruction >> 15 & 0x1f) as u8,
            rs2: (instruction >> 20 & 0x1f) as u8,
        }
    }
}

impl IType {
    fn new(instruction: u32) -> Self {
        IType {
            rd: (instruction >> 7 & 0x1f) as u8,
            rs1: (instruction >> 15 & 0x1f) as u8,
            imm: (instruction >> 20) as u16,
        }
    }
}

impl SType {
    fn new(instruction: u32) -> Self {
        SType {
            rs1: (instruction >> 15 & 0x1f) as u8,
            rs2: (instruction >> 20 & 0x1f) as u8,
            imm: (instruction >> 20 | instruction >> 7 & 0x1f) as u16,
        }
    }
}

impl BType {
    fn new(instruction: u32) -> Self {
        BType {
            rs1: (instruction >> 15 & 0x1f) as u8,
            rs2: (instruction >> 20 & 0x1f) as u8,
            imm: (instruction >> 7 & 0x1e
                | instruction >> 20 & 0x7e0
                | instruction << 4 & 0x800
                | instruction >> 19 & 0x1000) as u16,
        }
    }
}

impl UType {
    fn new(instruction: u32) -> Self {
        UType {
            rd: (instruction >> 7 & 0x1f) as u8,
            imm: instruction & 0xfffff000,
        }
    }
}

impl JType {
    fn new(instruction: u32) -> Self {
        JType {
            rd: (instruction >> 7 & 0x1f) as u8,
            imm: instruction >> 20 & 0x7fe
                | instruction >> 9 & 0x800
                | instruction & 0xff000
                | instruction >> 11 & 0x10000,
        }
    }
}

pub enum Instruction {
    Base(BaseInstruction),
    Machine(MachineInstruction),
    Zicsr(ZicsrInstruction),
    Mul(MulInstruction),
    Atomic {
        instr: AtomicInstruction,
        aq: bool,
        rl: bool,
    },
}

impl Instruction {
    pub fn parse(instruction: u32) -> Option<Instruction> {
        if instruction & 0b11 != 0b11 {
            //todo!("16 bit instructions!")
            return None;
        }

        let opcode = instruction & 0x7f;
        let funct3 = instruction >> 12 & 0x7;
        let funct7 = instruction >> 25 & 0x7f;

        use AtomicInstruction as A;
        use BaseInstruction as B;
        use Instruction as I;
        use MachineInstruction as MA;
        use MulInstruction as M;
        use ZicsrInstruction as Z;

        match opcode {
            0b0010011 => {
                match funct3 {
                    0b000 => Some(I::Base(B::Addi(IType::new(instruction)))),
                    0b010 => Some(I::Base(B::Slti(IType::new(instruction)))),
                    0b011 => Some(I::Base(B::Sltiu(IType::new(instruction)))),
                    0b100 => Some(I::Base(B::Xori(IType::new(instruction)))),
                    0b110 => Some(I::Base(B::Ori(IType::new(instruction)))),
                    0b111 => Some(I::Base(B::Andi(IType::new(instruction)))),
                    0b001 => {
                        let upper = instruction >> 26 & 0x3f;
                        match upper {
                            0b000000 => Some(I::Base(B::Slli(IType::new(instruction)))),
                            _ => None,
                        }
                    }
                    0b101 => {
                        let upper = instruction >> 26 & 0x3f;
                        match upper {
                            0b0000000 => Some(I::Base(B::Srli(IType::new(instruction)))),
                            0b010000 => Some(I::Base(B::Srai(IType::new(instruction & 0x1ffffff)))),
                            _ => None,
                        }
                    }
                    _ => unreachable!(),
                }
            }

            0b0011011 => {
                match funct3 {
                    0b000 => Some(I::Base(B::Addiw(IType::new(instruction)))),
                    0b001 => {
                        let upper = instruction >> 25 & 0x7f;
                        match upper {
                            0b0000000 => Some(I::Base(B::Slliw(IType::new(instruction)))),
                            _ => None,
                        }
                    }
                    0b101 => {
                        let upper = instruction >> 25 & 0x7f;
                        match upper {
                            0b0000000 => Some(I::Base(B::Srliw(IType::new(instruction)))),
                            0b0100000 => {
                                Some(I::Base(B::Sraiw(IType::new(instruction & 0x1ffffff))))
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }

            0b0110111 => Some(I::Base(B::Lui(UType::new(instruction)))),

            0b0010111 => Some(I::Base(B::Auipc(UType::new(instruction)))),

            0b0110011 => {
                match (funct3, funct7) {
                    (0b000, 0b0000000) => Some(I::Base(B::Add(RType::new(instruction)))),
                    (0b000, 0b0100000) => Some(I::Base(B::Sub(RType::new(instruction)))),
                    (0b010, 0b0000000) => Some(I::Base(B::Slt(RType::new(instruction)))),
                    (0b011, 0b0000000) => Some(I::Base(B::Sltu(RType::new(instruction)))),
                    (0b100, 0b0000000) => Some(I::Base(B::Xor(RType::new(instruction)))),
                    (0b110, 0b0000000) => Some(I::Base(B::Or(RType::new(instruction)))),
                    (0b111, 0b0000000) => Some(I::Base(B::And(RType::new(instruction)))),
                    (0b001, 0b0000000) => Some(I::Base(B::Sll(RType::new(instruction)))),
                    (0b101, 0b0000000) => Some(I::Base(B::Srl(RType::new(instruction)))),
                    (0b101, 0b0100000) => Some(I::Base(B::Sra(RType::new(instruction)))),
                    (0b000, 0b0000001) => Some(I::Mul(M::Mul(RType::new(instruction)))),
                    (0b001, 0b0000001) => Some(I::Mul(M::Mulh(RType::new(instruction)))),
                    (0b011, 0b0000001) => Some(I::Mul(M::Mulhu(RType::new(instruction)))),
                    (0b010, 0b0000001) => Some(I::Mul(M::Mulhsu(RType::new(instruction)))),
                    (0b100, 0b0000001) => Some(I::Mul(M::Div(RType::new(instruction)))),
                    (0b101, 0b0000001) => Some(I::Mul(M::Divu(RType::new(instruction)))),
                    (0b110, 0b0000001) => Some(I::Mul(M::Rem(RType::new(instruction)))),
                    (0b111, 0b0000001) => Some(I::Mul(M::Remu(RType::new(instruction)))),
                    _ => None,
                }
            }

            0b0111011 => {
                match (funct3, funct7) {
                    (0b000, 0b0000000) => Some(I::Base(B::Addw(RType::new(instruction)))),
                    (0b000, 0b0100000) => Some(I::Base(B::Subw(RType::new(instruction)))),
                    (0b001, 0b0000000) => Some(I::Base(B::Sllw(RType::new(instruction)))),
                    (0b101, 0b0000000) => Some(I::Base(B::Srlw(RType::new(instruction)))),
                    (0b101, 0b0100000) => Some(I::Base(B::Sraw(RType::new(instruction)))),
                    (0b000, 0b0000001) => Some(I::Mul(M::Mulw(RType::new(instruction)))),
                    (0b100, 0b0000001) => Some(I::Mul(M::Divw(RType::new(instruction)))),
                    (0b101, 0b0000001) => Some(I::Mul(M::Divuw(RType::new(instruction)))),
                    (0b110, 0b0000001) => Some(I::Mul(M::Remw(RType::new(instruction)))),
                    (0b111, 0b0000001) => Some(I::Mul(M::Remuw(RType::new(instruction)))),
                    _ => None,
                }
            }

            0b1101111 => Some(I::Base(B::Jal(JType::new(instruction)))),
            0b1100111 => {
                if funct3 == 0b000 {
                    Some(I::Base(B::Jalr(IType::new(instruction))))
                } else {
                    None
                }
            }
            0b1100011 => {
                match funct3 {
                    0b000 => Some(I::Base(B::Beq(BType::new(instruction)))),
                    0b001 => Some(I::Base(B::Bne(BType::new(instruction)))),
                    0b100 => Some(I::Base(B::Blt(BType::new(instruction)))),
                    0b110 => Some(I::Base(B::Bltu(BType::new(instruction)))),
                    0b101 => Some(I::Base(B::Bge(BType::new(instruction)))),
                    0b111 => Some(I::Base(B::Bgeu(BType::new(instruction)))),
                    _ => None,
                }
            }

            0b0000011 => {
                match funct3 {
                    0b000 => Some(I::Base(B::Lb(IType::new(instruction)))),
                    0b001 => Some(I::Base(B::Lh(IType::new(instruction)))),
                    0b010 => Some(I::Base(B::Lw(IType::new(instruction)))),
                    0b100 => Some(I::Base(B::Lbu(IType::new(instruction)))),
                    0b101 => Some(I::Base(B::Lhu(IType::new(instruction)))),
                    0b110 => Some(I::Base(B::Lwu(IType::new(instruction)))),
                    0b011 => Some(I::Base(B::Ld(IType::new(instruction)))),
                    _ => None,
                }
            }
            0b0100011 => {
                match funct3 {
                    0b000 => Some(I::Base(B::Sb(SType::new(instruction)))),
                    0b001 => Some(I::Base(B::Sh(SType::new(instruction)))),
                    0b010 => Some(I::Base(B::Sw(SType::new(instruction)))),
                    0b011 => Some(I::Base(B::Sd(SType::new(instruction)))),
                    _ => None,
                }
            }

            0b0001111 => {
                Some(I::Base(B::Fence(instruction)))
            }

            0b1110011 => {
                if funct3 == 0 {
                    match instruction {
                        0b00000000000000000000000001110011 => Some(I::Base(B::Ecall(instruction))),
                        0b00000000000100000000000001110011 => Some(I::Base(B::Ebreak(instruction))),
                        0b00110000001000000000000001110011 => {
                            Some(I::Machine(MA::MRet(IType::new(instruction))))
                        }
                        0b00010000010100000000000001110011 => {
                            Some(I::Machine(MA::Wfi(IType::new(instruction))))
                        }
                        _ => None,
                    }
                } else {
                    match funct3 {
                        0b001 => Some(I::Zicsr(Z::Csrrw(IType::new(instruction)))),
                        0b010 => Some(I::Zicsr(Z::Csrrw(IType::new(instruction)))),
                        0b011 => Some(I::Zicsr(Z::Csrrw(IType::new(instruction)))),
                        0b101 => Some(I::Zicsr(Z::Csrrw(IType::new(instruction)))),
                        0b110 => Some(I::Zicsr(Z::Csrrw(IType::new(instruction)))),
                        0b111 => Some(I::Zicsr(Z::Csrrw(IType::new(instruction)))),
                        _ => None,
                    }
                }
            }

            0b0101111 => {
                let funct5 = instruction >> 27 & 0x1f;
                let aq = instruction >> 26 & 1 != 0;
                let rl = instruction >> 25 & 1 != 0;
                let instrucion = match (funct5, funct3) {
                    (0b00010, 0b010) => Some(A::LrW(RType::new(instruction))),
                    (0b00010, 0b011) => Some(A::LrD(RType::new(instruction))),
                    (0b00011, 0b010) => Some(A::ScW(RType::new(instruction))),
                    (0b00011, 0b011) => Some(A::ScD(RType::new(instruction))),
                    (0b00001, 0b010) => Some(A::AmoSwapW(RType::new(instruction))),
                    (0b00001, 0b011) => Some(A::AmoSwapD(RType::new(instruction))),
                    (0b00000, 0b010) => Some(A::AmoAddW(RType::new(instruction))),
                    (0b00000, 0b011) => Some(A::AmoAddD(RType::new(instruction))),
                    (0b01100, 0b010) => Some(A::AmoAndW(RType::new(instruction))),
                    (0b01100, 0b011) => Some(A::AmoAndW(RType::new(instruction))),
                    (0b01000, 0b010) => Some(A::AmoOrW(RType::new(instruction))),
                    (0b01000, 0b011) => Some(A::AmoOrD(RType::new(instruction))),
                    (0b00100, 0b010) => Some(A::AmoXorW(RType::new(instruction))),
                    (0b00100, 0b011) => Some(A::AmoXorD(RType::new(instruction))),
                    (0b10100, 0b010) => Some(A::AmoMaxD(RType::new(instruction))),
                    (0b10100, 0b011) => Some(A::AmoMaxD(RType::new(instruction))),
                    (0b10000, 0b010) => Some(A::AmoMinW(RType::new(instruction))),
                    (0b10000, 0b011) => Some(A::AmoMinD(RType::new(instruction))),
                    (0b11100, 0b010) => Some(A::AmoMaxuW(RType::new(instruction))),
                    (0b11100, 0b011) => Some(A::AmoMaxuD(RType::new(instruction))),
                    (0b11000, 0b010) => Some(A::AmoMinuW(RType::new(instruction))),
                    (0b11000, 0b011) => Some(A::AmoMinuD(RType::new(instruction))),
                    _ => None,
                };
                instrucion.map(|instr| I::Atomic { instr, aq, rl })
            }

            _ => None,
        }
    }
}
