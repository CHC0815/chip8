use ggez::event::EventHandler;

pub fn emulate(what: &str) {}

pub struct Emulator {}
impl Emulator {
    pub fn new() -> Self {
        Emulator {}
    }
    pub fn load(&mut self, program: &[u8]) {}
    pub fn run(&mut self) {}
}

impl EventHandler for Emulator {
    fn update(&mut self, _ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        todo!()
    }

    fn draw(&mut self, _ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        todo!()
    }
}
