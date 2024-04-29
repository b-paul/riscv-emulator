use super::IType;

pub enum MachineInstruction {
    MRet(IType),
    Wfi(IType),
}
