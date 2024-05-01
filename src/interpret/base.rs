use crate::{instructions::base::BaseInstruction, Emulator, Privilege};

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
            BaseInstruction::Beq(i) => {
                let offset = ((i.imm as i32) << 20 >> 19) as u64;
                if self.x[i.rs1] == self.x[i.rs2] {
                    self.pc = self.pc.wrapping_add(offset).wrapping_sub(4);
                }
            },
            BaseInstruction::Bne(i) => {
                let offset = ((i.imm as i32) << 20 >> 19) as u64;
                if self.x[i.rs1] != self.x[i.rs2] {
                    self.pc = self.pc.wrapping_add(offset).wrapping_sub(4);
                }
            },
            BaseInstruction::Blt(i) => {
                let offset = ((i.imm as i32) << 20 >> 19) as u64;
                if (self.x[i.rs1] as i64) < (self.x[i.rs2] as i64) {
                    self.pc = self.pc.wrapping_add(offset).wrapping_sub(4);
                }
            },
            BaseInstruction::Bltu(i) => {
                let offset = ((i.imm as i32) << 20 >> 19) as u64;
                if self.x[i.rs1] < self.x[i.rs2] {
                    self.pc = self.pc.wrapping_add(offset).wrapping_sub(4);
                }
            },
            BaseInstruction::Bge(i) => {
                let offset = ((i.imm as i32) << 20 >> 19) as u64;
                if (self.x[i.rs1] as i64) >= (self.x[i.rs2] as i64) {
                    self.pc = self.pc.wrapping_add(offset).wrapping_sub(4);
                }
            },
            BaseInstruction::Bgeu(i) => {
                let offset = ((i.imm as i32) << 20 >> 19) as u64;
                if self.x[i.rs1] >= self.x[i.rs2] {
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
            BaseInstruction::Sd(i) => self.write_u64(self.x[i.rs1].wrapping_add(((i.imm as i32) << 20 >> 20) as i64 as u64) as usize, self.x[i.rs2] as u64),
            BaseInstruction::Addi(i) => self.x[i.rd] = (self.x[i.rs1] as i64).wrapping_add((i.imm as i64) << 32 >> 52) as u64,
            BaseInstruction::Slti(i) => self.x[i.rd] = if (self.x[i.rs1] as i64) < (i.imm as i64) << 32 >> 52 { 1 } else { 0 },
            BaseInstruction::Sltiu(i) => self.x[i.rd] = if self.x[i.rs1] < ((i.imm as i64) << 32 >> 52) as u64 { 1 } else { 0 },
            BaseInstruction::Xori(i) => self.x[i.rd] = self.x[i.rs1] ^ ((i.imm as i64) << 32 >> 52) as u64,
            BaseInstruction::Ori(i) => self.x[i.rd] = self.x[i.rs1] | ((i.imm as i64) << 32 >> 52) as u64,
            BaseInstruction::Andi(i) => self.x[i.rd] = self.x[i.rs1] ^ ((i.imm as i64) << 32 >> 52) as u64,
            BaseInstruction::Slli(i) => self.x[i.rd] = self.x[i.rs1].wrapping_shl(i.imm as u32),
            BaseInstruction::Srli(i) => self.x[i.rd] = self.x[i.rs1].wrapping_shl(i.imm as u32),
            BaseInstruction::Srai(i) => self.x[i.rd] = (self.x[i.rs1] as i64).wrapping_shl(i.imm as u32) as u64,
            BaseInstruction::Add(i) => self.x[i.rd] = self.x[i.rs1].wrapping_add(self.x[i.rs2]),
            BaseInstruction::Sub(i) => self.x[i.rd] = self.x[i.rs1].wrapping_sub(self.x[i.rs2]),
            BaseInstruction::Sll(i) => self.x[i.rd] = self.x[i.rs1].wrapping_shl((self.x[i.rs2] & 0x3f) as u32),
            BaseInstruction::Slt(i) => self.x[i.rd] = if (self.x[i.rs1] as i64) < (self.x[i.rs2] as i64) { 1 } else { 0 },
            BaseInstruction::Sltu(i) => self.x[i.rd] = if self.x[i.rs1] < self.x[i.rs2] { 1 } else { 0 },
            BaseInstruction::Xor(i) => self.x[i.rd] = self.x[i.rs1] ^ self.x[i.rs2],
            BaseInstruction::Srl(i) => self.x[i.rd] = self.x[i.rs1].wrapping_shr((self.x[i.rs2] & 0x3f) as u32),
            BaseInstruction::Sra(i) => self.x[i.rd] = ((self.x[i.rs1] as i64) >> (self.x[i.rs2] & 0x3f)) as u64,
            BaseInstruction::Or(i) => self.x[i.rd] = self.x[i.rs1] | self.x[i.rs2],
            BaseInstruction::And(i) => self.x[i.rd] = self.x[i.rs1] & self.x[i.rs2],
            BaseInstruction::Addiw(i) => self.x[i.rd] = (self.x[i.rs1] as i32).wrapping_add((i.imm as i32) >> 20) as i64 as u64,
            BaseInstruction::Slliw(i) => self.x[i.rd] = (self.x[i.rs1] as u32).wrapping_shl(i.imm as u32) as u64,
            BaseInstruction::Srliw(i) => self.x[i.rd] = (self.x[i.rs1] as u32).wrapping_shl(i.imm as u32) as u64,
            BaseInstruction::Sraiw(i) => self.x[i.rd] = (self.x[i.rs1] as i32).wrapping_shl(i.imm as u32) as i64 as u64,
            BaseInstruction::Addw(i) => self.x[i.rd] = (self.x[i.rs1] as i32).wrapping_add(self.x[i.rs2] as i32) as i64 as u64,
            BaseInstruction::Subw(i) => self.x[i.rd] = (self.x[i.rs1] as i32).wrapping_sub(self.x[i.rs2] as i32) as i64 as u64,
            BaseInstruction::Sllw(i) => self.x[i.rd] = (self.x[i.rs1] as i32).wrapping_shl((self.x[i.rs2] & 0x1f) as u32) as i64 as u64,
            BaseInstruction::Srlw(i) => self.x[i.rd] = (self.x[i.rs1] as u32).wrapping_shr((self.x[i.rs2] & 0x1f) as u32) as i32 as i64 as u64,
            BaseInstruction::Sraw(i) => self.x[i.rd] = (self.x[i.rs1] as i32).wrapping_shr((self.x[i.rs2] & 0x1f) as u32) as i64 as u64,
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
