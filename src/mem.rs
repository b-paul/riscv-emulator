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

    pub fn read_bytes(&self, addr: usize, count: usize) -> Result<&[u8], AccessFault> {
        self.bytes.get(addr..addr + count).ok_or(AccessFault::Load)
    }

    pub fn write_bytes(&mut self, mut addr: usize, bytes: &[u8]) -> Result<(), AccessFault> {
        for &b in bytes {
            *self.bytes.get_mut(addr).ok_or(AccessFault::Store)? = b;
            addr += 1;
            addr %= self.size;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum AccessFault {
    Load,
    Store,
}

pub enum ReadBuffer<'a> {
    Memory(&'a [u8]),
    Device(Vec<u8>),
}

impl ReadBuffer<'_> {
    fn to_slice(&self) -> &[u8] {
        match self {
            ReadBuffer::Memory(b) => b,
            ReadBuffer::Device(v) => v,
        }
    }
}

impl Emulator {
    pub fn read_bytes(&self, addr: usize, count: usize) -> Result<ReadBuffer<'_>, AccessFault> {
        if let Some((idx, reg)) = self.device_map.get(&addr) {
            if !reg.access_type.can_read() {
                return Err(AccessFault::Load);
            }
            let bytes = self.devices[*idx]
                .borrow_mut()
                .read_bytes(addr, count)
                .to_vec();
            Ok(ReadBuffer::Device(bytes))
        } else {
            match addr {
                // CLINT
                0x2000000..=0x200BFFF => match addr - 0x2000000 {
                    0x0 => todo!("msip"),
                    _ => todo!("Slave bus error on invalid access or misaligned read"),
                },
                _ => Ok(ReadBuffer::Memory(self.memory.read_bytes(addr, count)?)),
            }
        }
    }

    pub fn read_u8(&self, addr: usize) -> Result<u8, AccessFault> {
        let bytes = self.read_bytes(addr, 1)?;
        let bytes = bytes.to_slice();
        let mut buf = [0; 1];
        buf.copy_from_slice(bytes);
        Ok(u8::from_le_bytes(buf))
    }

    pub fn read_u16(&self, addr: usize) -> Result<u16, AccessFault> {
        let bytes = self.read_bytes(addr, 2)?;
        let bytes = bytes.to_slice();
        let mut buf = [0; 2];
        buf.copy_from_slice(bytes);
        Ok(u16::from_le_bytes(buf))
    }

    pub fn read_u32(&self, addr: usize) -> Result<u32, AccessFault> {
        let bytes = self.read_bytes(addr, 4)?;
        let bytes = bytes.to_slice();
        let mut buf = [0; 4];
        buf.copy_from_slice(bytes);
        Ok(u32::from_le_bytes(buf))
    }

    pub fn read_u64(&self, addr: usize) -> Result<u64, AccessFault> {
        match addr {
            // CLINT
            0x2000000..=0x200BFFF => match addr - 0x2000000 {
                0x4000 => Ok(self.mtimecmp),
                0x8000 => Ok(self.mtime),
                _ => todo!("Slave bus error on invalid access or misaligned read"),
            },
            _ => {
                let bytes = self.read_bytes(addr, 8)?;
                let bytes = bytes.to_slice();
                let mut buf = [0; 8];
                buf.copy_from_slice(bytes);
                Ok(u64::from_le_bytes(buf))
            }
        }
    }

    pub fn write_bytes(&mut self, addr: usize, bytes: &[u8]) -> Result<(), AccessFault> {
        if let Some((idx, reg)) = self.device_map.get(&addr) {
            if !reg.access_type.can_write() {
                return Err(AccessFault::Store);
            }
            self.devices[*idx].borrow_mut().write_bytes(addr, bytes);
        } else {
            match addr {
                // CLINT
                0x2000000..=0x200BFFF => {
                    let addr = addr - 0x2000000;
                    match addr {
                        0x0 => todo!("msip"),
                        _ => todo!("Slave bus error on invalid access or misaligned write"),
                    }
                }
                _ => self.memory.write_bytes(addr, bytes)?,
            }
        }
        Ok(())
    }

    pub fn write_u8(&mut self, addr: usize, val: u8) -> Result<(), AccessFault> {
        self.write_bytes(addr, &val.to_le_bytes())
    }

    pub fn write_u16(&mut self, addr: usize, val: u16) -> Result<(), AccessFault> {
        self.write_bytes(addr, &val.to_le_bytes())
    }

    pub fn write_u32(&mut self, addr: usize, val: u32) -> Result<(), AccessFault> {
        self.write_bytes(addr, &val.to_le_bytes())
    }

    pub fn write_u64(&mut self, addr: usize, val: u64) -> Result<(), AccessFault> {
        match addr {
            // CLINT
            0x2000000..=0x200BFFF => {
                let addr = addr - 0x2000000;
                match addr {
                    0x4000 => self.mtimecmp = val,
                    0x8000 => self.mtime = val,
                    _ => todo!("Slave bus error on invalid access or misaligned write"),
                }
                Ok(())
            }
            _ => self.write_bytes(addr, &val.to_le_bytes()),
        }
    }
}
