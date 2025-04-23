use std::cell::RefCell;
use std::rc::Rc;

use clap::Parser;

use riscv::{device::Device, tester::Tester, Emulator};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to the executable to run
    executable: String,
    /// Path to a signature file to output (when running riscof tests)
    #[arg(long)]
    signature: String,
    /// Run debug mode, where each cycle is  stepped through manually
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();

    let path = args.executable;

    let mut emu = Emulator::new(128 * 1024 * 1024);

    let elf = emu.load_binary(&path).unwrap();

    let tester_addr = elf.get_symbol("tohost").unwrap().value;
    let signature_start = elf.get_symbol("begin_signature").unwrap().value;
    let signature_end = elf.get_symbol("end_signature").unwrap().value;

    let tester = Rc::new(RefCell::new(Tester::new(tester_addr)));

    emu.add_device(tester.clone() as Rc<RefCell<dyn Device>>);

    loop {
        if args.debug {
            emu.debug();

            use std::io::BufRead;
            let mut b = String::new();
            let mut h = std::io::stdin().lock();
            h.read_line(&mut b).unwrap();
        }

        emu.cycle();
        if let Some(code) = tester.borrow().get_exit_code() {
            println!("{code}");
            emu.write_signature(&args.signature, signature_start, signature_end)
                .unwrap();
            break;
        }
    }

    /*
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
    */
}
