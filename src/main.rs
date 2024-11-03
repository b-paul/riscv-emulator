use std::rc::Rc;
use std::cell::RefCell;

use riscv::{Emulator, device::Device, tester::Tester};

fn main() {
    let path = std::env::args().nth(1).unwrap();

    for entry in std::fs::read_dir(path).unwrap().flatten() {
        let name = entry.file_name();
        let name = name.to_str().unwrap();

        if !name.starts_with("rv64mi-p-") || name.ends_with(".dump") {
            continue;
        }

        let path = entry.path();
        let path = path.to_str().unwrap();

        let mut emu = Emulator::new(128 * 1024 * 1024);

        let tester_addr = match name {
            "rv64ui-p-ma_data" => 0x3000,
            "rv64uc-p-rvc" => 0x4000,
            _ => 0x2000,
        };

        let tester = Rc::new(RefCell::new(Tester::new(tester_addr)));

        emu.load_binary(path, 0x1000).unwrap();

        emu.add_device(tester.clone() as Rc<RefCell<dyn Device>>);

        loop {
            emu.cycle();
            if let Some(code) = tester.borrow().get_exit_code() {
                println!("{name}: {code}");
                break;
            }
        }
    }
}
