use crate::{
    instructions::base::{
        BImmediate32, BImmediate64, BLoad, BRegister32, BRegister64, BStore, BaseInstruction,
        Branch,
    },
    Emulator, Privilege,
};

impl Emulator {
    pub fn execute_base(&mut self, instruction: BaseInstruction) {
        if self.misa & 1 << 8 == 0 {
            self.illegal_instruction();
            return;
        }
        match instruction {
            BaseInstruction::Lui(i) => self.x[i.rd] = i.imm as i64 as u64,
            BaseInstruction::Auipc(i) => self.x[i.rd] = self.pc.wrapping_add(i.imm as i64 as u64),
            BaseInstruction::Jal(i) => {
                let offset = (i.imm << 12 >> 11) as i64;
                self.x[i.rd] = self.pc.wrapping_add(4);
                self.pc = self.pc.wrapping_add(offset as u64).wrapping_sub(4);
            }
            BaseInstruction::Jalr(i) => {
                let offset = ((i.imm as i32) << 20 >> 20) as i64 as u64;
                let tmp = self.x[i.rs1];
                self.x[i.rd] = self.pc.wrapping_add(4);
                self.pc = tmp.wrapping_add(offset).wrapping_sub(4);
            }
            BaseInstruction::Branch(branch, i) => {
                let offset = ((i.imm as i32) << 19 >> 19) as u64;
                let taken = match branch {
                    Branch::Eq => self.x[i.rs1] == self.x[i.rs2],
                    Branch::Ne => self.x[i.rs1] != self.x[i.rs2],
                    Branch::Lt => (self.x[i.rs1] as i64) < (self.x[i.rs2] as i64),
                    Branch::Ltu => self.x[i.rs1] < self.x[i.rs2],
                    Branch::Ge => (self.x[i.rs1] as i64) >= (self.x[i.rs2] as i64),
                    Branch::Geu => self.x[i.rs1] >= self.x[i.rs2],
                };
                if taken {
                    self.pc = self.pc.wrapping_add(offset).wrapping_sub(4);
                }
            }
            BaseInstruction::Load(op, i) => {
                let addr = self.x[i.rs1].wrapping_add(i.imm as u64) as usize;
                self.x[i.rd] = match op {
                    BLoad::B => self.read_u8(addr) as i8 as i64 as u64,
                    BLoad::Bu => self.read_u8(addr) as u64,
                    BLoad::H => self.read_u16(addr) as i16 as i64 as u64,
                    BLoad::Hu => self.read_u16(addr) as u64,
                    BLoad::W => self.read_u32(addr) as i32 as i64 as u64,
                    BLoad::Wu => self.read_u32(addr) as u64,
                    BLoad::D => self.read_u64(addr),
                };
            }
            BaseInstruction::Store(op, i) => {
                let offset = ((i.imm as i64) << 52 >> 52) as u64;
                let addr = self.x[i.rs1].wrapping_add(offset) as usize;
                let val = self.x[i.rs2];
                match op {
                    BStore::B => self.write_u8(addr, val as u8),
                    BStore::H => self.write_u16(addr, val as u16),
                    BStore::W => self.write_u32(addr, val as u32),
                    BStore::D => self.write_u64(addr, val),
                }
            }
            BaseInstruction::Imm64(op, i) => {
                let imm = ((i.imm as i64) << 52 >> 52) as u64;
                let val = self.x[i.rs1];
                self.x[i.rd] = match op {
                    BImmediate64::Add => val.wrapping_add(imm),
                    BImmediate64::Slt => ((val as i64) < (imm as i64)) as u64,
                    BImmediate64::Sltu => (val < imm) as u64,
                    BImmediate64::Xor => val ^ imm,
                    BImmediate64::Or => val | imm,
                    BImmediate64::And => val & imm,
                    BImmediate64::Sll => val.wrapping_shl(imm as u32),
                    BImmediate64::Srl => val.wrapping_shr(imm as u32),
                    BImmediate64::Sra => (val as i64).wrapping_shr(imm as u32) as u64,
                };
            }
            BaseInstruction::Imm32(op, i) => {
                let imm = i.imm as u32;
                let val = self.x[i.rs1] as u32;
                self.x[i.rd] = match op {
                    BImmediate32::Add => val.wrapping_add(((imm as i32) << 20 >> 20) as u32),
                    BImmediate32::Sll => val.wrapping_shl(imm),
                    BImmediate32::Srl => val.wrapping_shr(imm),
                    BImmediate32::Sra => (val as i32).wrapping_shr(imm) as u32,
                } as i32 as i64 as u64;
            }
            BaseInstruction::Reg64(op, i) => {
                let a = self.x[i.rs1];
                let b = self.x[i.rs2];
                self.x[i.rd] = match op {
                    BRegister64::Add => a.wrapping_add(b),
                    BRegister64::Sub => a.wrapping_sub(b),
                    BRegister64::Slt => ((a as i64) < (b as i64)) as u64,
                    BRegister64::Sltu => (a < b) as u64,
                    BRegister64::Xor => a ^ b,
                    BRegister64::Or => a | b,
                    BRegister64::And => a & b,
                    BRegister64::Sll => a.wrapping_shl((b & 0x3f) as u32),
                    BRegister64::Srl => a.wrapping_shr((b & 0x3f) as u32),
                    BRegister64::Sra => (a as i64).wrapping_shr((b & 0x3f) as u32) as u64,
                };
            }
            BaseInstruction::Reg32(op, i) => {
                let a = self.x[i.rs1] as i32;
                let b = self.x[i.rs2] as i32;
                self.x[i.rd] = match op {
                    BRegister32::Add => a.wrapping_add(b),
                    BRegister32::Sub => a.wrapping_sub(b),
                    BRegister32::Sll => a.wrapping_shl((b & 0x1f) as u32),
                    BRegister32::Srl => (a as u32).wrapping_shl((b & 0x1f) as u32) as i32,
                    BRegister32::Sra => a.wrapping_shl((b & 0x1f) as u32),
                } as i64 as u64;
            }
            BaseInstruction::Fence(_) => (),
            BaseInstruction::Ecall => {
                match self.privilege {
                    Privilege::User => self.set_mtrap(8),
                    Privilege::Machine => self.set_mtrap(11),
                }
                self.minstret = self.minstret.wrapping_sub(1);
            }
            BaseInstruction::Ebreak => {
                self.set_mtrap(3);
                self.minstret = self.minstret.wrapping_sub(1)
            }
        }
    }
}
