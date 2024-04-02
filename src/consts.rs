use sdl2::keyboard::Keycode;

pub const SLEEP_MICROS: u64 = 1500;
pub const FONT_BASE_ADDRESS: usize = 0x050;

pub const KEYS: [Keycode; 16] = [
    Keycode::Num1,
    Keycode::Num2,
    Keycode::Num3,
    Keycode::Num4,
    Keycode::Q,
    Keycode::W,
    Keycode::E,
    Keycode::R,
    Keycode::A,
    Keycode::S,
    Keycode::D,
    Keycode::F,
    Keycode::Y,
    Keycode::X,
    Keycode::C,
    Keycode::V,
];
