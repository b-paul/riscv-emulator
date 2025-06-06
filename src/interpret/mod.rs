#![allow(dead_code)]

use super::{instructions::Instruction, Emulator, Trap};

mod atomic;
mod base;
mod machine;
mod mul;
mod zicsr;

impl Emulator {
    pub fn execute(&mut self, instruction: Instruction, opcode: u64) {
        self.x[0] = 0;

        let trap = if self.machine_csrs.can_exec(&instruction) {
            match instruction {
                Instruction::Base(instr) => self.execute_base(instr),
                Instruction::Machine(instr) => self.execute_machine(instr),
                Instruction::Zicsr(instr) => self.execute_zicsr(instr),
                Instruction::Mul(instr) => self.execute_mul(instr),
                Instruction::Atomic(instr) => self.execute_atomic(instr),
            }
        } else {
            Err(Trap::IllegalInstruction)
        };
        if let Err(trap) = trap {
            self.set_trap(trap, opcode);
        }
    }
}
