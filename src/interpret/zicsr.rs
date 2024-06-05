use crate::{
    instructions::zicsr::{ZOp, ZicsrInstruction},
    Emulator, Trap,
};

impl Emulator {
    pub fn execute_zicsr(
        &mut self,
        ZicsrInstruction(op, is_imm, i): ZicsrInstruction,
    ) -> Result<(), Trap> {
        let val = if is_imm { i.rs1 as u64 } else { self.x[i.rs1] };
        let write = op == ZOp::Csrrw || i.rs1 != 0;
        let read = !(op == ZOp::Csrrw && i.rs1 == 0);

        if let Some(csr_val) = self.get_csr(i.imm as u32, read) {
            let val = match op {
                ZOp::Csrrw => val,
                ZOp::Csrrs => csr_val | val,
                ZOp::Csrrc => csr_val & !val,
            };
            if self.set_csr(i.imm as u32, val, write) {
                self.x[i.rd] = csr_val;
            } else {
                return Err(Trap::IllegalInstruction);
            }
        } else {
            return Err(Trap::IllegalInstruction);
        }
        Ok(())
    }
}
