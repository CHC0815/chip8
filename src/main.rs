use std::{
    env,
    fs::{self, File},
    io::Read,
};

use chip8::disassembler;
use chip8::emulator;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        3 => {
            let prog = &args[1];
            let what = &args[2];

            let mut file = File::open(what).expect("Could not open file");
            let metadata = fs::metadata(what).expect("Could not read metadata");
            let mut buffer = vec![0; metadata.len() as usize];
            file.read(&mut buffer).expect("buffer overflow");

            match prog.as_str() {
                "dis" => {
                    println!("Disassembling: {}", what);
                    let mut disassembler = disassembler::Disassembler::new();
                    disassembler.load(&buffer);
                    disassembler.disassemble();
                }
                "emu" => {
                    println!("Emulating: {}", what);
                    emulator::emulate(what);
                }
                _ => println!("Unknown program. Use 'dis' or 'emu'"),
            }
        }
        _ => println!("Too few or too many arguments"),
    }
}
