use ggez::input::keyboard::KeyCode;

pub const SLEEP_MICROS: u64 = 1500;
pub const FONT_BASE_ADDRESS: usize = 0x050;

pub const KEYS: [KeyCode; 16] = [
    KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4,
    KeyCode::Q, KeyCode::W, KeyCode::E, KeyCode::R,
    KeyCode::A, KeyCode::S, KeyCode::D, KeyCode::F,
    KeyCode::Y, KeyCode::X, KeyCode::C, KeyCode::V,
];