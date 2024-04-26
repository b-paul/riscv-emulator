#![allow(dead_code)]

pub mod base;
pub mod ziscr;
pub mod mul;

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

pub enum Instruction {
    Base(BaseInstruction),
    Ziscr(ZiscrInstruction),
    Mul(MulInstruction),
}
