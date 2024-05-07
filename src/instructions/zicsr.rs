use super::IType;

#[derive(PartialEq, Eq)]
pub enum ZOp {
    Csrrw,
    Csrrs,
    Csrrc,
}

pub struct ZicsrInstruction(pub ZOp, pub bool, pub IType);
