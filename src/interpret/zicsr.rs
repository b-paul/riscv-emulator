use crate::{instructions::zicsr::ZicsrInstruction, Emulator};

impl Emulator {
    pub fn execute_zicsr(&mut self, instruction: ZicsrInstruction) {
        match instruction {
            ZicsrInstruction::Csrrw(_) => todo!(),
            ZicsrInstruction::Csrrs(_) => todo!(),
            ZicsrInstruction::Csrrc(_) => todo!(),
            ZicsrInstruction::Csrrwi(_) => todo!(),
            ZicsrInstruction::Csrrsi(_) => todo!(),
            ZicsrInstruction::Csrrci(_) => todo!(),
        }
    }
}
