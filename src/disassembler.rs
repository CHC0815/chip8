pub struct Disassembler {
    pub memory: Vec<u8>,
    pub pc: usize,
}

impl Disassembler {
    pub fn new() -> Self {
        Disassembler {
            memory: vec![],
            pc: 0,
        }
    }
    pub fn load(&mut self, program: &[u8]) {
        self.memory = program.to_vec();
    }

    pub fn disassemble(&self) {
        let code = self.memory[self.pc];
        let instr = code >> 4;

        print!("{:04X} {:02X} ", self.pc, code);
        match instr {
            0x06 => {
                let reg = code & 0x0F;
                print!("LEL");
            }
            _ => {
                println!("Unknown instruction: {:02X}", instr);
            }
        }
    }
}
