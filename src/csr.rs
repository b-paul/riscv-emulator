use crate::{Emulator, Privilege};

impl Emulator {
    pub fn get_csr(&mut self, csr: u32, _read: bool) -> Option<u64> {
        if self.privilege >= Privilege::Machine {
            let val = match csr {
                0x301 => Some(self.misa),                 // misa
                0xF11 => Some(0),                         // mvendorid
                0xF12 => Some(0),                         // marchid
                0xF13 => Some(0),                         // mimpid
                0xF14 => Some(0),                         // mhartid
                0x300 => Some(self.mstatus),              // mstatus
                0x305 => Some(self.mtvec),                // mtvec
                0x344 => Some(0),                         // mip
                0x304 => Some(0),                         // mie
                0xB00 => Some(self.mcycle),               // mcycle
                0xB02 => Some(self.minstret),             // minstret
                0xB04..=0xB1F => Some(0),                 // mhpmcounterN (unimplemented)
                0x323..=0x33F => Some(0),                 // mhpmeventN (unimplemented)
                0x306 => Some(self.mcounteren as u64),    // mcounteren
                0x320 => Some(self.mcountinhibit as u64), // mcountinhibit
                0x340 => Some(self.mscratch),             // mscratch
                0x341 => Some(self.mepc),                 // mepc
                0x342 => Some(self.mcause),               // mcause
                0x343 => Some(self.mtval),                // mtval
                0xF15 => Some(0),                         // mconfigptr
                0x30A => Some(self.menvcfg),              // menvcfg
                0x747 => Some(self.mseccfg),              // mseccfg
                _ => None,
            };
            if val.is_some() {
                return val;
            }
        }
        if self.privilege == Privilege::User {
            let val = match csr {
                0xC00 if self.mcounteren & 1 != 0 => Some(self.mcycle),
                0xC01 if self.mcounteren & 2 != 0 => Some(self.mtime),
                0xC02 if self.mcounteren & 4 != 0 => Some(self.minstret),
                0xC03..=0xC1F if self.mcounteren & 1 << (csr - 0xC00) != 0 => Some(self.minstret),
                _ => None,
            };
            if val.is_some() {
                return val;
            }
        }
        None
    }

    pub fn set_csr(&mut self, csr: u32, val: u64, write: bool) -> bool {
        if !write {
            return true;
        }
        if self.privilege >= Privilege::Machine {
            match csr {
                // misa
                // Don't allow modification of allowed extensions for simplicity
                // (might change later)
                0x301 => {}
                // mstatus (0x7fffffc0ff800015 is the WPRI mask)
                0x300 => {
                    let old_mpp = self.mstatus & (3 << 11);
                    self.mstatus = val & !0x7fffffc0ff800015;
                    // We don't implement S yet, so keep SPP to 0
                    self.mstatus &= !0x100;
                    // We want to ensure that MPP only has legal values (we don't implement S yet)
                    let mpp = self.mstatus & (3 << 11);
                    if mpp != 0b00 << 11 && mpp != 0b11 << 11 {
                        self.mstatus = self.mstatus & !(3 << 11) | old_mpp;
                    }
                    // SXL is read only 0 since we do not implement S yet
                    self.mstatus &= !(3 << 34);
                    // Ensure that UXL stays on 64 bit, since we don't want to allow variable len
                    self.mstatus = (self.mstatus & !(3 << 32)) | 2 << 32;
                    // MPRIV is read only 0 if U is not implemented
                    self.mstatus &= !(1 << 17);
                    // MXR is read only 0 if S is not implemented
                    self.mstatus &= !(1 << 19);
                    // SUM is read only 0 if S is not implemented
                    self.mstatus &= !(1 << 18);
                    // We only support little endian, so MBE, SBE and UBE are effectively read only 0.
                    self.mstatus &= !(1 << 37);
                    self.mstatus &= !(1 << 36);
                    self.mstatus &= !(1 << 6);
                    // TVM is read only 0 if S is not implemented
                    self.mstatus &= !(1 << 20);
                    // TW is read only 0 if there are no modes less than M implemented
                    self.mstatus &= !(1 << 21);
                    // TSR is read only 0 if S is not implemented
                    self.mstatus &= !(1 << 22);
                    // For simplicity FS will always say dirty
                    self.mstatus |= 3 << 13;
                    // VS and XS are read only zero as neither V nor X are implemented
                    self.mstatus &= !(3 << 9);
                    self.mstatus &= !(3 << 15);
                }
                0x305 => self.mtvec = val & !3, // mtvec (we assume always direct)
                // mip
                0x344 => {
                    // For us everything in the bottom 16 bites of mip is read only
                    self.mip = val & !0xffff;
                }
                // mie
                0x304 => {
                    // Zero out the zero bits of mie.
                    self.mie = val & !0xd555;
                    // SEIP, STIP and SSIP are read only zero since S is not implemented
                    self.mie &= !(1 << 1);
                    self.mie &= !(1 << 5);
                    self.mie &= !(1 << 9);
                    // LCOFIE is read only zero since Sscofpmf is not implemented
                    self.mie &= !(1 << 13);
                }
                0xB00 => self.mcycle = val,            // mcycle
                0xB02 => self.minstret = val,          // minstret
                0x306 => self.mcounteren = val as u32, // mcounteren
                0x340 => self.mscratch = val,          // mscratch
                0x341 => self.mepc = val & !1,         // mepc
                0x342 => self.mcause = val,            // mcause
                0x343 => self.mtval = val,             // mtval
                0x30A => self.menvcfg = val,           // menvcfg TODO
                0x747 => self.mseccfg = val,           // mseccfg TODO?

                _ => return false,
            }
            return true;
        }
        false
    }
}
