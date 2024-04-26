use super::{BType, IType, JType, RType, SType, UType};

pub enum ZiscrInstruction {
    Csrrw(IType),
    Csrrs(IType),
    Csrrc(IType),

    Csrrwi(IType),
    Csrrsi(IType),
    Csrrci(IType),
}
