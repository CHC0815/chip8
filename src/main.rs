use std::{
    env,
    fs::{self, File},
    io::Read,
};

use chip8::{disassembler, emulator, prep_buffer};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        3 => {
            let which = &args[1];

            let what = &args[2];

            let file = File::open(what).expect("Could not open file");
            let metadata = fs::metadata(what).expect("Could not read metadata");
            if metadata.len() > (4096 - 0x200) {
                // Program memory is 4096 bytes, but the first 512 bytes are reserved for the interpreter
                panic!("Program File too large");
            }
            let mut buffer = vec![0; metadata.len() as usize];
            file.take(metadata.len())
                .read_exact(&mut buffer)
                .expect("buffer overflow");
            buffer.resize(4096, 0);

            prep_buffer(&mut buffer);

            match which.as_str() {
                "dis" => {
                    println!("Disassembling: {}", what);
                    disassembler::disassemble(&buffer);
                }
                "emu" => {
                    println!("Emulating: {}", what);
                    emulator::emulate(&buffer);
                }
                _ => println!("Unknown command"),
            }
        }
        _ => println!("Too few or too many arguments"),
    }
}
