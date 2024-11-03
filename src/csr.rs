use crate::{Emulator, Privilege};

pub struct MachineCsrs {
    pub misa: u64,
    pub mstatus: u64,
    pub mtvec: u64,
    pub mip: u64,
    pub mie: u64,
    pub mcycle: u64,
    pub minstret: u64,
    pub mcounteren: u32,
    pub mcountinhibit: u32,
    pub mscratch: u64,
    pub mepc: u64,
    pub mcause: u64,
    pub mtval: u64,
    pub menvcfg: u64,
    pub mseccfg: u64,
    pub mtime: u64,
    pub mtimecmp: u64,
}

impl Default for MachineCsrs {
    fn default() -> Self {
        Self {
            misa: 2 << 62 | 0b00000100000001000100000101,
            mstatus: 0,
            mtvec: 0,
            mip: 0,
            mie: 0,
            mcycle: 0,
            minstret: 0,
            mcounteren: 0,
            mcountinhibit: 0,
            mscratch: 0,
            mepc: 0,
            mcause: 0,
            mtval: 0,
            menvcfg: 0,
            mseccfg: 0,
            mtime: 0,
            mtimecmp: 0,
        }
    }
}

impl Emulator {
    pub fn get_csr(&mut self, csr: u32, _read: bool) -> Option<u64> {
        if self.privilege >= Privilege::Machine {
            let val = match csr {
                0x301 => Some(self.machine_csrs.misa),                 // misa
                0xF11 => Some(0),                                      // mvendorid
                0xF12 => Some(0),                                      // marchid
                0xF13 => Some(0),                                      // mimpid
                0xF14 => Some(0),                                      // mhartid
                0x300 => Some(self.machine_csrs.mstatus),              // mstatus
                0x305 => Some(self.machine_csrs.mtvec),                // mtvec
                0x344 => Some(0),                                      // mip
                0x304 => Some(0),                                      // mie
                0xB00 => Some(self.machine_csrs.mcycle),               // mcycle
                0xB02 => Some(self.machine_csrs.minstret),             // minstret
                0xB04..=0xB1F => Some(0), // mhpmcounterN (unimplemented)
                0x323..=0x33F => Some(0), // mhpmeventN (unimplemented)
                0x306 => Some(self.machine_csrs.mcounteren as u64), // mcounteren
                0x320 => Some(self.machine_csrs.mcountinhibit as u64), // mcountinhibit
                0x340 => Some(self.machine_csrs.mscratch), // mscratch
                0x341 => Some(self.machine_csrs.mepc), // mepc
                0x342 => Some(self.machine_csrs.mcause), // mcause
                0x343 => Some(self.machine_csrs.mtval), // mtval
                0xF15 => Some(0),         // mconfigptr
                0x30A => Some(self.machine_csrs.menvcfg), // menvcfg
                0x747 => Some(self.machine_csrs.mseccfg), // mseccfg
                _ => None,
            };
            if val.is_some() {
                return val;
            }
        }
        if self.privilege == Privilege::User {
            let val = match csr {
                0xC00 if self.machine_csrs.mcounteren & 1 != 0 => Some(self.machine_csrs.mcycle),
                0xC01 if self.machine_csrs.mcounteren & 2 != 0 => Some(self.machine_csrs.mtime),
                0xC02 if self.machine_csrs.mcounteren & 4 != 0 => Some(self.machine_csrs.minstret),
                0xC03..=0xC1F if self.machine_csrs.mcounteren & 1 << (csr - 0xC00) != 0 => {
                    Some(self.machine_csrs.minstret)
                }
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
                    let old_mpp = self.machine_csrs.mstatus & (3 << 11);
                    self.machine_csrs.mstatus = val & !0x7fffffc0ff800015;
                    // We don't implement S yet, so keep SPP to 0
                    self.machine_csrs.mstatus &= !0x100;
                    // We want to ensure that MPP only has legal values (we don't implement S yet)
                    let mpp = self.machine_csrs.mstatus & (3 << 11);
                    if mpp != 0b00 << 11 && mpp != 0b11 << 11 {
                        self.machine_csrs.mstatus =
                            self.machine_csrs.mstatus & !(3 << 11) | old_mpp;
                    }
                    // SXL is read only 0 since we do not implement S yet
                    self.machine_csrs.mstatus &= !(3 << 34);
                    // Ensure that UXL stays on 64 bit, since we don't want to allow variable len
                    self.machine_csrs.mstatus = (self.machine_csrs.mstatus & !(3 << 32)) | 2 << 32;
                    // MPRIV is read only 0 if U is not implemented
                    self.machine_csrs.mstatus &= !(1 << 17);
                    // MXR is read only 0 if S is not implemented
                    self.machine_csrs.mstatus &= !(1 << 19);
                    // SUM is read only 0 if S is not implemented
                    self.machine_csrs.mstatus &= !(1 << 18);
                    // We only support little endian, so MBE, SBE and UBE are effectively read only 0.
                    self.machine_csrs.mstatus &= !(1 << 37);
                    self.machine_csrs.mstatus &= !(1 << 36);
                    self.machine_csrs.mstatus &= !(1 << 6);
                    // TVM is read only 0 if S is not implemented
                    self.machine_csrs.mstatus &= !(1 << 20);
                    // TW is read only 0 if there are no modes less than M implemented
                    self.machine_csrs.mstatus &= !(1 << 21);
                    // TSR is read only 0 if S is not implemented
                    self.machine_csrs.mstatus &= !(1 << 22);
                    // For simplicity FS will always say dirty
                    self.machine_csrs.mstatus |= 3 << 13;
                    // VS and XS are read only zero as neither V nor X are implemented
                    self.machine_csrs.mstatus &= !(3 << 9);
                    self.machine_csrs.mstatus &= !(3 << 15);
                }
                0x305 => self.machine_csrs.mtvec = val & !3, // mtvec (we assume always direct)
                // mip
                0x344 => {
                    // For us everything in the bottom 16 bites of mip is read only
                    self.machine_csrs.mip = val & !0xffff;
                }
                // mie
                0x304 => {
                    // Zero out the zero bits of mie.
                    self.machine_csrs.mie = val & !0xd555;
                    // SEIP, STIP and SSIP are read only zero since S is not implemented
                    self.machine_csrs.mie &= !(1 << 1);
                    self.machine_csrs.mie &= !(1 << 5);
                    self.machine_csrs.mie &= !(1 << 9);
                    // LCOFIE is read only zero since Sscofpmf is not implemented
                    self.machine_csrs.mie &= !(1 << 13);
                }
                0xB00 => self.machine_csrs.mcycle = val, // mcycle
                0xB02 => self.machine_csrs.minstret = val, // minstret
                0x306 => self.machine_csrs.mcounteren = val as u32, // mcounteren
                0x340 => self.machine_csrs.mscratch = val, // mscratch
                0x341 => self.machine_csrs.mepc = val & !1, // mepc
                0x342 => self.machine_csrs.mcause = val, // mcause
                0x343 => self.machine_csrs.mtval = val,  // mtval
                0x30A => self.machine_csrs.menvcfg = val, // menvcfg TODO
                0x747 => self.machine_csrs.mseccfg = val, // mseccfg TODO?

                _ => return false,
            }
            return true;
        }
        false
    }
}
