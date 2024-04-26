use super::IType;

pub enum ZiscrInstruction {
    Csrrw(IType),
    Csrrs(IType),
    Csrrc(IType),

    Csrrwi(IType),
    Csrrsi(IType),
    Csrrci(IType),
}
