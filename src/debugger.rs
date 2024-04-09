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
    _timer: &sdl2::TimerSubsystem,
    emulator: &mut Emulator,
) -> bool {
    let _next: u64 = 0;
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
    false
}

#[derive(Debug, Eq, PartialEq)]
enum State {
    Running(Option<u32>),
    Paused,
    HitBreackpoint,
    Stopped,
}

pub struct Debugger {
    state: State,
    breakpoints: Vec<u16>,
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger {
            state: State::Stopped,
            breakpoints: Vec::new(),
        }
    }
    pub fn attach(
        &mut self,
        emulator: &mut Emulator,
        display: &mut Display,
        event_pump: &mut sdl2::EventPump,
        timer: &sdl2::TimerSubsystem,
    ) {
        let mut skip = false;
        self.state = State::Paused;
        loop {
            if handle_loop(event_pump, timer, emulator) {
                break;
            }
            if !skip
                && self.state != State::HitBreackpoint
                && self.state != State::Paused
                && self.state != State::Stopped
            {
                if self.breakpoints.contains(&(emulator.pc as u16)) {
                    self.state = State::HitBreackpoint;
                }
            } else {
                skip = false;
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
                    if input.is_empty() {
                        continue;
                    }
                    let cmd = input.chars().next().unwrap();
                    match cmd {
                        's' => {
                            let n = input[1..].trim().parse::<u32>().unwrap_or(1);
                            self.state = State::Running(Some(n));
                            skip = true;
                        }
                        'c' => {
                            self.state = State::Running(None);
                            skip = true;
                        }
                        'b' => {
                            let addr = u16::from_str_radix(input[1..].trim(), 16);
                            if let Ok(addr) = addr {
                                self.breakpoints.push(addr);
                            } else {
                                println!("Invalid address");
                            }
                        }
                        'd' => {
                            let addr = u16::from_str_radix(input[1..].trim(), 16);
                            if let Ok(addr) = addr {
                                self.breakpoints.retain(|&x| x != addr);
                            } else {
                                println!("Invalid address");
                            }
                        }
                        'l' => {
                            for (i, &addr) in self.breakpoints.iter().enumerate() {
                                println!("{}: 0x{:x}", i, addr);
                            }
                        }
                        'h' => {
                            println!("--------------- HELP ---------------");
                            println!("s        - step for 1 instruction");
                            println!("s [n]    - step for n instructions");
                            println!("b [addr] - add breakpoint at addr");
                            println!("d [addr] - delete breakpoint at addr");
                            println!("l        - list breakpoints");
                            println!("c        - continue");
                            println!("q        - quit");
                        }
                        'q' => {
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
