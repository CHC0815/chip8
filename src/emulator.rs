use oorandom::Rand32;
use sdl2::{event::Event, keyboard::Keycode, render::Canvas, video::Window, Sdl};

use std::{thread, time::Duration};

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

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}
pub fn emulate(program: &[u8]) {
    let sdl_context = sdl2::init().expect("sdl2 should initialize");

    let timer = sdl_context
        .timer()
        .expect("sdl2 context should have a timer");
    let mut display = Display::new(&sdl_context);
    let mut event_pump = sdl_context
        .event_pump()
        .expect("sdl2 context should have an event pump");

    let mut emulator = Emulator::new();
    emulator.load(program);
    display.canvas.present();

    let mut before = timer.ticks64();
    let mut next: u64 = 0;
    'run: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'run,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    let key = KEYS.iter().position(|&x| x == keycode);
                    match key {
                        Some(k) => {
                            emulator.key_buffer.key = Some(k as u8);
                        }
                        None => {}
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    let key = KEYS.iter().position(|&x| x == keycode);
                    match key {
                        Some(key) => {
                            if let Some(k) = emulator.key_buffer.key {
                                if k == key as u8 {
                                    emulator.key_buffer.key = None;
                                }
                            }
                        }
                        None => {}
                    }
                }
                _ => {}
            }
        }
        before = timer.ticks64();
        if before >= next {
            if emulator.delay_timer > 0 {
                println!("Delay timer: {}", emulator.delay_timer);
                emulator.delay_timer -= 1;
            }
            if emulator.sound_timer > 0 {
                emulator.sound_timer -= 1;
            }
            next = before + 1000 / 60;
        }
        const STEPS: usize = 2;
        for _ in 0..STEPS {
            emulator.run(Some(&mut display));
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Register {
    pub v: u8,
}

pub struct Display {
    canvas: Canvas<Window>,
}

impl Display {
    pub fn new(context: &Sdl) -> Self {
        let video_subsystem = context.video().unwrap();
        let window = video_subsystem
            .window("CHIP 8", 640, 320)
            .opengl()
            .position_centered()
            .build()
            .unwrap();
        let canvas = window
            .into_canvas()
            .index(find_sdl_gl_driver().unwrap())
            .build()
            .unwrap();
        Display { canvas: canvas }
    }
    fn draw(&mut self, graphics: &Graphics) {
        for y in 0..32 {
            for x in 0..64 {
                let idx = x + y * 64;
                let color = if graphics.buffer[idx] == 0 { 0 } else { 255 };
                if color == 0 {
                    self.canvas
                        .set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
                } else {
                    self.canvas
                        .set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
                }

                self.canvas
                    .fill_rect(sdl2::rect::Rect::new(x as i32 * 10, y as i32 * 10, 10, 10))
                    .unwrap();
            }
        }
        self.canvas.present();
    }
}

pub struct Emulator {
    pub memory: [u8; 4096],
    graphics: Graphics,
    key_buffer: KeyState,
    pub pc: usize,
    pub stack: Vec<u16>,
    pub registers: [Register; 16],
    pub index: u16,
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
    pub fn new() -> Self {
        Emulator {
            memory: [0; 4096],
            graphics: Graphics::new(),
            key_buffer: KeyState { key: None },
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
    fn execute(&mut self, display: Option<&mut Display>) {
        match self.instr {
            0x0 => {
                match self.nnn {
                    0x0E0 => {
                        // clear screen
                        self.clear_screen();
                        if let Some(display) = display {
                            display.draw(&self.graphics);
                        }
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
                self.registers[self.x as usize].v = self.registers[self.x as usize]
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
                        let old_pixel = self.graphics.buffer[xx + yy * 64] != 0;
                        let pixel = sprite[row] & (1 << col) != 0;
                        let new_pixel = pixel ^ old_pixel;
                        self.graphics.buffer[xx + yy * 64] = if new_pixel { 1 } else { 0 };
                    }
                }
                if let Some(display) = display {
                    display.draw(&self.graphics);
                }
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
                        println!("Waiting for key press");
                        let key_pressed = self.key_buffer.key.is_some();
                        if key_pressed {
                            let key_code = self.key_buffer.key.unwrap();
                            println!("key pressed: {}", key_code);
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
                        for i in 0..=self.x {
                            self.memory[self.index as usize + i as usize] =
                                self.registers[i as usize].v;
                        }
                    }
                    0x65 => {
                        // fill V0 to Vx with memory starting at I
                        for i in 0..=self.x {
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
    pub fn run(&mut self, display: Option<&mut Display>) {
        // let start = std::time::Instant::now();
        self.fetch();
        self.decode();
        self.execute(display);
        thread::sleep(Duration::from_micros(SLEEP_MICROS));
        // println!("Cycle took: {:?}", start.elapsed());
    }
    pub fn clear_screen(&mut self) {
        self.graphics.buffer = [0; 64 * 32];
    }
}
