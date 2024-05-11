use crate::{
    instructions::atomic::{AAmoD, AAmoW, AMem, AOp, AtomicInstruction},
    Emulator,
};
use std::sync::atomic::Ordering;

impl Emulator {
    pub fn execute_atomic(
        &mut self,
        AtomicInstruction {
            aq: _,
            rl: _,
            op,
            instr,
        }: AtomicInstruction,
    ) {
        if self.misa & 1 == 0 {
            self.illegal_instruction();
            return;
        }
        let addr = instr.rs1;
        match op {
            AOp::Mem(op) => match op {
                AMem::LrW => {
                    if instr.rs2 != 0 {
                        self.illegal_instruction();
                    }
                    self.x[instr.rd] = self.read_u32(addr) as i32 as i64 as u64;

                    self.reservation.store(addr | 0b01, Ordering::Relaxed);
                }
                AMem::ScW => {
                    if instr.rs2 != 0 {
                        self.illegal_instruction();
                    }
                    self.x[instr.rd] = self.read_u64(addr);

                    self.reservation.store(addr | 0b10, Ordering::Relaxed);
                }
                AMem::LrD => {
                    if self.reservation.load(Ordering::Acquire) == addr | 0b01 {
                        self.write_u32(addr, self.x[instr.rs2] as u32);
                        self.reservation.store(0, Ordering::Release);
                        self.x[instr.rd] = 0;
                    } else {
                        self.x[instr.rd] = 1;
                    }
                }
                AMem::ScD => {
                    if self.reservation.load(Ordering::Acquire) == addr | 0b10 {
                        self.write_u64(addr, self.x[instr.rs2]);
                        self.reservation.store(0, Ordering::Release);
                        self.x[instr.rd] = 0;
                    } else {
                        self.x[instr.rd] = 1;
                    }
                }
            },
            AOp::AmoW(op) => {
                let inp = self.read_u32(addr);
                let inp2 = self.x[instr.rs2] as u32;
                let out = match op {
                    AAmoW::Swap => inp2,
                    AAmoW::Add => inp.wrapping_add(inp2),
                    AAmoW::Xor => inp ^ inp2,
                    AAmoW::And => inp & inp2,
                    AAmoW::Or => inp | inp2,
                    AAmoW::Min => (inp as i32).min(inp2 as i32) as u32,
                    AAmoW::Max => (inp as i32).max(inp2 as i32) as u32,
                    AAmoW::Minu => inp.min(inp2),
                    AAmoW::Maxu => inp.max(inp2),
                };
                self.x[instr.rd] = inp as i64 as u64;
                self.write_u32(addr, out);
            }
            AOp::AmoD(op) => {
                let inp = self.read_u64(addr);
                let inp2 = self.x[instr.rs2];
                let out = match op {
                    AAmoD::Swap => inp2,
                    AAmoD::Add => inp.wrapping_add(inp2),
                    AAmoD::Xor => inp ^ inp2,
                    AAmoD::And => inp & inp2,
                    AAmoD::Or => inp | inp2,
                    AAmoD::Min => (inp as i64).min(inp2 as i64) as u64,
                    AAmoD::Max => (inp as i64).max(inp2 as i64) as u64,
                    AAmoD::Minu => inp.min(inp2),
                    AAmoD::Maxu => inp.max(inp2),
                };
                self.x[instr.rd] = inp;
                self.write_u64(addr, out);
            }
        }
    }
}
