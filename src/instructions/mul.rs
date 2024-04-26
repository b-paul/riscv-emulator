use super::{BType, IType, JType, RType, SType, UType};

pub enum MulInstruction {
    Mul(RType),
    Mulw(RType),
    Mulh(RType),
    Mulhsu(RType),
    Mulhu(RType),
    Div(RType),
    Divw(RType),
    Divu(RType),
    Divuw(RType),
    Rem(RType),
    Remw(RType),
    Remu(RType),
    Remuw(RType),
}
