pub fn disassemble(program: &[u8]) {
    let mut disassembler = Disassembler::new();
    disassembler.load(program);
    disassembler.disassemble();
}

pub struct Disassembler {
    pub memory: [u8; 4096],
}

impl Disassembler {
    pub fn new() -> Disassembler {
        Disassembler { memory: [0; 4096] }
    }
    pub fn load(&mut self, program: &[u8]) {
        self.memory = program.try_into().expect("Program should be 4096 bytes");
    }
    pub fn disassemble(&self) {
        for i in (512..4096).step_by(2) {
            let opcode = (self.memory[i] as u16) << 8 | self.memory[i + 1] as u16;
            if opcode == 0x0000 {
                continue;
            }
            println!(
                "{:04X}: {:04X} {}",
                i - 512,
                opcode,
                self.disassemble_opcode(opcode)
            );
        }
    }
    pub fn disassemble_opcode(&self, opcode: u16) -> String {
        let instr = (opcode & 0xF000) >> 12;
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        let n = opcode & 0x000F;
        let nn = opcode & 0x00FF;
        let nnn = opcode & 0x0FFF;
        match instr {
            0x0 => {
                match nnn {
                    0x0E0 => {
                        // clear screen
                        "clear screen".to_owned()
                    }
                    0x0EE => {
                        // return from subroutine
                        "return from subroutine".to_owned()
                    }
                    _ => "unknown".to_owned(),
                }
            }
            0x1 => {
                // jump
                format!("jump to 0x{:03X}", nnn)
            }
            0x2 => {
                // call subroutine
                format!("call subroutine at 0x{:03X}", nnn)
            }
            0x3 => {
                // skip next instruction if Vx == nn
                format!("skip next instruction if V{:X} == 0x{:02x}", x, nn)
            }
            0x4 => {
                // skip next instruction if Vx != nn
                format!("skip next instruction if V{:X} != 0x{:02x}", x, nn)
            }
            0x5 => {
                // skip next instruction if Vx == Vy
                format!("skip next instruction if V{:X} == V{:X}", x, y)
            }
            0x6 => {
                // set Vx to nn
                format!("set V{:X} to 0x{:02x}", x, nn)
            }
            0x7 => {
                // add nn to Vx
                format!("add 0x{:02x} to V{:X}", nn, x)
            }
            0x8 => {
                match (opcode & 0x000F) as u8 {
                    0x0 => {
                        // set Vx to Vy
                        format!("set V{:X} to V{:X}", x, y)
                    }
                    0x1 => {
                        // set Vx to Vx | Vy
                        format!("set V{:X} to V{:X} | V{:X}", x, x, y)
                    }
                    0x2 => {
                        // set Vx to Vx & Vy
                        format!("set V{:X} to V{:X} & V{:X}", x, x, y)
                    }
                    0x3 => {
                        // set Vx to Vx ^ Vy
                        format!("set V{:X} to V{:X} ^ V{:X}", x, x, y)
                    }
                    0x4 => {
                        // add Vx to Vy
                        format!("add V{:X} to V{:X}", x, y)
                    }
                    0x5 => {
                        // subtract Vy from Vx
                        format!("subtract V{:X} from V{:X}", y, x)
                    }
                    0x6 => {
                        // TODO: configure this to be optional
                        format!("shift V{:X} right", x)
                    }
                    0x7 => {
                        // subtract Vx from Vy
                        format!("subtract V{:X} from V{:X}", x, y)
                    }
                    0xE => {
                        //TODO: configure this to be optional
                        format!("shift V{:X} left", x)
                    }
                    _ => "unknown".to_owned(),
                }
            }
            0x9 => {
                // skip next instruction if Vx != Vy
                format!("skip next instruction if V{:X} != V{:X}", x, y)
            }
            0xA => {
                // set index to nnn
                format!("set index to 0x{:03X}", nnn)
            }
            0xB => {
                // TODO: configure this to be optional: BXNN -> jump to XNN + VX
                // jump to nnn + V0
                format!("jump to 0x{:03X} + V0", nnn)
            }
            0xC => {
                // set Vx to random number & nn
                format!("set V{:X} to random number & 0x{:02x}", x, nn)
            }
            0xD => {
                // display
                format!("display at V{:X}, V{:X}, 0x{:X}", x, y, n)
            }
            0xF => {
                match nn {
                    0x07 => {
                        // set Vx to delay timer
                        format!("set V{:X} to delay timer", x)
                    }
                    0x15 => {
                        // set delay timer to Vx
                        format!("set delay timer to V{:X}", x)
                    }
                    0x18 => {
                        // set sound timer to Vx
                        format!("set sound timer to V{:X}", x)
                    }
                    0x1E => {
                        // add Vx to index
                        // TODO: VF is set to 1 when there is a range overflow (I + Vx > 0xFFF)
                        format!("add V{:X} to index", x)
                    }
                    0x0A => {
                        format!("wait for key press and store in V{:X}", x)
                    }
                    0x29 => {
                        // set index to location of sprite for digit Vx
                        format!("set index to location of sprite for digit V{:X}", x)
                    }
                    0x33 => {
                        // store BCD representation of Vx in memory locations I, I+1, I+2
                        format!(
                            "store BCD representation of V{:X} in memory locations I, I+1, I+2",
                            x
                        )
                    }
                    0x55 => {
                        // store V0 to Vx in memory starting at I
                        format!("store V0 to V{:X} in memory starting at I", x)
                    }
                    0x65 => {
                        // fill V0 to Vx with memory starting at I
                        format!("fill V0 to V{:X} with memory starting at I", x)
                    }
                    _ => "unknown".to_owned(),
                }
            }
            _ => "unknown".to_owned(),
        }
    }
}
