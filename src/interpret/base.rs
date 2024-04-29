use crate::{Emulator, instructions::base::BaseInstruction};

impl Emulator {
    pub fn execute_base(&mut self, instruction: BaseInstruction) {
        if self.misa & 1 << 8 != 0 {
            todo!("Disabled extension")
        }
        match instruction {
            BaseInstruction::Lui(_) => todo!(),
            BaseInstruction::Auipc(_) => todo!(),
            BaseInstruction::Jal(_) => todo!(),
            BaseInstruction::Jalr(_) => todo!(),
            BaseInstruction::Beq(_) => todo!(),
            BaseInstruction::Bne(_) => todo!(),
            BaseInstruction::Blt(_) => todo!(),
            BaseInstruction::Bltu(_) => todo!(),
            BaseInstruction::Bge(_) => todo!(),
            BaseInstruction::Bgeu(_) => todo!(),
            BaseInstruction::Lb(_) => todo!(),
            BaseInstruction::Lh(_) => todo!(),
            BaseInstruction::Lw(_) => todo!(),
            BaseInstruction::Lwu(_) => todo!(),
            BaseInstruction::Ld(_) => todo!(),
            BaseInstruction::Lbu(_) => todo!(),
            BaseInstruction::Lhu(_) => todo!(),
            BaseInstruction::Sb(_) => todo!(),
            BaseInstruction::Sh(_) => todo!(),
            BaseInstruction::Sw(_) => todo!(),
            BaseInstruction::Sd(_) => todo!(),
            BaseInstruction::Addi(_) => todo!(),
            BaseInstruction::Slti(_) => todo!(),
            BaseInstruction::Sltiu(_) => todo!(),
            BaseInstruction::Xori(_) => todo!(),
            BaseInstruction::Ori(_) => todo!(),
            BaseInstruction::Andi(_) => todo!(),
            BaseInstruction::Slli(_) => todo!(),
            BaseInstruction::Srli(_) => todo!(),
            BaseInstruction::Srai(_) => todo!(),
            BaseInstruction::Add(_) => todo!(),
            BaseInstruction::Sub(_) => todo!(),
            BaseInstruction::Sll(_) => todo!(),
            BaseInstruction::Slt(_) => todo!(),
            BaseInstruction::Sltu(_) => todo!(),
            BaseInstruction::Xor(_) => todo!(),
            BaseInstruction::Srl(_) => todo!(),
            BaseInstruction::Sra(_) => todo!(),
            BaseInstruction::Or(_) => todo!(),
            BaseInstruction::And(_) => todo!(),
            BaseInstruction::Addiw(_) => todo!(),
            BaseInstruction::Slliw(_) => todo!(),
            BaseInstruction::Srliw(_) => todo!(),
            BaseInstruction::Sraiw(_) => todo!(),
            BaseInstruction::Addw(_) => todo!(),
            BaseInstruction::Subw(_) => todo!(),
            BaseInstruction::Sllw(_) => todo!(),
            BaseInstruction::Srlw(_) => todo!(),
            BaseInstruction::Sraw(_) => todo!(),
            BaseInstruction::Fence(_) => todo!(),
            BaseInstruction::Ecall(_) => todo!(),
            BaseInstruction::Ebreak(_) => todo!(),
        }
    }
}
