use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use ggez::{
    conf::{self},
    event::{self, EventHandler},
    graphics::{self, Color},
    Context, ContextBuilder,
};

use oorandom::Rand32;

#[derive(Clone)]
pub struct Graphics {
    buffer: [u8; 64 * 32],
}
impl Graphics {
    pub fn new() -> Self {
        Graphics {
            buffer: [0; 64 * 32],
        }
    }
}

pub fn emulate(program: &[u8]) {
    //setup game engine
    let mut c = conf::Conf::new();
    c.window_mode.width = 320.0;
    c.window_mode.height = 640.0;
    c.window_setup.title = "CHIP 8 Emulator".to_string();
    c.window_setup.vsync = true;

    let (mut ctx, event_loop) = ContextBuilder::new("CHIP8", "Conrad H. Carl")
        .default_conf(c)
        .build()
        .expect("Could not create ggez context.");

    let graphics_buffer = Arc::new(Mutex::new(Graphics::new()));

    let app = EmulatorApp::new(&mut ctx, graphics_buffer.clone());

    let _emu_thread = {
        let mut emulator = Emulator::new(graphics_buffer.clone());
        emulator.load(program);
        thread::spawn(move || {
            emulator.run();
        });
    };

    event::run(ctx, event_loop, app);
}

#[derive(Clone, Copy, Debug)]
pub struct Register {
    pub v: u8,
}
pub struct Emulator {
    pub memory: [u8; 4096],
    pub graphics: Arc<Mutex<Graphics>>,
    local_graphics: Graphics,
    pub pc: usize,
    stack: Vec<u16>,
    registers: [Register; 16],
    index: u16,
    instruction: u16,
    instr: u16,
    x: u16,
    y: u16,
    n: u16,
    nn: u16,
    nnn: u16,
}
impl Emulator {
    pub fn new(graphics: Arc<Mutex<Graphics>>) -> Self {
        Emulator {
            memory: [0; 4096],
            graphics: graphics,
            local_graphics: Graphics::new(),
            pc: 0,
            stack: Vec::new(),
            registers: [Register { v: 0 }; 16],
            index: 0,
            instruction: 0,
            instr: 0,
            x: 0,
            y: 0,
            n: 0,
            nn: 0,
            nnn: 0,

        }
    }
    pub fn load(&mut self, program: &[u8]) {
        self.memory = program.try_into().expect("Program should be 4096 bytes");
    }
    fn fetch(&mut self){
        if self.pc >= 4096 {
            panic!("Program counter out of bounds");
        }
        let first_byte = self.memory[self.pc];
        let second_byte = self.memory[self.pc + 1];
        self.pc += 2;
        self.instruction = u16::from_be_bytes([first_byte, second_byte]);
    }
    fn decode(&mut self) {
        self.instr = self.instruction & 0xF000 >> 12;
        self.x = ((self.instruction & 0x0F00) >> 8) as u16;
        self.y = ((self.instruction & 0x00F0) >> 4) as u16;
        self.n = (self.instruction & 0x000F) as u16;
        self.nn = (self.instruction & 0x00FF) as u16;
        self.nnn = self.instruction & 0x0FFF as u16;
    }
    fn execute(&mut self) {
        match self.instr {
            0x0 => {
                match self.nnn {
                    0x0E0 => {
                        // clear screen
                        self.clear_screen();
                    }
                    0x0EE => {
                        // return from subroutine
                        self.pc = self.stack.pop().unwrap() as usize;
                    }
                    _ => {
                        println!("Unknown instruction: {:x}", self.instruction);
                    }
                }
            }
            0x1 => {
                // jump
                self.pc = self.nnn as usize;
            }
            0x2 => {
                // call subroutine
                self.stack.push(self.pc as u16);
                self.pc = self.nnn as usize;
            }
            0x3 => {
                // skip next instruction if Vx == nn
                if self.registers[self.x as usize].v == self.nn as u8 {
                    self.pc += 2;
                }
            }
            0x4 => {
                // skip next instruction if Vx != nn
                if self.registers[self.x as usize].v != self.nn as u8 {
                    self.pc += 2;
                }
            }
            0x5 => {
                // skip next instruction if Vx == Vy
                if self.registers[self.x as usize].v == self.registers[self.y as usize].v {
                    self.pc += 2;
                }
            }
            0x6 => {
                // set Vx to nn
                self.registers[self.x as usize].v = self.nn as u8;
            }
            0x7 => {
                self.registers[self.x as usize].v += self.nn as u8;
            }
            0x8 => {
                match (self.instruction & 0x000F) as u8 {
                    0x0 => {
                        // set Vx to Vy
                        self.registers[self.x as usize].v = self.registers[self.y as usize].v;
                    }
                    0x1 => {
                        // set Vx to Vx | Vy
                        self.registers[self.x as usize].v |= self.registers[self.y as usize].v;
                    }
                    0x2 => {
                        // set Vx to Vx & Vy
                        self.registers[self.x as usize].v &= self.registers[self.y as usize].v;
                    }
                    0x3 => {
                        // set Vx to Vx ^ Vy
                        self.registers[self.x as usize].v ^= self.registers[self.y as usize].v;
                    }
                    0x4 => {
                        // add Vx to Vy
                        let result = self.registers[self.x as usize].v as u16 + self.registers[self.y as usize].v as u16;
                        self.registers[self.x as usize].v = result as u8;
                        self.registers[0xF].v = if result > 0xFF { 1 } else { 0 };
                    }
                    0x5 => {
                        // subtract Vy from Vx
                        let x = self.registers[self.x as usize].v;
                        let y = self.registers[self.y as usize].v;
                        self.registers[self.x as usize].v = x.wrapping_sub(y);
                        self.registers[0xF].v = if x > y { 1 } else { 0 };
                    }
                    0x6 => {
                        // TODO: configure this to be optional
                        self.registers[self.x as usize].v = self.registers[self.y as usize].v;
                        let flag = self.registers[self.x as usize].v & 0x1;
                        self.registers[self.x as usize].v >>= 1;
                        self.registers[0xF].v = flag;
                    }
                    0x7 => {
                        // subtract Vx from Vy
                        let x = self.registers[self.x as usize].v;
                        let y = self.registers[self.y as usize].v;
                        self.registers[self.x as usize].v = y.wrapping_sub(x);
                        self.registers[0xF].v = if y > x { 1 } else { 0 };
                    }
                    0xE => {
                        //TODO: configure this to be optional
                        self.registers[self.x as usize].v = self.registers[self.y as usize].v;
                        let flag = (self.registers[self.x as usize].v & 0x80) >> 7;
                        self.registers[self.x as usize].v <<= 1;
                        self.registers[0xF].v = flag;
                    }
                    _ => {
                        println!("Unknown instruction: {:x}", self.instruction);
                    } 
                }
            }
            0x9 => {
                // skip next instruction if Vx != Vy
                if self.registers[self.x as usize].v != self.registers[self.y as usize].v {
                    self.pc += 2;
                }
            }
            0xA => {
                // set index to nnn
                self.index = self.nnn;
            }
            0xB => {
                // TODO: configure this to be optional: BXNN -> jump to XNN + VX
                // jump to nnn + V0
                self.pc = self.nnn as usize + self.registers[0].v as usize;
            }
            0xC => {
                // set Vx to random number & nn
                let mut rng = Rand32::new(0);
                self.registers[self.x as usize].v = rng.rand_u32() as u8 & self.nn as u8;
            }
            0xD => {
                // display
                self.local_graphics = self.graphics.lock().unwrap().clone();
                let mut x = self.registers[self.x as usize].v as usize % 64;
                let mut y = self.registers[self.y as usize].v as usize % 32;
                self.registers[0xF].v = 0;
                for i in 0..self.n {
                    let sprite = self.memory[self.index as usize + i as usize];
                    for i in 0..8 {
                        let pixel = (sprite >> (7 - i)) & 0x1;
                        let current = self.local_graphics.buffer[(x + i) + (y + i) * 32];
                        if pixel == 1 && current == 1 {
                            self.local_graphics.buffer[(x + i) + (y + i) * 32] = 0;
                            self.registers[0xF].v = 1;
                        }
                        if pixel == 1 && current == 0 {
                            self.local_graphics.buffer[(x + i) + (y + i) * 32] = 1;
                        }
                        if y + i >= 32 {
                            break;
                        }
                        x += 1;
                    }
                    y += 1;
                    if y >= 32 {
                        break;
                    }
                }

                self.graphics.lock().unwrap().buffer = self.local_graphics.buffer;
            }
            _ => {
                println!("Unknown instruction: {:x}", self.instruction);
            }
        }
    }
    pub fn run(&mut self) {
        loop {
            self.fetch();
            self.decode();
            self.execute();
            // thread::sleep(Duration::from_millis(1));
        }
    }
    pub fn set_pixel(&mut self, x: usize, y: usize, value: u8) {
        let mut state = self.graphics.lock().unwrap();
        state.buffer[x + y * 32] = value;
    }
    pub fn clear_screen(&mut self) {
        let mut state = self.graphics.lock().unwrap();
        state.buffer = [0; 64 * 32];
    }
}

struct EmulatorApp {
    graphics: Arc<Mutex<Graphics>>,
    local_graphics: Graphics,
}

impl EmulatorApp {
    pub fn new(_ctx: &mut Context, graphics: Arc<Mutex<Graphics>>) -> Self {
        EmulatorApp {
            graphics,
            local_graphics: Graphics::new(),
        }
    }
}

impl EventHandler for EmulatorApp {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), ggez::GameError> {
        self.local_graphics = self.graphics.lock().unwrap().clone();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), ggez::GameError> {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        let mut rect = graphics::Rect::new(0 as f32 * 10.0, 0 as f32 * 10.0, 10.0, 10.0);
        let mut mesh =
            graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, Color::BLACK)?;

        for (i, pixel) in self.local_graphics.buffer.iter().enumerate() {
            let x = i % 32;
            let y = i / 32;
            let color = if *pixel == 0 {
                Color::BLACK
            } else {
                Color::WHITE
            };
            rect.x = x as f32 * 10.0;
            rect.y = y as f32 * 10.0;
            mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, color)?;
            canvas.draw(&mesh, graphics::DrawParam::default());
        }
        canvas.finish(ctx)
    }
}
