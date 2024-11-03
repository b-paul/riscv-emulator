use crate::{instructions::machine::MachineInstruction, Emulator, Privilege, Trap};

impl Emulator {
    pub(crate) fn execute_machine(&mut self, instruction: MachineInstruction) -> Result<(), Trap> {
        if self.privilege < Privilege::Machine {
            return Err(Trap::IllegalInstruction);
        }
        match instruction {
            MachineInstruction::MRet(_) => {
                // Update the pc. MRET isn't ever compressed, so we will always subtract 4.
                self.pc = self.machine_csrs.mepc.wrapping_sub(4);
                // Set MIE to MPIE
                self.machine_csrs.mstatus =
                    (self.machine_csrs.mstatus & !0x8) | (self.machine_csrs.mstatus & !0x80) >> 4;
                // Set privilege to the value in MPP
                self.privilege = (self.machine_csrs.mstatus >> 11 & 0x3)
                    .try_into()
                    .expect("An illegal MPP value was written to mstatus.");
                // Set MPIE to 1
                self.machine_csrs.mstatus |= 0x80;
                // Set MPP to user mode
                self.machine_csrs.mstatus &= !(3 << 11);
                // If we are not in machine mode, set MPRV to 0
                if self.privilege != Privilege::Machine {
                    self.machine_csrs.mstatus &= !(1 << 17);
                }
            }
            MachineInstruction::Wfi(_) => {
                self.waiting = true;
            }
        }
        Ok(())
    }
}
