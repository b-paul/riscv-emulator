use super::device::{AccessType, Device, DeviceRegister};

pub struct Tester {
    addr: usize,
    exit_code: Option<u32>,
}

impl Tester {
    pub fn new(addr: usize) -> Tester {
        Tester {
            addr,
            exit_code: None,
        }
    }

    pub fn get_exit_code(&self) -> Option<u32> {
        self.exit_code
    }
}

impl Device for Tester {
    fn get_registers(&self) -> Vec<DeviceRegister> {
        vec![DeviceRegister {
            addr: self.addr,
            size: 4,
            access_type: AccessType::Write,
        }]
    }

    fn read_bytes(&mut self, _addr: usize, _size: usize) -> Vec<u8> {
        unreachable!()
    }

    fn write_bytes(&mut self, _addr: usize, val: &[u8]) {
        assert!(val.len() == 4);
        let mut bytes = [0; 4];
        bytes.copy_from_slice(val);
        let code = u32::from_le_bytes(bytes) >> 1;
        self.exit_code = Some(code);
    }
}
