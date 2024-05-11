use super::RType;

pub enum AMem {
    LrW,
    ScW,
    LrD,
    ScD,
}

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

pub enum AOp {
    Mem(AMem),
    AmoW(AAmoW),
    AmoD(AAmoD),
}

pub struct AtomicInstruction {
    pub aq: bool,
    pub rl: bool,
    pub op: AOp,
    pub instr: RType,
}
