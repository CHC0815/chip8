use std::{
    sync::{Arc, Mutex},
    thread,
};

use ggez::{
    conf,
    event::{self, EventHandler},
    graphics::{self, Color, Mesh},
    input::keyboard::KeyCode,
    Context, ContextBuilder,
};

use oorandom::Rand32;

use std::time::Duration;

use crate::consts::{FONT_BASE_ADDRESS, KEYS, SLEEP_MICROS};

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

pub struct KeyState {
    pub key: Option<u8>,
}

pub fn emulate(program: &[u8]) {
    //setup game engine
    let mut c = conf::Conf::new();
    c.window_mode.width = 640.0;
    c.window_mode.height = 320.0;
    c.window_setup.title = "CHIP 8 Emulator".to_string();
    c.window_setup.vsync = true;
    c.backend = conf::Backend::Metal;

    let (mut ctx, event_loop) = ContextBuilder::new("CHIP8", "Conrad H. Carl")
        .default_conf(c)
        .build()
        .expect("Could not create ggez context.");

    // RwLock would not be better because we have only one reader
    let graphics_buffer = Arc::new(Mutex::new(Graphics::new()));
    let key_buffer = Arc::new(Mutex::new(KeyState { key: None }));

    let app = EmulatorApp::new(&mut ctx, graphics_buffer.clone(), key_buffer.clone());

    let _emu_thread = {
        let mut emulator = Emulator::new(graphics_buffer.clone(), key_buffer.clone());
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
    pub key_buffer: Arc<Mutex<KeyState>>,
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
    delay_timer: u16,
    sound_timer: u16,
}
impl Emulator {
    pub fn new(graphics: Arc<Mutex<Graphics>>, key_buffer: Arc<Mutex<KeyState>>) -> Self {
        Emulator {
            memory: [0; 4096],
            graphics: graphics,
            key_buffer: key_buffer,
            local_graphics: Graphics::new(),
            pc: 0x200,
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
            delay_timer: 0,
            sound_timer: 0,
        }
    }
    pub fn load(&mut self, program: &[u8]) {
        self.memory = program.try_into().expect("Program should be 4096 bytes");
    }
    fn fetch(&mut self) {
        if self.pc >= 4096 {
            panic!("Program counter out of bounds");
        }
        let first_byte = self.memory[self.pc];
        let second_byte = self.memory[self.pc + 1];
        self.pc += 2;
        self.instruction = ((first_byte as u16) << 8) | second_byte as u16;
    }
    fn decode(&mut self) {
        self.instr = (self.instruction & 0xF000) >> 12;
        self.x = ((self.instruction & 0x0F00) >> 8) as u16;
        self.y = ((self.instruction & 0x00F0) >> 4) as u16;
        self.n = (self.instruction & 0x000F) as u16;
        self.nn = (self.instruction & 0x00FF) as u16;
        self.nnn = self.instruction & 0x0FFF as u16;
        // println!("instr: {:x}, x: {:x}, y: {:x}, n: {:x}, nn: {:x}, nnn: {:x}", self.instr, self.x, self.y, self.n, self.nn, self.nnn);
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
                        panic!(
                            "Unknown instruction: 0x{:04x} at 0x{:04x}",
                            self.instruction,
                            self.pc - 2
                        );
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
                // add nn to Vx
                let _ = self.registers[self.x as usize]
                    .v
                    .wrapping_add(self.nn as u8);
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
                        let result = self.registers[self.x as usize].v as u16
                            + self.registers[self.y as usize].v as u16;
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
                        panic!(
                            "Unknown instruction: 0x{:04x} at 0x{:04x}",
                            self.instruction,
                            self.pc - 2
                        );
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
                let x = self.registers[self.x as usize].v as usize % 64;
                let y = self.registers[self.y as usize].v as usize % 32;
                self.registers[0xF].v = 0;

                let mut sprite = [0u8; 15];
                for s in self.index..(self.index + self.n) {
                    sprite[(s - self.index) as usize] = self.memory[s as usize];
                }

                for row in 0..self.n as usize {
                    let rev = [7, 6, 5, 4, 3, 2, 1, 0];
                    for col in 0..8 {
                        let xx = rev[col] + x;
                        let yy = row + y;
                        let old_pixel = self.local_graphics.buffer[xx + yy * 64] != 0;
                        let pixel = sprite[row] & (1 << col) != 0;
                        let new_pixel = pixel ^ old_pixel;
                        self.local_graphics.buffer[xx + yy * 64] = if new_pixel { 1 } else { 0 };
                    }
                }

                self.graphics.lock().unwrap().buffer = self.local_graphics.buffer;
            }
            0xF => {
                match self.nn {
                    0x07 => {
                        // set Vx to delay timer
                        self.registers[self.x as usize].v = self.delay_timer as u8;
                    }
                    0x15 => {
                        // set delay timer to Vx
                        self.delay_timer = self.registers[self.x as usize].v as u16;
                    }
                    0x18 => {
                        // set sound timer to Vx
                        self.sound_timer = self.registers[self.x as usize].v as u16;
                    }
                    0x1E => {
                        // add Vx to index
                        self.index += self.registers[self.x as usize].v as u16;
                        // TODO: VF is set to 1 when there is a range overflow (I + Vx > 0xFFF)
                    }
                    0x0A => {
                        let key_pressed = self.key_buffer.lock().unwrap().key.is_some();
                        if key_pressed {
                            let key_code = self.key_buffer.lock().unwrap().key.unwrap();
                            self.registers[self.x as usize].v = key_code;
                        } else {
                            self.pc -= 2;
                        }
                    }
                    0x29 => {
                        // set index to location of sprite for digit Vx
                        self.index =
                            self.registers[self.x as usize].v as u16 * 5 + FONT_BASE_ADDRESS as u16;
                    }
                    0x33 => {
                        // store BCD representation of Vx in memory locations I, I+1, I+2
                        let value = self.registers[self.x as usize].v;
                        self.memory[self.index as usize] = value / 100;
                        self.memory[self.index as usize + 1] = (value / 10) % 10;
                        self.memory[self.index as usize + 2] = value % 10;
                    }
                    0x55 => {
                        // store V0 to Vx in memory starting at I
                        for i in 0..self.x {
                            self.memory[self.index as usize + i as usize] =
                                self.registers[i as usize].v;
                        }
                    }
                    0x65 => {
                        // fill V0 to Vx with memory starting at I
                        for i in 0..self.x {
                            self.registers[i as usize].v =
                                self.memory[self.index as usize + i as usize];
                        }
                    }
                    _ => {
                        panic!(
                            "Unknown instruction: 0x{:04x} at 0x{:04x}",
                            self.instruction,
                            self.pc - 2
                        );
                    }
                }
            }
            _ => {
                panic!(
                    "Unknown instruction: 0x{:04x} at 0x{:04x}",
                    self.instruction,
                    self.pc - 2
                );
            }
        }
    }
    pub fn run(&mut self) {
        loop {
            // let start = std::time::Instant::now();
            self.fetch();
            self.decode();
            self.execute();
            thread::sleep(Duration::from_micros(SLEEP_MICROS));
            // println!("Cycle took: {:?}", start.elapsed());
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
    keybuffer: Arc<Mutex<KeyState>>,
    local_graphics: Graphics,
}

impl EmulatorApp {
    pub fn new(
        _ctx: &mut Context,
        graphics: Arc<Mutex<Graphics>>,
        key_buffer: Arc<Mutex<KeyState>>,
    ) -> Self {
        EmulatorApp {
            graphics,
            keybuffer: key_buffer,
            local_graphics: Graphics::new(),
        }
    }
}

impl EventHandler for EmulatorApp {
    fn update(&mut self, ctx: &mut Context) -> Result<(), ggez::GameError> {
        let pressed_key = { KEYS.iter().find(|key| ctx.keyboard.is_key_pressed(**key)) };

        if let Some(key) = pressed_key {
            let mut key_buffer = self.keybuffer.lock().unwrap();
            key_buffer.key = Some(key_index(key.to_owned()));
        } else {
            let mut key_buffer = self.keybuffer.lock().unwrap();
            key_buffer.key = None;
        }

        self.local_graphics = self.graphics.lock().unwrap().clone();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), ggez::GameError> {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        let mut rect = graphics::Rect::new(0 as f32 * 10.0, 0 as f32 * 10.0, 10.0, 10.0);
        let mut mesh: Mesh;

        for (i, pixel) in self.local_graphics.buffer.iter().enumerate() {
            let x = i % 64;
            let y = i / 64;
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

fn key_index(key: KeyCode) -> u8 {
    match key {
        KeyCode::Key1 => 0x0,
        KeyCode::Key2 => 0x1,
        KeyCode::Key3 => 0x2,
        KeyCode::Key4 => 0x3,
        KeyCode::Q => 0x4,
        KeyCode::W => 0x5,
        KeyCode::E => 0x6,
        KeyCode::R => 0x7,
        KeyCode::A => 0x8,
        KeyCode::S => 0x9,
        KeyCode::D => 0xA,
        KeyCode::F => 0xB,
        KeyCode::Y => 0xC, // german layout
        KeyCode::X => 0xD,
        KeyCode::C => 0xE,
        KeyCode::V => 0xF,

        _ => panic!("Unknown key pressed: {:?}", key),
    }
}
