#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Trap {
    InstrAddrMisaligned,
    InstrAccessFault,
    IllegalInstruction,
    Breakpoint,
    LoadAccessFault,
    StoreAccessFault,
    ECallU,
    ECallM,
}

impl Trap {
    pub fn to_code(self) -> u64 {
        match self {
            Trap::InstrAddrMisaligned => 0,
            Trap::InstrAccessFault => 1,
            Trap::IllegalInstruction => 2,
            Trap::Breakpoint => 3,
            Trap::LoadAccessFault => 5,
            Trap::StoreAccessFault => 7,
            Trap::ECallU => 8,
            Trap::ECallM => 11,
        }
    }
}
