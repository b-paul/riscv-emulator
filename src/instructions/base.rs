use super::{BType, IType, JType, RType, SType, UType};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Branch {
    Eq,
    Ne,
    Lt,
    Ltu,
    Ge,
    Geu,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BLoad {
    B,
    H,
    W,
    Wu,
    D,
    Bu,
    Hu,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BStore {
    B,
    H,
    W,
    D,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BImmediate64 {
    Add,
    Slt,
    Sltu,
    Xor,
    Or,
    And,
    Sll,
    Srl,
    Sra,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BImmediate32 {
    Add,
    Sll,
    Srl,
    Sra,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BRegister64 {
    Add,
    Sub,
    Slt,
    Sltu,
    Xor,
    Or,
    And,
    Sll,
    Srl,
    Sra,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BRegister32 {
    Add,
    Sub,
    Sll,
    Srl,
    Sra,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BaseInstruction {
    Lui(UType),
    Auipc(UType),
    Jal(JType),
    Jalr(IType),
    Branch(Branch, BType),
    Load(BLoad, IType),
    Store(BStore, SType),
    Imm64(BImmediate64, IType),
    Imm32(BImmediate32, IType),
    Reg64(BRegister64, RType),
    Reg32(BRegister32, RType),
    Fence(u32),
    Ecall,
    Ebreak,
}
