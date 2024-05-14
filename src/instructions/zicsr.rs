use super::IType;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ZOp {
    Csrrw,
    Csrrs,
    Csrrc,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZicsrInstruction(pub ZOp, pub bool, pub IType);
