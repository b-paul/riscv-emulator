use crate::Emulator;

pub struct Memory {
    bytes: Box<[u8]>,
    size: usize,
}

impl Memory {
    pub fn new(size: usize) -> Memory {
        Memory {
            bytes: vec![0; size].into_boxed_slice(),
            size,
        }
    }

    // slightly confusing api with this being arrays and write_bytes being slices but whatever
    pub fn read_bytes<const COUNT: usize>(&self, mut addr: usize) -> [u8; COUNT] {
        let mut out = [0; COUNT];
        for b in out.iter_mut() {
            *b = self.bytes[addr];
            addr += 1;
            addr %= self.size;
        }
        out
    }

    pub fn write_bytes(&mut self, mut addr: usize, bytes: &[u8]) {
        for &b in bytes {
            self.bytes[addr] = b;
            addr += 1;
            addr %= self.size;
        }
    }
}

impl Emulator {
    pub fn read_bytes<const COUNT: usize>(&self, addr: usize) -> [u8; COUNT] {
        /* TODO this needs to be dynamic i think oh god
        if let Some((idx, reg)) = self.device_map.get(&addr) {
            if !reg.access_type.can_read() {
                todo!()
            }
            self.devices[*idx].write_bytes(addr, bytes);
        }
        */
        match addr {
            // CLINT
            0x2000000..=0x200BFFF => match addr - 0x2000000 {
                0x0 => todo!("msip"),
                _ => todo!("Slave bus error on invalid access or misaligned read"),
            },
            _ => self.memory.read_bytes(addr),
        }
    }

    pub fn read_u8(&self, addr: usize) -> u8 {
        u8::from_le_bytes(self.read_bytes(addr))
    }

    pub fn read_u16(&self, addr: usize) -> u16 {
        u16::from_le_bytes(self.read_bytes(addr))
    }

    pub fn read_u32(&self, addr: usize) -> u32 {
        u32::from_le_bytes(self.read_bytes(addr))
    }

    pub fn read_u64(&self, addr: usize) -> u64 {
        match addr {
            // CLINT
            0x2000000..=0x200BFFF => match addr - 0x2000000 {
                0x4000 => self.mtimecmp,
                0x8000 => self.mtime,
                _ => todo!("Slave bus error on invalid access or misaligned read"),
            },
            _ => u64::from_le_bytes(self.read_bytes(addr)),
        }
    }

    pub fn write_bytes(&mut self, addr: usize, bytes: &[u8]) {
        if let Some((idx, reg)) = self.device_map.get(&addr) {
            if !reg.access_type.can_write() {
                todo!()
            }
            self.devices[*idx].borrow_mut().write_bytes(addr, bytes);
            return;
        }

        match addr {
            // CLINT
            0x2000000..=0x200BFFF => {
                let addr = addr - 0x2000000;
                match addr {
                    0x0 => todo!("msip"),
                    _ => todo!("Slave bus error on invalid access or misaligned write"),
                }
            }
            _ => self.memory.write_bytes(addr, bytes),
        }
    }

    pub fn write_u8(&mut self, addr: usize, val: u8) {
        self.write_bytes(addr, &val.to_le_bytes())
    }

    pub fn write_u16(&mut self, addr: usize, val: u16) {
        self.write_bytes(addr, &val.to_le_bytes())
    }

    pub fn write_u32(&mut self, addr: usize, val: u32) {
        self.write_bytes(addr, &val.to_le_bytes())
    }

    pub fn write_u64(&mut self, addr: usize, val: u64) {
        match addr {
            // CLINT
            0x2000000..=0x200BFFF => {
                let addr = addr - 0x2000000;
                match addr {
                    0x4000 => self.mtimecmp = val,
                    0x8000 => self.mtime = val,
                    _ => todo!("Slave bus error on invalid access or misaligned write"),
                }
            }
            _ => self.write_bytes(addr, &val.to_le_bytes()),
        }
    }
}
