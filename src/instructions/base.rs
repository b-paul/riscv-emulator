use super::{BType, IType, JType, RType, SType, UType};

pub enum Branch {
    Eq,
    Ne,
    Lt,
    Ltu,
    Ge,
    Geu,
}

pub enum Immediate64 {
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

pub enum Immediate32 {
    Add,
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
    Imm64(Immediate64, IType),
    Imm32(Immediate32, IType),
    Add(RType),
    Sub(RType),
    Sll(RType),
    Slt(RType),
    Sltu(RType),
    Xor(RType),
    Srl(RType),
    Sra(RType),
    Or(RType),
    And(RType),
    Addw(RType),
    Subw(RType),
    Sllw(RType),
    Srlw(RType),
    Sraw(RType),
    Fence(u32),
    Ecall(u32),
    Ebreak(u32),
}
