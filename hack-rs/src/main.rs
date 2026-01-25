//! # hack-rs CLI
//!
//! A simple CLI for testing the Hack computer simulator.
//!
//! Run with: `cargo run`

use hack_rs::gates;
use hack_rs::arithmetic;
use hack_rs::memory;
use hack_rs::instruction;

fn main() {
    println!("===========================================");
    println!("  hack-rs: Hack Computer Simulator");
    println!("  Built from NAND gates up!");
    println!("===========================================\n");

    // Quick smoke tests for each module
    test_gates();
    test_arithmetic();
    test_memory();
    test_instructions();

    println!("\n===========================================");
    println!("  All smoke tests passed!");
    println!("  Run 'cargo test' for full test suite.");
    println!("===========================================");
}

fn test_gates() {
    print!("Testing gates module... ");

    // Basic gates
    assert_eq!(gates::nand(true, true), false);
    assert_eq!(gates::and(true, true), true);
    assert_eq!(gates::or(false, true), true);
    assert_eq!(gates::not(true), false);
    assert_eq!(gates::xor(true, true), false);

    // Mux/Dmux
    assert_eq!(gates::mux(true, false, false), true);
    assert_eq!(gates::dmux(true, false), (true, false));

    // 16-bit
    assert_eq!(gates::not16(0x0000), 0xFFFF);
    assert_eq!(gates::and16(0xFFFF, 0x0F0F), 0x0F0F);

    println!("OK");
}

fn test_arithmetic() {
    print!("Testing arithmetic module... ");

    // Adders
    assert_eq!(arithmetic::half_adder(true, true), (false, true));
    assert_eq!(arithmetic::full_adder(true, true, true), (true, true));
    assert_eq!(arithmetic::add16(100, 200), 300);
    assert_eq!(arithmetic::inc16(99), 100);

    // ALU
    let result = arithmetic::alu(5, 3, false, false, false, false, true, false);
    assert_eq!(result.out, 8); // x + y

    println!("OK");
}

fn test_memory() {
    print!("Testing memory module... ");

    // DFF
    let mut dff = memory::DFF::new();
    dff.set(true);
    dff.tick();
    assert_eq!(dff.get(), true);

    // Register
    let mut reg = memory::Register::new();
    reg.set(0x1234, true);
    reg.tick();
    assert_eq!(reg.get(), 0x1234);

    // RAM8
    let mut ram = memory::RAM8::new();
    ram.set(0xABCD, 5, true);
    ram.tick();
    assert_eq!(ram.get(5), 0xABCD);

    // PC
    let mut pc = memory::PC::new();
    pc.set(0, true, false, false); // inc
    pc.tick();
    assert_eq!(pc.get(), 1);

    println!("OK");
}

fn test_instructions() {
    print!("Testing instruction module... ");

    // A-instruction
    let inst = instruction::parse_instruction(100).unwrap();
    if let instruction::Instruction::A(a) = inst {
        assert_eq!(a.value, 100);
    }

    // C-instruction roundtrip
    let word = 0b1110_1010_1000_0111; // 0;JMP
    let inst = instruction::parse_instruction(word).unwrap();
    let encoded = instruction::encode_instruction(&inst);
    assert_eq!(word, encoded);

    println!("OK");
}
