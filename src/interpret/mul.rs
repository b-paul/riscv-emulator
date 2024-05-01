use crate::{instructions::mul::MulInstruction, Emulator};

impl Emulator {
    pub fn execute_mul(&mut self, instruction: MulInstruction) {
        if self.misa & 1 << 12 == 0 {
            todo!("Disabled extension")
        }
        match instruction {
            MulInstruction::Mul(_) => todo!(),
            MulInstruction::Mulw(_) => todo!(),
            MulInstruction::Mulh(_) => todo!(),
            MulInstruction::Mulhsu(_) => todo!(),
            MulInstruction::Mulhu(_) => todo!(),
            MulInstruction::Div(_) => todo!(),
            MulInstruction::Divw(_) => todo!(),
            MulInstruction::Divu(_) => todo!(),
            MulInstruction::Divuw(_) => todo!(),
            MulInstruction::Rem(_) => todo!(),
            MulInstruction::Remw(_) => todo!(),
            MulInstruction::Remu(_) => todo!(),
            MulInstruction::Remuw(_) => todo!(),
        }
    }
}
