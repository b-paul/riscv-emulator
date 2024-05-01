use crate::{instructions::base::{BaseInstruction, Branch, BImmediate32, BImmediate64, BRegister32, BRegister64}, Emulator, Privilege};

impl Emulator {
    pub fn execute_base(&mut self, instruction: BaseInstruction) {
        if self.misa & 1 << 8 != 0 {
            self.illegal_instruction();
            return;
        }
        match instruction {
            BaseInstruction::Lui(i) => self.x[i.rd] = (i.imm << 20 >> 20) as u64,
            BaseInstruction::Auipc(i) => self.x[i.rd] = self.pc + (i.imm << 20 >> 20) as u64,
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
                let offset = ((i.imm as i32) << 20 >> 19) as u64;
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
            },
            BaseInstruction::Lb(i) => self.x[i.rd] = self.read_u8(self.x[i.rs1].wrapping_add(i.imm as u64) as usize) as i8 as i64 as u64,
            BaseInstruction::Lbu(i) => self.x[i.rd] = self.read_u8(self.x[i.rs1].wrapping_add(i.imm as u64) as usize) as u64,
            BaseInstruction::Lh(i) => self.x[i.rd] = self.read_u16(self.x[i.rs1].wrapping_add(i.imm as u64) as usize) as i16 as i64 as u64,
            BaseInstruction::Lhu(i) => self.x[i.rd] = self.read_u16(self.x[i.rs1].wrapping_add(i.imm as u64) as usize) as u64,
            BaseInstruction::Lw(i) => self.x[i.rd] = self.read_u32(self.x[i.rs1].wrapping_add(i.imm as u64) as usize) as i32 as i64 as u64,
            BaseInstruction::Lwu(i) => self.x[i.rd] = self.read_u32(self.x[i.rs1].wrapping_add(i.imm as u64) as usize) as u64,
            BaseInstruction::Ld(i) => self.x[i.rd] = self.read_u64(self.x[i.rs1].wrapping_add(i.imm as u64) as usize),
            BaseInstruction::Sb(i) => self.write_u8(self.x[i.rs1].wrapping_add(((i.imm as i32) << 20 >> 20) as i64 as u64) as usize, self.x[i.rs2] as u8),
            BaseInstruction::Sh(i) => self.write_u16(self.x[i.rs1].wrapping_add(((i.imm as i32) << 20 >> 20) as i64 as u64) as usize, self.x[i.rs2] as u16),
            BaseInstruction::Sw(i) => self.write_u32(self.x[i.rs1].wrapping_add(((i.imm as i32) << 20 >> 20) as i64 as u64) as usize, self.x[i.rs2] as u32),
            BaseInstruction::Sd(i) => self.write_u64(self.x[i.rs1].wrapping_add(((i.imm as i32) << 20 >> 20) as i64 as u64) as usize, self.x[i.rs2]),
            BaseInstruction::Imm64(op, i) => {
                let imm = ((i.imm as i64) << 32 >> 52) as u64;
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
                    BImmediate32::Add => val.wrapping_add(imm),
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
            BaseInstruction::Ecall(_) => {
                match self.privilege {
                    Privilege::User => self.set_mtrap(8),
                    Privilege::Machine => self.set_mtrap(11),
                }
                self.minstret = self.minstret.wrapping_sub(1);
            },
            BaseInstruction::Ebreak(_) => {
                self.set_mtrap(3);
                self.minstret = self.minstret.wrapping_sub(1)
            },
        }
    }
}
