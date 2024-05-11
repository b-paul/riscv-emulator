#![allow(dead_code)]

pub mod atomic;
pub mod base;
pub mod machine;
pub mod mul;
pub mod zicsr;

use atomic::{AAmoD, AAmoW, AMem, AOp, AtomicInstruction};
use base::{
    BImmediate32, BImmediate64, BLoad, BRegister32, BRegister64, BStore, BaseInstruction, Branch,
};
use machine::MachineInstruction;
use mul::{MReg32, MReg64, MulInstruction};
use zicsr::{ZOp, ZicsrInstruction};

pub struct RType {
    pub rd: usize,
    pub rs1: usize,
    pub rs2: usize,
}

pub struct IType {
    pub rd: usize,
    pub rs1: usize,
    pub imm: u16,
}

pub struct SType {
    pub rs1: usize,
    pub rs2: usize,
    pub imm: u16,
}

pub struct BType {
    pub rs1: usize,
    pub rs2: usize,
    pub imm: u16,
}

pub struct UType {
    pub rd: usize,
    pub imm: i32,
}

pub struct JType {
    pub rd: usize,
    pub imm: i32,
}

impl RType {
    fn new(instruction: u32) -> Self {
        RType {
            rd: (instruction >> 7 & 0x1f) as usize,
            rs1: (instruction >> 15 & 0x1f) as usize,
            rs2: (instruction >> 20 & 0x1f) as usize,
        }
    }
}

impl IType {
    fn new(instruction: u32) -> Self {
        IType {
            rd: (instruction >> 7 & 0x1f) as usize,
            rs1: (instruction >> 15 & 0x1f) as usize,
            imm: (instruction >> 20) as u16,
        }
    }
}

impl SType {
    fn new(instruction: u32) -> Self {
        SType {
            rs1: (instruction >> 15 & 0x1f) as usize,
            rs2: (instruction >> 20 & 0x1f) as usize,
            imm: (instruction >> 20 | instruction >> 7 & 0x1f) as u16,
        }
    }
}

impl BType {
    fn new(instruction: u32) -> Self {
        BType {
            rs1: (instruction >> 15 & 0x1f) as usize,
            rs2: (instruction >> 20 & 0x1f) as usize,
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
            rd: (instruction >> 7 & 0x1f) as usize,
            imm: (instruction & 0xfffff000) as i32,
        }
    }
}

impl JType {
    fn new(instruction: u32) -> Self {
        JType {
            rd: (instruction >> 7 & 0x1f) as usize,
            imm: (instruction >> 20 & 0x7fe
                | instruction >> 9 & 0x800
                | instruction & 0xff000
                | instruction >> 11 & 0x10000) as i32,
        }
    }
}

pub enum Instruction {
    Base(BaseInstruction),
    Machine(MachineInstruction),
    Zicsr(ZicsrInstruction),
    Mul(MulInstruction),
    Atomic(AtomicInstruction),
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

        use BImmediate32 as Bimm32;
        use BImmediate64 as Bimm64;
        use BRegister32 as BReg32;
        use BRegister64 as BReg64;

        Some(match opcode {
            0b0010011 => match funct3 {
                0b000 => I::Base(B::Imm64(Bimm64::Add, IType::new(instruction))),
                0b010 => I::Base(B::Imm64(Bimm64::Slt, IType::new(instruction))),
                0b011 => I::Base(B::Imm64(Bimm64::Sltu, IType::new(instruction))),
                0b100 => I::Base(B::Imm64(Bimm64::Xor, IType::new(instruction))),
                0b110 => I::Base(B::Imm64(Bimm64::Or, IType::new(instruction))),
                0b111 => I::Base(B::Imm64(Bimm64::And, IType::new(instruction))),
                0b001 => {
                    let upper = instruction >> 26 & 0x3f;
                    match upper {
                        0b000000 => I::Base(B::Imm64(Bimm64::Sll, IType::new(instruction))),
                        _ => None?,
                    }
                }
                0b101 => {
                    let upper = instruction >> 26 & 0x3f;
                    let instruction = instruction & 0x1ffffff;
                    match upper {
                        0b0000000 => I::Base(B::Imm64(Bimm64::Srl, IType::new(instruction))),
                        0b010000 => I::Base(B::Imm64(Bimm64::Sra, IType::new(instruction))),
                        _ => None?,
                    }
                }
                _ => unreachable!(),
            },

            0b0011011 => match funct3 {
                0b000 => I::Base(B::Imm32(Bimm32::Add, IType::new(instruction))),
                0b001 => {
                    let upper = instruction >> 25 & 0x7f;
                    match upper {
                        0b0000000 => I::Base(B::Imm32(Bimm32::Sll, IType::new(instruction))),
                        _ => None?,
                    }
                }
                0b101 => {
                    let upper = instruction >> 25 & 0x7f;
                    match upper {
                        0b0000000 => I::Base(B::Imm32(Bimm32::Srl, IType::new(instruction))),
                        0b0100000 => I::Base(B::Imm32(Bimm32::Sra, IType::new(instruction))),
                        _ => None?,
                    }
                }
                _ => None?,
            },

            0b0110111 => I::Base(B::Lui(UType::new(instruction))),

            0b0010111 => I::Base(B::Auipc(UType::new(instruction))),

            0b0110011 => match (funct3, funct7) {
                (0b000, 0b0000000) => I::Base(B::Reg64(BReg64::Add, RType::new(instruction))),
                (0b000, 0b0100000) => I::Base(B::Reg64(BReg64::Sub, RType::new(instruction))),
                (0b010, 0b0000000) => I::Base(B::Reg64(BReg64::Slt, RType::new(instruction))),
                (0b011, 0b0000000) => I::Base(B::Reg64(BReg64::Sltu, RType::new(instruction))),
                (0b100, 0b0000000) => I::Base(B::Reg64(BReg64::Xor, RType::new(instruction))),
                (0b110, 0b0000000) => I::Base(B::Reg64(BReg64::Or, RType::new(instruction))),
                (0b111, 0b0000000) => I::Base(B::Reg64(BReg64::And, RType::new(instruction))),
                (0b001, 0b0000000) => I::Base(B::Reg64(BReg64::Sll, RType::new(instruction))),
                (0b101, 0b0000000) => I::Base(B::Reg64(BReg64::Srl, RType::new(instruction))),
                (0b101, 0b0100000) => I::Base(B::Reg64(BReg64::Sra, RType::new(instruction))),
                (0b000, 0b0000001) => I::Mul(M::Reg64(MReg64::Mul, RType::new(instruction))),
                (0b001, 0b0000001) => I::Mul(M::Reg64(MReg64::Mulh, RType::new(instruction))),
                (0b011, 0b0000001) => I::Mul(M::Reg64(MReg64::Mulhu, RType::new(instruction))),
                (0b010, 0b0000001) => I::Mul(M::Reg64(MReg64::Mulhsu, RType::new(instruction))),
                (0b100, 0b0000001) => I::Mul(M::Reg64(MReg64::Div, RType::new(instruction))),
                (0b101, 0b0000001) => I::Mul(M::Reg64(MReg64::Divu, RType::new(instruction))),
                (0b110, 0b0000001) => I::Mul(M::Reg64(MReg64::Rem, RType::new(instruction))),
                (0b111, 0b0000001) => I::Mul(M::Reg64(MReg64::Remu, RType::new(instruction))),
                _ => None?,
            },

            0b0111011 => match (funct3, funct7) {
                (0b000, 0b0000000) => I::Base(B::Reg32(BReg32::Add, RType::new(instruction))),
                (0b000, 0b0100000) => I::Base(B::Reg32(BReg32::Add, RType::new(instruction))),
                (0b001, 0b0000000) => I::Base(B::Reg32(BReg32::Add, RType::new(instruction))),
                (0b101, 0b0000000) => I::Base(B::Reg32(BReg32::Add, RType::new(instruction))),
                (0b101, 0b0100000) => I::Base(B::Reg32(BReg32::Add, RType::new(instruction))),
                (0b000, 0b0000001) => I::Mul(M::Reg32(MReg32::Mul, RType::new(instruction))),
                (0b100, 0b0000001) => I::Mul(M::Reg32(MReg32::Div, RType::new(instruction))),
                (0b101, 0b0000001) => I::Mul(M::Reg32(MReg32::Divu, RType::new(instruction))),
                (0b110, 0b0000001) => I::Mul(M::Reg32(MReg32::Rem, RType::new(instruction))),
                (0b111, 0b0000001) => I::Mul(M::Reg32(MReg32::Remu, RType::new(instruction))),
                _ => None?,
            },

            0b1101111 => I::Base(B::Jal(JType::new(instruction))),
            0b1100111 => {
                if funct3 == 0b000 {
                    I::Base(B::Jalr(IType::new(instruction)))
                } else {
                    None?
                }
            }
            0b1100011 => match funct3 {
                0b000 => I::Base(B::Branch(Branch::Eq, BType::new(instruction))),
                0b001 => I::Base(B::Branch(Branch::Ne, BType::new(instruction))),
                0b100 => I::Base(B::Branch(Branch::Lt, BType::new(instruction))),
                0b110 => I::Base(B::Branch(Branch::Ltu, BType::new(instruction))),
                0b101 => I::Base(B::Branch(Branch::Ge, BType::new(instruction))),
                0b111 => I::Base(B::Branch(Branch::Geu, BType::new(instruction))),
                _ => None?,
            },

            0b0000011 => match funct3 {
                0b000 => I::Base(B::Load(BLoad::B, IType::new(instruction))),
                0b001 => I::Base(B::Load(BLoad::H, IType::new(instruction))),
                0b010 => I::Base(B::Load(BLoad::W, IType::new(instruction))),
                0b100 => I::Base(B::Load(BLoad::Bu, IType::new(instruction))),
                0b101 => I::Base(B::Load(BLoad::Hu, IType::new(instruction))),
                0b110 => I::Base(B::Load(BLoad::Wu, IType::new(instruction))),
                0b011 => I::Base(B::Load(BLoad::D, IType::new(instruction))),
                _ => None?,
            },
            0b0100011 => match funct3 {
                0b000 => I::Base(B::Store(BStore::B, SType::new(instruction))),
                0b001 => I::Base(B::Store(BStore::H, SType::new(instruction))),
                0b010 => I::Base(B::Store(BStore::W, SType::new(instruction))),
                0b011 => I::Base(B::Store(BStore::D, SType::new(instruction))),
                _ => None?,
            },

            0b0001111 => I::Base(B::Fence(instruction)),

            0b1110011 => {
                if funct3 == 0 {
                    match instruction {
                        0b00000000000000000000000001110011 => I::Base(B::Ecall(instruction)),
                        0b00000000000100000000000001110011 => I::Base(B::Ebreak(instruction)),
                        0b00110000001000000000000001110011 => {
                            I::Machine(MA::MRet(IType::new(instruction)))
                        }
                        0b00010000010100000000000001110011 => {
                            I::Machine(MA::Wfi(IType::new(instruction)))
                        }
                        _ => None?,
                    }
                } else {
                    match funct3 {
                        0b001 => I::Zicsr(Z(ZOp::Csrrw, false, IType::new(instruction))),
                        0b010 => I::Zicsr(Z(ZOp::Csrrs, false, IType::new(instruction))),
                        0b011 => I::Zicsr(Z(ZOp::Csrrc, false, IType::new(instruction))),
                        0b101 => I::Zicsr(Z(ZOp::Csrrw, true, IType::new(instruction))),
                        0b110 => I::Zicsr(Z(ZOp::Csrrs, true, IType::new(instruction))),
                        0b111 => I::Zicsr(Z(ZOp::Csrrc, true, IType::new(instruction))),
                        _ => None?,
                    }
                }
            }

            0b0101111 => {
                let funct5 = instruction >> 27 & 0x1f;
                let aq = instruction >> 26 & 1 != 0;
                let rl = instruction >> 25 & 1 != 0;
                let op = match (funct5, funct3) {
                    (0b00010, 0b010) => AOp::Mem(AMem::LrW),
                    (0b00010, 0b011) => AOp::Mem(AMem::LrD),
                    (0b00011, 0b010) => AOp::Mem(AMem::ScW),
                    (0b00011, 0b011) => AOp::Mem(AMem::ScD),
                    (0b00001, 0b010) => AOp::AmoW(AAmoW::Swap),
                    (0b00001, 0b011) => AOp::AmoD(AAmoD::Swap),
                    (0b00000, 0b010) => AOp::AmoW(AAmoW::Add),
                    (0b00000, 0b011) => AOp::AmoD(AAmoD::Add),
                    (0b01100, 0b010) => AOp::AmoW(AAmoW::And),
                    (0b01100, 0b011) => AOp::AmoD(AAmoD::And),
                    (0b01000, 0b010) => AOp::AmoW(AAmoW::Or),
                    (0b01000, 0b011) => AOp::AmoD(AAmoD::Or),
                    (0b00100, 0b010) => AOp::AmoW(AAmoW::Xor),
                    (0b00100, 0b011) => AOp::AmoD(AAmoD::Xor),
                    (0b10100, 0b010) => AOp::AmoW(AAmoW::Max),
                    (0b10100, 0b011) => AOp::AmoD(AAmoD::Max),
                    (0b10000, 0b010) => AOp::AmoW(AAmoW::Min),
                    (0b10000, 0b011) => AOp::AmoD(AAmoD::Min),
                    (0b11100, 0b010) => AOp::AmoW(AAmoW::Maxu),
                    (0b11100, 0b011) => AOp::AmoD(AAmoD::Maxu),
                    (0b11000, 0b010) => AOp::AmoW(AAmoW::Minu),
                    (0b11000, 0b011) => AOp::AmoD(AAmoD::Minu),
                    _ => None?,
                };
                let arg = RType::new(instruction);
                I::Atomic(AtomicInstruction {
                    instr: arg,
                    aq,
                    op,
                    rl,
                })
            }

            _ => None?,
        })
    }
}
