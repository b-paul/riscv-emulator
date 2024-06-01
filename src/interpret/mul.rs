use crate::{
    instructions::mul::{MReg32, MReg64, MulInstruction},
    Emulator,
};

impl Emulator {
    pub fn execute_mul(&mut self, instruction: MulInstruction) {
        if self.misa & 1 << 12 == 0 {
            self.illegal_instruction();
            return;
        }
        match instruction {
            MulInstruction::Reg64(op, i) => {
                let a = self.x[i.rs1];
                let b = self.x[i.rs2];
                self.x[i.rd] = match op {
                    MReg64::Mul => a.wrapping_mul(b),
                    MReg64::Mulh => {
                        ((a as i64 as i128).wrapping_mul(b as i64 as i128) >> 64) as u64
                    }
                    MReg64::Mulhsu => {
                        ((a as i64 as i128).wrapping_mul(b as u128 as i128) >> 64) as u64
                    }
                    MReg64::Mulhu => ((a as u128).wrapping_mul(b as u128) >> 64) as u64,
                    MReg64::Div => {
                        if a as i64 == i64::MIN && b as i64 == -1 {
                            a
                        } else if b == 0 {
                            u64::MAX
                        } else {
                            (a as i64).wrapping_div(b as i64) as u64
                        }
                    }
                    MReg64::Divu => {
                        if b == 0 {
                            u64::MAX
                        } else {
                            a.wrapping_div(b)
                        }
                    }
                    MReg64::Rem => {
                        if a as i64 == i64::MIN && b as i64 == -1 {
                            0
                        } else if b == 0 {
                            a
                        } else {
                            (a as i64).wrapping_rem(b as i64) as u64
                        }
                    }
                    MReg64::Remu => {
                        if b == 0 {
                            a
                        } else {
                            a.wrapping_rem(b)
                        }
                    }
                };
            }
            MulInstruction::Reg32(op, i) => {
                let a = self.x[i.rs1] as i32;
                let b = self.x[i.rs2] as i32;
                self.x[i.rd] = match op {
                    MReg32::Mul => a.wrapping_mul(b) as i64 as u64,
                    MReg32::Div => {
                        if a == i32::MIN && b == -1 {
                            i32::MIN as i64 as u64
                        } else if b == 0 {
                            u64::MAX
                        } else {
                            a.wrapping_div(b) as i64 as u64
                        }
                    }
                    MReg32::Divu => {
                        if b == 0 {
                            u64::MAX
                        } else {
                            (a as u32).wrapping_div(b as u32) as i32 as i64 as u64
                        }
                    }
                    MReg32::Rem => {
                        if a == i32::MIN && b == -1 {
                            0
                        } else if b == 0 {
                            a as i64 as u64
                        } else {
                            a.wrapping_rem(b) as i64 as u64
                        }
                    }
                    MReg32::Remu => {
                        if b == 0 {
                            a as i64 as u64
                        } else {
                            (a as u32).wrapping_rem(b as u32) as i32 as i64 as u64
                        }
                    }
                };
            }
        }
    }
}
