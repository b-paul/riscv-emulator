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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RType {
    pub rd: usize,
    pub rs1: usize,
    pub rs2: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IType {
    pub rd: usize,
    pub rs1: usize,
    pub imm: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SType {
    pub rs1: usize,
    pub rs2: usize,
    pub imm: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BType {
    pub rs1: usize,
    pub rs2: usize,
    pub imm: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UType {
    pub rd: usize,
    pub imm: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

    fn cr(instruction: u16) -> Self {
        RType {
            rd: (instruction >> 7 & 0x07) as usize + 8,
            rs1: (instruction >> 7 & 0x07) as usize + 8,
            rs2: (instruction >> 2 & 0x07) as usize + 8,
        }
    }

    fn ca(instruction: u16) -> Self {
        RType {
            rd: (instruction >> 7 & 7) as usize + 8,
            rs1: (instruction >> 7 & 7) as usize + 8,
            rs2: (instruction >> 2 & 7) as usize + 8,
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

    fn ci(instruction: u16, imm: u16) -> Self {
        IType {
            rd: (instruction >> 7 & 0x1f) as usize,
            rs1: (instruction >> 7 & 0x1f) as usize,
            imm,
        }
    }

    fn cis(instruction: u16, imm: u16) -> Self {
        IType {
            rd: (instruction >> 7 & 0x1f) as usize,
            rs1: 2,
            imm,
        }
    }

    fn ciw(instruction: u16, imm: u16) -> Self {
        IType {
            rd: (instruction >> 2 & 7) as usize + 8,
            rs1: 2,
            imm,
        }
    }

    fn cl(instruction: u16, imm: u16) -> Self {
        IType {
            rd: (instruction >> 2 & 7) as usize + 8,
            rs1: (instruction >> 7 & 7) as usize + 8,
            imm,
        }
    }

    fn cb(instruction: u16) -> Self {
        IType {
            rd: (instruction >> 7 & 7) as usize + 8,
            rs1: (instruction >> 7 & 7) as usize + 8,
            imm: (instruction >> 2 & 0x1f | instruction >> 7 & 0x20),
        }
    }
}

impl SType {
    fn new(instruction: u32) -> Self {
        SType {
            rs1: (instruction >> 15 & 0x1f) as usize,
            rs2: (instruction >> 20 & 0x1f) as usize,
            imm: (instruction >> 20 & 0xfe0 | instruction >> 7 & 0x1f) as u16,
        }
    }

    fn css(instruction: u16, imm: u16) -> Self {
        SType {
            rs1: 2,
            rs2: (instruction >> 2 & 0x1f) as usize,
            imm,
        }
    }

    fn cs(instruction: u16, imm: u16) -> Self {
        SType {
            rs1: (instruction >> 7 & 7) as usize + 8,
            rs2: (instruction >> 2 & 7) as usize + 8,
            imm,
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

    fn cb(instruction: u16) -> Self {
        BType {
            rs1: (instruction >> 7 & 7) as usize + 8,
            rs2: 0,
            imm: (instruction >> 2 & 0x6
                | instruction >> 7 & 0x18
                | instruction << 3 & 0x20
                | instruction << 1 & 0xc0
                | instruction >> 4 & 0x100),
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

    fn ci(instruction: u16, imm: i32) -> Self {
        UType {
            rd: (instruction >> 7 & 0x1f) as usize,
            imm,
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
                | instruction >> 11 & 0x100000) as i32,
        }
    }

    fn cj(instruction: u16, rd: usize) -> Self {
        JType {
            rd,
            imm: (instruction >> 2 & 0x6
                | instruction >> 7 & 0x10
                | instruction << 3 & 0x20
                | instruction >> 1 & 0x40
                | instruction << 1 & 0x80
                | instruction >> 1 & 0x300
                | instruction << 2 & 0x400
                | instruction >> 1 & 0x800) as i16 as i32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Instruction {
    Base(BaseInstruction),
    Machine(MachineInstruction),
    Zicsr(ZicsrInstruction),
    Mul(MulInstruction),
    Atomic(AtomicInstruction),
}

impl Instruction {
    pub fn parse_compressed(instruction: u16) -> Option<Instruction> {
        let opcode = instruction & 0x3;
        let funct3 = (instruction >> 13) & 0x7;

        if instruction == 0 {
            return None;
        }

        match opcode {
            0b00 => {
                match funct3 {
                    0b010 => Some(Instruction::Base(BaseInstruction::Load(
                        BLoad::W,
                        IType::cl(
                            instruction,
                            instruction >> 4 & 0x4
                                | instruction >> 7 & 0x38
                                | instruction << 1 & 0x40,
                        ),
                    ))),
                    0b011 => Some(Instruction::Base(BaseInstruction::Load(
                        BLoad::D,
                        IType::cl(
                            instruction,
                            instruction >> 7 & 0x38 | instruction << 1 & 0xc0,
                        ),
                    ))),
                    0b110 => Some(Instruction::Base(BaseInstruction::Store(
                        BStore::W,
                        SType::cs(
                            instruction,
                            instruction >> 4 & 0x4
                                | instruction >> 7 & 0x38
                                | instruction << 1 & 0x40,
                        ),
                    ))),
                    0b111 => Some(Instruction::Base(BaseInstruction::Store(
                        BStore::D,
                        SType::cs(
                            instruction,
                            instruction >> 7 & 0x38 | instruction << 1 & 0xc0,
                        ),
                    ))),
                    // C.ADDI4SPN
                    0b000 => Some(Instruction::Base(BaseInstruction::Imm64(
                        BImmediate64::Add,
                        IType::ciw(
                            instruction,
                            instruction >> 4 & 0x4
                                | instruction >> 2 & 0x8
                                | instruction >> 7 & 0x30
                                | instruction >> 1 & 0x3c0,
                        ),
                    ))),
                    _ => None,
                }
            }

            0b01 => {
                match funct3 {
                    // C.J
                    0b101 => Some(Instruction::Base(BaseInstruction::Jal(
                        JType::cj(instruction, 0),
                        true,
                    ))),
                    0b110 => Some(Instruction::Base(BaseInstruction::Branch(
                        Branch::Eq,
                        BType::cb(instruction),
                        true,
                    ))),
                    0b111 => Some(Instruction::Base(BaseInstruction::Branch(
                        Branch::Ne,
                        BType::cb(instruction),
                        true,
                    ))),
                    // C.LI
                    0b010 => {
                        let mut instr = IType::ci(
                            instruction,
                            (((instruction >> 2 & 0x1f | instruction >> 7 & 0x20) as i16) << 10
                                >> 10) as u16,
                        );
                        instr.rs1 = 0; // yuck
                        Some(Instruction::Base(BaseInstruction::Imm64(
                            BImmediate64::Add,
                            instr,
                        )))
                    }
                    0b011 => {
                        let rd = (instruction >> 7 & 0x1f) as usize;
                        // C.ADDI16SP
                        if rd == 2 {
                            // We know that rd == 2, so we can just parse the instruction like
                            // normal
                            Some(Instruction::Base(BaseInstruction::Imm64(
                                BImmediate64::Add,
                                IType::ci(
                                    instruction,
                                    (((instruction >> 2 & 0x10
                                        | instruction << 3 & 0x20
                                        | instruction << 1 & 0x40
                                        | instruction << 4 & 0x180
                                        | instruction >> 3 & 0x200)
                                        as i16)
                                        << 6
                                        >> 6) as u16,
                                ),
                            )))
                        } else {
                            // C.LUI
                            Some(Instruction::Base(BaseInstruction::Lui(UType::ci(
                                instruction,
                                ((instruction as i32) << 10 & 0x1f000
                                    | (instruction as i32) << 5 & 0x20000)
                                    << 14
                                    >> 14,
                            ))))
                        }
                    }
                    // C.ADDI
                    0b000 => Some(Instruction::Base(BaseInstruction::Imm64(
                        BImmediate64::Add,
                        IType::ci(
                            instruction,
                            (((instruction >> 2 & 0x1f | instruction >> 7 & 0x20) as i16) << 10
                                >> 10) as u16,
                        ),
                    ))),
                    // C.ADDIW
                    0b001 => Some(Instruction::Base(BaseInstruction::Imm32(
                        BImmediate32::Add,
                        IType::ci(
                            instruction,
                            (((instruction >> 2 & 0x1f | instruction >> 7 & 0x20) as i16) << 10
                                >> 10) as u16,
                        ),
                    ))),
                    0b100 => {
                        let funct2 = instruction >> 10 & 0x3;
                        match funct2 {
                            0b00 => Some(Instruction::Base(BaseInstruction::Imm64(
                                BImmediate64::Srl,
                                IType::cb(instruction),
                            ))),
                            0b01 => Some(Instruction::Base(BaseInstruction::Imm64(
                                BImmediate64::Sra,
                                IType::cb(instruction),
                            ))),
                            0b10 => {
                                let mut instr = IType::cb(instruction);
                                instr.imm = ((instr.imm as i16) << 10 >> 10) as u16;
                                Some(Instruction::Base(BaseInstruction::Imm64(
                                    BImmediate64::And,
                                    instr,
                                )))
                            }
                            0b11 => {
                                let funct3 = instruction >> 5 & 0x3 | instruction >> 10 & 0x4;
                                match funct3 {
                                    0b000 => Some(Instruction::Base(BaseInstruction::Reg64(
                                        BRegister64::Sub,
                                        RType::cr(instruction),
                                    ))),
                                    0b001 => Some(Instruction::Base(BaseInstruction::Reg64(
                                        BRegister64::Xor,
                                        RType::cr(instruction),
                                    ))),
                                    0b010 => Some(Instruction::Base(BaseInstruction::Reg64(
                                        BRegister64::Or,
                                        RType::cr(instruction),
                                    ))),
                                    0b011 => Some(Instruction::Base(BaseInstruction::Reg64(
                                        BRegister64::And,
                                        RType::cr(instruction),
                                    ))),
                                    0b101 => Some(Instruction::Base(BaseInstruction::Reg32(
                                        BRegister32::Add,
                                        RType::cr(instruction),
                                    ))),
                                    0b100 => Some(Instruction::Base(BaseInstruction::Reg32(
                                        BRegister32::Sub,
                                        RType::cr(instruction),
                                    ))),
                                    _ => None,
                                }
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }

            0b10 => {
                match funct3 {
                    // C.LWSP
                    0b010 => Some(Instruction::Base(BaseInstruction::Load(
                        BLoad::W,
                        IType::cis(
                            instruction,
                            instruction >> 2 & 0x1c
                                | instruction >> 7 & 0x20
                                | instruction << 4 & 0xc0,
                        ),
                    ))),
                    // C.LDSP
                    0b011 => Some(Instruction::Base(BaseInstruction::Load(
                        BLoad::D,
                        IType::cis(
                            instruction,
                            instruction >> 2 & 0x18
                                | instruction >> 7 & 0x20
                                | instruction << 4 & 0x1c0,
                        ),
                    ))),
                    // C.SWSP
                    0b110 => Some(Instruction::Base(BaseInstruction::Store(
                        BStore::W,
                        SType::css(
                            instruction,
                            instruction >> 7 & 0x3c | instruction >> 1 & 0xc0,
                        ),
                    ))),
                    // C.SDSP
                    0b111 => Some(Instruction::Base(BaseInstruction::Store(
                        BStore::D,
                        SType::css(
                            instruction,
                            instruction >> 7 & 0x38 | instruction >> 1 & 0x1c0,
                        ),
                    ))),
                    0b100 => {
                        let rs1 = (instruction >> 7 & 0x1f) as usize;
                        let rs2 = (instruction >> 2 & 0x1f) as usize;
                        let funct4 = (instruction >> 12 & 0x1) == 1;
                        if rs1 == 0 {
                            None
                        } else {
                            match (rs1 == 0, rs2 == 0, funct4) {
                                // C.JR
                                (false, true, false) => Some(Instruction::Base(
                                    BaseInstruction::Jalr(IType { rd: 0, rs1, imm: 0 }, true),
                                )),
                                // C.JALR
                                (false, true, true) => Some(Instruction::Base(
                                    BaseInstruction::Jalr(IType { rd: 1, rs1, imm: 0 }, true),
                                )),
                                // C.MV
                                (false, false, false) => {
                                    Some(Instruction::Base(BaseInstruction::Reg64(
                                        BRegister64::Add,
                                        RType {
                                            rd: rs1,
                                            rs1: 0,
                                            rs2,
                                        },
                                    )))
                                }
                                // C.ADD
                                (false, false, true) => {
                                    Some(Instruction::Base(BaseInstruction::Reg64(
                                        BRegister64::Add,
                                        RType { rd: rs1, rs1, rs2 },
                                    )))
                                }
                                (true, true, true) => {
                                    Some(Instruction::Base(BaseInstruction::Ecall))
                                }
                                _ => None,
                            }
                        }
                    }
                    0b000 => Some(Instruction::Base(BaseInstruction::Imm64(
                        BImmediate64::Sll,
                        IType::ci(
                            instruction,
                            instruction >> 2 & 0x1f | instruction >> 7 & 0x20,
                        ),
                    ))),
                    _ => None,
                }
            }

            _ => None,
        }
    }

    pub fn parse(instruction: u32) -> Option<Instruction> {
        let opcode = instruction & 0x7f;
        let funct3 = instruction >> 12 & 0x7;
        let funct7 = instruction >> 25 & 0x7f;

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
                    let instruction = instruction & 0x3ffffff;
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
                (0b000, 0b0100000) => I::Base(B::Reg32(BReg32::Sub, RType::new(instruction))),
                (0b001, 0b0000000) => I::Base(B::Reg32(BReg32::Sll, RType::new(instruction))),
                (0b101, 0b0000000) => I::Base(B::Reg32(BReg32::Srl, RType::new(instruction))),
                (0b101, 0b0100000) => I::Base(B::Reg32(BReg32::Sra, RType::new(instruction))),
                (0b000, 0b0000001) => I::Mul(M::Reg32(MReg32::Mul, RType::new(instruction))),
                (0b100, 0b0000001) => I::Mul(M::Reg32(MReg32::Div, RType::new(instruction))),
                (0b101, 0b0000001) => I::Mul(M::Reg32(MReg32::Divu, RType::new(instruction))),
                (0b110, 0b0000001) => I::Mul(M::Reg32(MReg32::Rem, RType::new(instruction))),
                (0b111, 0b0000001) => I::Mul(M::Reg32(MReg32::Remu, RType::new(instruction))),
                _ => None?,
            },

            0b1101111 => I::Base(B::Jal(JType::new(instruction), false)),
            0b1100111 => {
                if funct3 == 0b000 {
                    I::Base(B::Jalr(IType::new(instruction), false))
                } else {
                    None?
                }
            }
            0b1100011 => match funct3 {
                0b000 => I::Base(B::Branch(Branch::Eq, BType::new(instruction), false)),
                0b001 => I::Base(B::Branch(Branch::Ne, BType::new(instruction), false)),
                0b100 => I::Base(B::Branch(Branch::Lt, BType::new(instruction), false)),
                0b110 => I::Base(B::Branch(Branch::Ltu, BType::new(instruction), false)),
                0b101 => I::Base(B::Branch(Branch::Ge, BType::new(instruction), false)),
                0b111 => I::Base(B::Branch(Branch::Geu, BType::new(instruction), false)),
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
                        0b00000000000000000000000001110011 => I::Base(B::Ecall),
                        0b00000000000100000000000001110011 => I::Base(B::Ebreak),
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
