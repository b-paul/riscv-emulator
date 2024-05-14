use super::RType;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MReg64 {
    Mul,
    Mulh,
    Mulhsu,
    Mulhu,
    Div,
    Divu,
    Rem,
    Remu,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MReg32 {
    Mul,
    Div,
    Divu,
    Rem,
    Remu,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MulInstruction {
    Reg64(MReg64, RType),
    Reg32(MReg32, RType),
}
