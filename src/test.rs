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

#[test]
fn test_opcode_4_x_nn_pos() {
    // test 4XNN opcode (skip if Vx != NN)
    let program = create_buffer(vec![0x40, 0x01, 0xFF, 0xFF]);
    let mut emulator = Emulator::new();
    emulator.load(&program);

    assert!(emulator.pc == 0x200, "PC should initially be 0x200");
    emulator.run(None);
    assert!(
        emulator.pc == 0x204,
        "PC should be 0x204 after opcode 0x4001"
    );
}

#[test]
fn test_opcode_4_x_nn_neg() {
    // test 4XNN opcode (skip if Vx != NN)
    let program = create_buffer(vec![0x40, 0x00, 0xFF, 0xFF]);
    let mut emulator = Emulator::new();
    emulator.load(&program);

    assert!(emulator.pc == 0x200, "PC should initially be 0x200");
    emulator.run(None);
    assert!(
        emulator.pc == 0x202,
        "PC should be 0x202 after opcode 0x4000"
    );
}

#[test]
fn test_opcode_5_x_y_pos() {
    // test 5XY0 opcode (skip if Vx == Vy)
    let program = create_buffer(vec![0x51, 0x20, 0xFF, 0xFF, 0x53, 0x40]); // V1 == V2 and V3 == V4
    let mut emulator = Emulator::new();
    emulator.load(&program);

    assert!(emulator.pc == 0x200, "PC should initially be 0x200");
    emulator.run(None);
    assert!(
        emulator.pc == 0x204,
        "PC should be 0x204 after opcode 0x5120"
    );
    emulator.registers[3].v = 0x4;
    emulator.registers[4].v = 0x4;
    emulator.run(None);
    assert!(
        emulator.pc == 0x208,
        "PC should be 0x208 after opcode 0x5340"
    );
}

#[test]
fn test_opcode_5_x_y_neg() {
    // test 5XY0 opcode (skip if Vx == Vy)
    let program = create_buffer(vec![0x51, 0x20, 0x53, 0x40]); // V1 == V2 and V3 == V4
    let mut emulator = Emulator::new();
    emulator.load(&program);

    assert!(emulator.pc == 0x200, "PC should initially be 0x200");
    emulator.registers[1].v = 0x1;
    emulator.run(None);
    assert!(
        emulator.pc == 0x202,
        "PC should be 0x202 after opcode 0x5120"
    );
    emulator.registers[3].v = 0x4;
    emulator.registers[4].v = 0x5;
    emulator.run(None);
    assert!(
        emulator.pc == 0x204,
        "PC should be 0x204 after opcode 0x5340"
    );
}
