//! Integration tests for gates module (Chapter 1)

use hack_rs::gates::*;

// ============================================
// Basic Gates (from NAND)
// ============================================

#[test]
fn test_nand_exhaustive() {
    // NAND truth table - the foundation of everything
    assert_eq!(nand(false, false), true, "NAND(0,0) should be 1");
    assert_eq!(nand(false, true), true, "NAND(0,1) should be 1");
    assert_eq!(nand(true, false), true, "NAND(1,0) should be 1");
    assert_eq!(nand(true, true), false, "NAND(1,1) should be 0");
}

#[test]
fn test_not_exhaustive() {
    assert_eq!(not(false), true);
    assert_eq!(not(true), false);
}

#[test]
fn test_and_exhaustive() {
    assert_eq!(and(false, false), false);
    assert_eq!(and(false, true), false);
    assert_eq!(and(true, false), false);
    assert_eq!(and(true, true), true);
}

#[test]
fn test_or_exhaustive() {
    assert_eq!(or(false, false), false);
    assert_eq!(or(false, true), true);
    assert_eq!(or(true, false), true);
    assert_eq!(or(true, true), true);
}

#[test]
fn test_xor_exhaustive() {
    assert_eq!(xor(false, false), false);
    assert_eq!(xor(false, true), true);
    assert_eq!(xor(true, false), true);
    assert_eq!(xor(true, true), false);
}

// ============================================
// Multiplexer and Demultiplexer
// ============================================

#[test]
fn test_mux_selects_a_when_sel_false() {
    assert_eq!(mux(true, false, false), true);
    assert_eq!(mux(false, true, false), false);
}

#[test]
fn test_mux_selects_b_when_sel_true() {
    assert_eq!(mux(true, false, true), false);
    assert_eq!(mux(false, true, true), true);
}

#[test]
fn test_dmux_routes_to_a_when_sel_false() {
    assert_eq!(dmux(true, false), (true, false));
    assert_eq!(dmux(false, false), (false, false));
}

#[test]
fn test_dmux_routes_to_b_when_sel_true() {
    assert_eq!(dmux(true, true), (false, true));
    assert_eq!(dmux(false, true), (false, false));
}

// ============================================
// 16-bit Gates
// ============================================

#[test]
fn test_not16_patterns() {
    assert_eq!(not16(0x0000), 0xFFFF);
    assert_eq!(not16(0xFFFF), 0x0000);
    assert_eq!(not16(0xAAAA), 0x5555); // Alternating bits
    assert_eq!(not16(0x5555), 0xAAAA);
    assert_eq!(not16(0x00FF), 0xFF00);
}

#[test]
fn test_and16_patterns() {
    assert_eq!(and16(0xFFFF, 0xFFFF), 0xFFFF);
    assert_eq!(and16(0x0000, 0xFFFF), 0x0000);
    assert_eq!(and16(0xFFFF, 0x0000), 0x0000);
    assert_eq!(and16(0xAAAA, 0x5555), 0x0000); // No overlap
    assert_eq!(and16(0xFF00, 0x0FF0), 0x0F00); // Partial overlap
}

#[test]
fn test_or16_patterns() {
    assert_eq!(or16(0x0000, 0x0000), 0x0000);
    assert_eq!(or16(0xFFFF, 0x0000), 0xFFFF);
    assert_eq!(or16(0xAAAA, 0x5555), 0xFFFF); // Complete coverage
    assert_eq!(or16(0xF000, 0x000F), 0xF00F);
}

#[test]
fn test_mux16_selects_correctly() {
    assert_eq!(mux16(0x1234, 0x5678, false), 0x1234);
    assert_eq!(mux16(0x1234, 0x5678, true), 0x5678);
    assert_eq!(mux16(0xFFFF, 0x0000, false), 0xFFFF);
    assert_eq!(mux16(0xFFFF, 0x0000, true), 0x0000);
}

// ============================================
// Multi-way Gates
// ============================================

#[test]
fn test_or8way_all_combinations() {
    assert_eq!(or8way(0b00000000), false);
    assert_eq!(or8way(0b00000001), true);
    assert_eq!(or8way(0b10000000), true);
    assert_eq!(or8way(0b01010101), true);
    assert_eq!(or8way(0b11111111), true);
}

#[test]
fn test_mux4way16_all_selections() {
    let a = 0x1111;
    let b = 0x2222;
    let c = 0x3333;
    let d = 0x4444;

    // sel[1] sel[0]
    assert_eq!(mux4way16(a, b, c, d, [false, false]), a); // 00
    assert_eq!(mux4way16(a, b, c, d, [false, true]), b);  // 01
    assert_eq!(mux4way16(a, b, c, d, [true, false]), c);  // 10
    assert_eq!(mux4way16(a, b, c, d, [true, true]), d);   // 11
}

#[test]
fn test_mux8way16_all_selections() {
    let inputs = [0x1000, 0x2000, 0x3000, 0x4000, 0x5000, 0x6000, 0x7000, 0x8000];

    let selectors = [
        [false, false, false], // 000 -> inputs[0]
        [false, false, true],  // 001 -> inputs[1]
        [false, true, false],  // 010 -> inputs[2]
        [false, true, true],   // 011 -> inputs[3]
        [true, false, false],  // 100 -> inputs[4]
        [true, false, true],   // 101 -> inputs[5]
        [true, true, false],   // 110 -> inputs[6]
        [true, true, true],    // 111 -> inputs[7]
    ];

    for (i, sel) in selectors.iter().enumerate() {
        let result = mux8way16(
            inputs[0], inputs[1], inputs[2], inputs[3],
            inputs[4], inputs[5], inputs[6], inputs[7],
            *sel
        );
        assert_eq!(result, inputs[i], "mux8way16 with sel={:?} should select inputs[{}]", sel, i);
    }
}

#[test]
fn test_dmux4way_routes_correctly() {
    // Input goes to position selected by sel
    assert_eq!(dmux4way(true, [false, false]), (true, false, false, false));
    assert_eq!(dmux4way(true, [false, true]), (false, true, false, false));
    assert_eq!(dmux4way(true, [true, false]), (false, false, true, false));
    assert_eq!(dmux4way(true, [true, true]), (false, false, false, true));

    // False input produces all false outputs
    assert_eq!(dmux4way(false, [true, true]), (false, false, false, false));
}

#[test]
fn test_dmux8way_routes_correctly() {
    let selectors = [
        [false, false, false],
        [false, false, true],
        [false, true, false],
        [false, true, true],
        [true, false, false],
        [true, false, true],
        [true, true, false],
        [true, true, true],
    ];

    let expected_outputs = [
        (true, false, false, false, false, false, false, false),
        (false, true, false, false, false, false, false, false),
        (false, false, true, false, false, false, false, false),
        (false, false, false, true, false, false, false, false),
        (false, false, false, false, true, false, false, false),
        (false, false, false, false, false, true, false, false),
        (false, false, false, false, false, false, true, false),
        (false, false, false, false, false, false, false, true),
    ];

    for (sel, expected) in selectors.iter().zip(expected_outputs.iter()) {
        assert_eq!(dmux8way(true, *sel), *expected, "dmux8way with sel={:?}", sel);
    }
}
