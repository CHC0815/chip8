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

#[test]
fn test_opcode_6_x_nn() {
    // test 6XNN opcode (set Vx = NN)
    let program = create_buffer(vec![0x60, 0x01, 0x61, 0x02, 0x62, 0x03]);
    let mut emulator = Emulator::new();
    emulator.load(&program);

    emulator.run(None);
    assert!(
        emulator.registers[0].v == 0x01,
        "V0 should be 0x1 after opcode 0x6001"
    );
    assert!(
        emulator.registers[1].v == 0x00,
        "V1 should be 0x0 after opcode 0x6102"
    );
    emulator.run(None);
    emulator.run(None);
    assert!(
        emulator.registers[2].v == 0x03,
        "V2 should be 0x3 after opcode 0x6203"
    );
}

#[test]
fn test_opcode_7_x_nn() {
    // test 7XNN opcode (set Vx = Vx + NN)
    let program = create_buffer(vec![0x70, 0x02, 0x70, 0xFF]);
    let mut emulator = Emulator::new();
    emulator.load(&program);

    assert!(
        emulator.registers[0].v == 0x00,
        "V0 should be 0x0 before opcode 0x7002"
    );
    emulator.run(None);
    assert!(
        emulator.registers[0].v == 0x02,
        "V0 should be 0x2 after opcode 0x7002"
    );
    emulator.run(None);
    assert!(
        emulator.registers[0].v == 0x01,
        "V1 should be 0x1 after opcode 0x71FF"
    );
    assert!(
        emulator.registers[0xF].v == 0x00,
        "VF should be 0x0 after opcode 0x71FF"
    )
}

#[test]
fn test_opcode_f_x_33() {
    // test FX33 opcode (store BCD representation of Vx in memory locations I, I+1, I+2)
    let program = create_buffer(vec![0x60, 0x12, 0xF0, 0x33]);
    let mut emulator = Emulator::new();
    emulator.load(&program);

    emulator.run(None); // load 0x12 (18) into V0
    emulator.run(None); // store BCD representation of V0 in memory locations I, I+1, I+2
    assert!(
        emulator.memory[emulator.index as usize] == 0,
        "Memory at I should be 0x0 after opcode 0xF033 {:?}",
        emulator.index
    );
    assert!(
        emulator.memory[emulator.index as usize + 1] == 1,
        "Memory at I+1 should be 0x1 after opcode 0xF033"
    );
    assert!(
        emulator.memory[emulator.index as usize + 2] == 8,
        "Memory at I+2 should be 0x2 after opcode 0xF033 {:?}, 0x{:0X?} 0x{:0X?} 0x{:0X?}",
        emulator.index,
        emulator.memory[emulator.index as usize + 0],
        emulator.memory[emulator.index as usize + 1],
        emulator.memory[emulator.index as usize + 2]
    );
}
