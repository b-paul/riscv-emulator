use super::RType;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AMem {
    LrW,
    ScW,
    LrD,
    ScD,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AAmoW {
    Swap,
    Add,
    Xor,
    And,
    Or,
    Min,
    Max,
    Minu,
    Maxu,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AAmoD {
    Swap,
    Add,
    Xor,
    And,
    Or,
    Min,
    Max,
    Minu,
    Maxu,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AOp {
    Mem(AMem),
    AmoW(AAmoW),
    AmoD(AAmoD),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AtomicInstruction {
    pub aq: bool,
    pub rl: bool,
    pub op: AOp,
    pub instr: RType,
}
