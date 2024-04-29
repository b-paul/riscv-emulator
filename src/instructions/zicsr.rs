use super::IType;

pub enum ZicsrInstruction {
    Csrrw(IType),
    Csrrs(IType),
    Csrrc(IType),

    Csrrwi(IType),
    Csrrsi(IType),
    Csrrci(IType),
}
