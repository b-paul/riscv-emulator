use super::{BType, IType, JType, RType, SType, UType};

pub enum Branch {
    Eq,
    Ne,
    Lt,
    Ltu,
    Ge,
    Geu,
}

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

pub enum BImmediate32 {
    Add,
    Sll,
    Srl,
    Sra,
}

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

pub enum BRegister32 {
    Add,
    Sub,
    Sll,
    Srl,
    Sra,
}

pub enum BaseInstruction {
    Lui(UType),
    Auipc(UType),
    Jal(JType),
    Jalr(IType),
    Branch(Branch, BType),
    Lb(IType),
    Lh(IType),
    Lw(IType),
    Lwu(IType),
    Ld(IType),
    Lbu(IType),
    Lhu(IType),
    Sb(SType),
    Sh(SType),
    Sw(SType),
    Sd(SType),
    Imm64(BImmediate64, IType),
    Imm32(BImmediate32, IType),
    Reg64(BRegister64, RType),
    Reg32(BRegister32, RType),
    Fence(u32),
    Ecall(u32),
    Ebreak(u32),
}
