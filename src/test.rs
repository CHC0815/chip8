use super::emulator::Emulator;
use super::prep_buffer;

fn create_buffer(program: Vec<u8>) -> Vec<u8> {
    let mut buffer = vec![0u8; 4096];
    buffer[0..program.len()].copy_from_slice(&program);
    prep_buffer(&mut buffer);
    buffer
}

#[test]
fn test_opcode_1_nnn() {
    // test 1NNN opcode (jump to address NNN)
    let program = create_buffer(vec![0x12, 0x34]);
    let mut emulator = Emulator::new();
    emulator.load(&program);

    assert!(emulator.pc == 0x200, "PC should initially be 0x200");
    emulator.run(None);
    assert!(
        emulator.pc == 0x234,
        "PC should be 0x234 after opcode 0x1234"
    );
}

#[test]
fn test_opcode_3_x_nn_pos() {
    // test 2XNN opcode (skip if Vx == NN)
    let program = create_buffer(vec![0x30, 0x00, 0xFF, 0xFF]);
    let mut emulator = Emulator::new();
    emulator.load(&program);

    assert!(emulator.pc == 0x200, "PC should initially be 0x200");
    emulator.run(None);
    assert!(
        emulator.pc == 0x204,
        "PC should be 0x204 after opcode 0x3000"
    );
}

#[test]
fn test_opcode_3_x_nn_neg() {
    // test 2XNN opcode (skip if Vx == NN)
    let program = create_buffer(vec![0x30, 0x01, 0xFF, 0xFF]);
    let mut emulator = Emulator::new();
    emulator.load(&program);

    assert!(emulator.pc == 0x200, "PC should initially be 0x200");
    emulator.run(None);
    assert!(
        emulator.pc == 0x202,
        "PC should be 0x202 after opcode 0x3001"
    );
}
