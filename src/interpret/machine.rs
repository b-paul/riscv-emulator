use crate::{instructions::machine::MachineInstruction, Emulator, Privilege};

impl Emulator {
    pub fn execute_machine(&mut self, instruction: MachineInstruction) {
        if self.privilege < Privilege::Machine {
            // TODO i wasn't able to figure out what should happen in this case
            // from reading the specification, so I'm just going to assume
            // invalid instruction
            self.illegal_instruction();
        }
        match instruction {
            MachineInstruction::MRet(_) => {
                self.pc = self.mepc.wrapping_sub(4);
                // Set MIE to MPIE
                self.mstatus = (self.mstatus & !0x8) | (self.mstatus & !0x80) >> 4;
                // Set privilege to the value in MPP
                self.privilege = (self.mstatus >> 11 & 0x3)
                    .try_into()
                    .expect("An illegal MPP value was written to mstatus.");
                // Set MPIE to 1
                self.mstatus |= 0x80;
                // Set MPP to user mode
                self.mstatus &= !(3 << 11);
                // If we are not in machine mode, set MPRV to 0
                if self.privilege != Privilege::Machine {
                    self.mstatus &= !(1 << 17);
                }
            }
            MachineInstruction::Wfi(_) => {
                self.waiting = true;
            },
        }
    }
}
