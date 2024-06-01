#![allow(dead_code)]

use super::{instructions::Instruction, mem::AccessFault, Emulator};

mod atomic;
mod base;
mod machine;
mod mul;
mod zicsr;

impl Emulator {
    pub fn execute(&mut self, instruction: Instruction) {
        self.x[0] = 0;

        match instruction {
            Instruction::Base(instr) => self.execute_base(instr),
            Instruction::Machine(instr) => self.execute_machine(instr),
            Instruction::Zicsr(instr) => self.execute_zicsr(instr),
            Instruction::Mul(instr) => self.execute_mul(instr),
            Instruction::Atomic(instr) => match self.execute_atomic(instr) {
                Ok(_) => (),
                Err(e) => match e {
                    AccessFault::Load => {
                        self.set_mtrap(5);
                        self.mtval = 0;
                    }
                    AccessFault::Store => {
                        self.set_mtrap(7);
                        self.mtval = 0;
                    }
                },
            },
        }
    }
}
