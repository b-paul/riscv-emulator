use crate::{
    instructions::zicsr::{ZOp, ZicsrInstruction},
    Emulator,
};

impl Emulator {
    pub fn execute_zicsr(&mut self, ZicsrInstruction(op, is_imm, i): ZicsrInstruction) {
        let val = if is_imm { i.rs1 as u64 } else { self.x[i.rs1] };
        let write = op == ZOp::Csrrw || i.rs1 != 0;
        let read = !(op == ZOp::Csrrw && i.rs1 == 0);

        if let Some(csr) = self.get_csr(i.imm as u32, read) {
            let val = match op {
                ZOp::Csrrw => val,
                ZOp::Csrrs => csr | val,
                ZOp::Csrrc => csr & !val,
            };
            // This throws illegal_instruction when needed
            self.set_csr(i.imm as u32, val, write);
        } else {
            self.illegal_instruction();
        }
    }
}
