use std::{
    env,
    fs::{self, File},
    io::Read,
};

use chip8::{consts::FONT_BASE_ADDRESS, disassembler};
use chip8::emulator;
use chip8::font::FONT;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        3 => {
            let prog = &args[1];
            let what = &args[2];

            let mut file = File::open(what).expect("Could not open file");
            let metadata = fs::metadata(what).expect("Could not read metadata");
            if metadata.len() > (4096 - 0x200) {
                // Program memory is 4096 bytes, but the first 512 bytes are reserved for the interpreter
                panic!("Program File too large");
            }
            let mut buffer = vec![0; 4096];
            file.read(&mut buffer).expect("buffer overflow");

            for i in (0..buffer.len() - 0x200).rev() {
                buffer.swap(i, i + 512);
            }
            for i in 0..0x200 {
                buffer[i] = 0; // Fill the first 512 bytes with 0
            }
            // copy font to 050-09F
            buffer[FONT_BASE_ADDRESS..FONT_BASE_ADDRESS + 80].copy_from_slice(&FONT);

            println!("{:?}", buffer);

            match prog.as_str() {
                "dis" => {
                    println!("Disassembling: {}", what);
                    let mut disassembler = disassembler::Disassembler::new();
                    disassembler.load(&buffer);
                    disassembler.disassemble();
                }
                "emu" => {
                    println!("Emulating: {}", what);
                    emulator::emulate(&buffer);
                }
                _ => println!("Unknown program. Use 'dis' or 'emu'"),
            }
        }
        _ => println!("Too few or too many arguments"),
    }
}
