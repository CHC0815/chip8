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

pub struct Emulator {
    pub memory: [u8; 4096],
    pub graphics: Arc<Mutex<Graphics>>,
    pub pc: usize,
    stack: Vec<u16>,
    instruction: u16,
    instr: u16,
    X: u16,
    Y: u16,
    N: u16,
    NN: u16,
    NNN: u16,
}
impl Emulator {
    pub fn new(graphics: Arc<Mutex<Graphics>>) -> Self {
        Emulator {
            memory: [0; 4096],
            graphics: graphics,
            pc: 0,
            stack: Vec::new(),
            instruction: 0,
            instr: 0,
            X: 0,
            Y: 0,
            N: 0,
            NN: 0,
            NNN: 0,

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
        self.X = ((self.instruction & 0x0F00) >> 8) as u16;
        self.Y = ((self.instruction & 0x00F0) >> 4) as u16;
        self.N = (self.instruction & 0x000F) as u16;
        self.NN = (self.instruction & 0x00FF) as u16;
        self.NNN = self.instruction & 0x0FFF as u16;
    }
    fn execute(&mut self) {
        match self.instr {
            0x0 => {
                match self.NNN {
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
                self.pc = self.NNN as usize;
            }
            0x2 => {
                // call subroutine
                self.stack.push(self.pc as u16);
                self.pc = self.NNN as usize;
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
