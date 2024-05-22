use super::device::{AccessType, Device, DeviceRegister};

pub struct Tester {
    addr: usize,
}

impl Tester {
    pub fn new(addr: usize) -> Tester {
        Tester {
            addr,
        }
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

    fn read_bytes(&self, _addr: usize, _size: usize) -> Vec<u8> {
        unreachable!()
    }

    fn write_bytes(&self, _addr: usize, val: &[u8]) {
        assert!(val.len() == 4);
        let mut bytes = [0; 4];
        bytes.copy_from_slice(val);
        let code = u32::from_le_bytes(bytes) >> 1;

        println!("Code {code}");

        std::process::exit(code as i32);
    }
}
