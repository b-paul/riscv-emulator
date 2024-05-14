use super::IType;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MachineInstruction {
    MRet(IType),
    Wfi(IType),
}
