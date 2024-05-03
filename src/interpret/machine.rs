use crate::{instructions::machine::MachineInstruction, Emulator, Privilege};

impl Emulator {
    pub fn execute_machine(&mut self, instruction: MachineInstruction) {
        if self.privilege < Privilege::Machine {
            todo!();
        }
        match instruction {
            MachineInstruction::MRet(_) => todo!(),
            MachineInstruction::Wfi(_) => todo!(),
        }
    }
}