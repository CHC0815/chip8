use std::io::Write;

use sdl2::{event::Event, keyboard::Keycode};

use crate::{
    consts::KEYS,
    emulator::{Display, Emulator},
};

pub fn debug(program: &[u8]) {
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

    let mut debugger = Debugger::new();
    debugger.attach(&mut emulator, &mut display, &mut event_pump, &timer);
}
fn handle_loop(
    event_pump: &mut sdl2::EventPump,
    timer: &sdl2::TimerSubsystem,
    emulator: &mut Emulator,
) -> bool {
    let mut before;
    let mut next: u64 = 0;
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => return true,
            Event::KeyDown {
                keycode: Some(keycode),
                ..
            } => {
                let key = KEYS.iter().position(|&x| x == keycode);
                if let Some(k) = key {
                    emulator.key_buffer.key = Some(k as u8);
                }
            }
            Event::KeyUp {
                keycode: Some(keycode),
                ..
            } => {
                let key = KEYS.iter().position(|&x| x == keycode);
                if let Some(key) = key {
                    if let Some(k) = emulator.key_buffer.key {
                        if k == key as u8 {
                            emulator.key_buffer.key = None;
                        }
                    }
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
    false
}

enum State {
    Running(Option<u32>),
    Paused,
    HitBreackpoint,
    Stopped,
}

pub struct Debugger {
    state: State,
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {
            state: State::Stopped,
        }
    }
    pub fn attach(
        &mut self,
        emulator: &mut Emulator,
        display: &mut Display,
        event_pump: &mut sdl2::EventPump,
        timer: &sdl2::TimerSubsystem,
    ) {
        self.state = State::Paused;
        loop {
            if handle_loop(event_pump, timer, emulator) {
                break;
            }
            match self.state {
                State::Running(None) => {
                    emulator.run(Some(display));
                }
                State::Running(Some(0)) => {
                    self.state = State::Paused;
                }
                State::Running(Some(n)) => {
                    self.state = State::Running(Some(n - 1));
                    emulator.run(Some(display));
                }
                State::Paused => {
                    print!(">");
                    std::io::stdout().flush().unwrap();
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    let input = input.trim();
                    match input {
                        "s" => {
                            self.state = State::Running(Some(1));
                        }
                        "c" => {
                            self.state = State::Running(None);
                        }
                        "q" => {
                            self.state = State::Stopped;
                        }
                        _ => {
                            println!("Unknown command");
                        }
                    }
                }
                State::Stopped => {
                    break;
                }
                State::HitBreackpoint => {
                    println!("Hit breakpoint at 0x{:x}", emulator.pc);
                    self.state = State::Paused;
                }
            }
        }
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}
