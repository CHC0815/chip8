use ggez::{
    conf::{self},
    event::{self, EventHandler},
    graphics::{self, Color},
    Context, ContextBuilder,
};

pub fn emulate(program: &[u8]) {
    let mut c = conf::Conf::new();
    c.window_mode.width = 320.0;
    c.window_mode.height = 640.0;
    c.window_setup.title = "CHIP 8 Emulator".to_string();
    c.window_setup.vsync = false;

    let (mut ctx, event_loop) = ContextBuilder::new("CHIP8", "Conrad H. Carl")
        .default_conf(c)
        .build()
        .expect("Could not create ggez context.");

    let mut emulator = Emulator::new(&mut ctx);
    emulator.load(program);

    event::run(ctx, event_loop, emulator);
}

pub struct Emulator {
    pub memory: [u8; 4096],
}
impl Emulator {
    pub fn new(_ctx: &mut Context) -> Self {
        Emulator { memory: [0; 4096] }
    }
    pub fn load(&mut self, program: &[u8]) {
        self.memory = program.try_into().expect("Program should be 4096 bytes");
    }
    pub fn run(&mut self) {}
}

impl EventHandler for Emulator {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), ggez::GameError> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), ggez::GameError> {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        canvas.finish(ctx)
    }
}
