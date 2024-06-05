use crate::{instructions::machine::MachineInstruction, Emulator, Privilege, Trap};

impl Emulator {
    pub fn execute_machine(&mut self, instruction: MachineInstruction) -> Result<(), Trap> {
        if self.privilege < Privilege::Machine {
            return Err(Trap::IllegalInstruction);
        }
        match instruction {
            MachineInstruction::MRet(_) => {
                // Update the pc. MRET isn't ever compressed, so we will always subtract 4.
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
            }
        }
        Ok(())
    }
}
