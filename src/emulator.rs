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
            //emulator.run();
        });
    };

    event::run(ctx, event_loop, app);
}

pub struct Emulator {
    pub memory: [u8; 4096],
    pub graphics: Arc<Mutex<Graphics>>,
}
impl Emulator {
    pub fn new(graphics: Arc<Mutex<Graphics>>) -> Self {
        Emulator {
            memory: [0; 4096],
            graphics,
        }
    }
    pub fn load(&mut self, program: &[u8]) {
        self.memory = program.try_into().expect("Program should be 4096 bytes");
    }
    pub fn run(&mut self) {
        let mut seed: [u8; 8] = [0; 8];
        getrandom::getrandom(&mut seed[..]).expect("Could not create RNG seed");
        let mut rng = Rand32::new(u64::from_ne_bytes(seed));
        loop {
            self.set_pixel(
                rng.rand_range(0..32) as usize,
                rng.rand_range(0..64) as usize,
                1,
            );
            thread::sleep(Duration::from_millis(100));
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
