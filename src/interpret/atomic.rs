use crate::{instructions::atomic::AtomicInstruction, Emulator};

impl Emulator {
    pub fn execute_atomic(&mut self, instruction: AtomicInstruction, _aq: bool, _rl: bool) {
        if self.misa & 1 == 0 {
            todo!("Disabled extension")
        }
        match instruction {
            AtomicInstruction::LrW(_) => todo!(),
            AtomicInstruction::ScW(_) => todo!(),
            AtomicInstruction::AmoSwapW(_) => todo!(),
            AtomicInstruction::AmoAddW(_) => todo!(),
            AtomicInstruction::AmoXorW(_) => todo!(),
            AtomicInstruction::AmoAndW(_) => todo!(),
            AtomicInstruction::AmoOrW(_) => todo!(),
            AtomicInstruction::AmoMinW(_) => todo!(),
            AtomicInstruction::AmoMaxW(_) => todo!(),
            AtomicInstruction::AmoMinuW(_) => todo!(),
            AtomicInstruction::AmoMaxuW(_) => todo!(),
            AtomicInstruction::LrD(_) => todo!(),
            AtomicInstruction::ScD(_) => todo!(),
            AtomicInstruction::AmoSwapD(_) => todo!(),
            AtomicInstruction::AmoAddD(_) => todo!(),
            AtomicInstruction::AmoXorD(_) => todo!(),
            AtomicInstruction::AmoAndD(_) => todo!(),
            AtomicInstruction::AmoOrD(_) => todo!(),
            AtomicInstruction::AmoMinD(_) => todo!(),
            AtomicInstruction::AmoMaxD(_) => todo!(),
            AtomicInstruction::AmoMinuD(_) => todo!(),
            AtomicInstruction::AmoMaxuD(_) => todo!(),
        }
    }
}
