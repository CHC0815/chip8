use consts::FONT_BASE_ADDRESS;
use font::FONT;

pub mod consts;
pub mod disassembler;
pub mod emulator;
pub mod font;

pub fn prep_buffer(buffer: &mut Vec<u8>) {
    for i in (0..buffer.len() - 0x200).rev() {
        buffer.swap(i, i + 512);
    }
    for i in 0..0x200 {
        buffer[i] = 0; // Fill the first 512 bytes with 0
    }
    // copy font to 050-09F
    buffer[FONT_BASE_ADDRESS..FONT_BASE_ADDRESS + 80].copy_from_slice(&FONT);
}

#[cfg(test)]
mod test;
