//! Integration tests for arithmetic module (Chapter 2)

use hack_rs::arithmetic::*;

// ============================================
// Adders
// ============================================

#[test]
fn test_half_adder_truth_table() {
    // (sum, carry)
    assert_eq!(half_adder(false, false), (false, false)); // 0 + 0 = 0
    assert_eq!(half_adder(false, true), (true, false));   // 0 + 1 = 1
    assert_eq!(half_adder(true, false), (true, false));   // 1 + 0 = 1
    assert_eq!(half_adder(true, true), (false, true));    // 1 + 1 = 10 (carry)
}

#[test]
fn test_full_adder_truth_table() {
    // (sum, carry)
    assert_eq!(full_adder(false, false, false), (false, false)); // 0+0+0 = 0
    assert_eq!(full_adder(false, false, true), (true, false));   // 0+0+1 = 1
    assert_eq!(full_adder(false, true, false), (true, false));   // 0+1+0 = 1
    assert_eq!(full_adder(false, true, true), (false, true));    // 0+1+1 = 10
    assert_eq!(full_adder(true, false, false), (true, false));   // 1+0+0 = 1
    assert_eq!(full_adder(true, false, true), (false, true));    // 1+0+1 = 10
    assert_eq!(full_adder(true, true, false), (false, true));    // 1+1+0 = 10
    assert_eq!(full_adder(true, true, true), (true, true));      // 1+1+1 = 11
}

#[test]
fn test_add16_basic() {
    assert_eq!(add16(0, 0), 0);
    assert_eq!(add16(1, 1), 2);
    assert_eq!(add16(1, 2), 3);
    assert_eq!(add16(100, 200), 300);
}

#[test]
fn test_add16_larger_numbers() {
    assert_eq!(add16(0x1234, 0x5678), 0x68AC);
    assert_eq!(add16(0x00FF, 0x0001), 0x0100);
    assert_eq!(add16(0x0F00, 0x0100), 0x1000);
}

#[test]
fn test_add16_overflow() {
    // Overflow wraps around (this is expected behavior)
    assert_eq!(add16(0xFFFF, 0x0001), 0x0000);
    assert_eq!(add16(0xFFFF, 0x0002), 0x0001);
    assert_eq!(add16(0x8000, 0x8000), 0x0000);
}

#[test]
fn test_add16_twos_complement() {
    // In two's complement, 0xFFFF = -1
    // -1 + 1 = 0
    assert_eq!(add16(0xFFFF, 0x0001), 0x0000);

    // -1 + -1 = -2 (0xFFFE)
    assert_eq!(add16(0xFFFF, 0xFFFF), 0xFFFE);

    // -1 + 2 = 1
    assert_eq!(add16(0xFFFF, 0x0002), 0x0001);
}

#[test]
fn test_inc16_basic() {
    assert_eq!(inc16(0), 1);
    assert_eq!(inc16(1), 2);
    assert_eq!(inc16(99), 100);
    assert_eq!(inc16(0x00FF), 0x0100);
}

#[test]
fn test_inc16_overflow() {
    assert_eq!(inc16(0xFFFF), 0x0000);
    assert_eq!(inc16(0xFFFE), 0xFFFF);
}

// ============================================
// ALU
// ============================================

/// Helper to make ALU tests more readable
fn run_alu(x: u16, y: u16, control: &str) -> AluOutput {
    let bits: Vec<bool> = control.chars().map(|c| c == '1').collect();
    assert_eq!(bits.len(), 6, "Control string must be 6 bits");
    alu(x, y, bits[0], bits[1], bits[2], bits[3], bits[4], bits[5])
}

#[test]
fn test_alu_constant_zero() {
    // zx=1, nx=0, zy=1, ny=0, f=1, no=0 → 0
    let result = run_alu(0x1234, 0x5678, "101010");
    assert_eq!(result.out, 0);
    assert!(result.zr, "Zero flag should be set");
    assert!(!result.ng, "Negative flag should not be set");
}

#[test]
fn test_alu_constant_one() {
    // zx=1, nx=1, zy=1, ny=1, f=1, no=1 → 1
    let result = run_alu(0x1234, 0x5678, "111111");
    assert_eq!(result.out, 1);
    assert!(!result.zr);
    assert!(!result.ng);
}

#[test]
fn test_alu_constant_minus_one() {
    // zx=1, nx=1, zy=1, ny=0, f=1, no=0 → -1 (0xFFFF)
    let result = run_alu(0x1234, 0x5678, "111010");
    assert_eq!(result.out, 0xFFFF);
    assert!(!result.zr);
    assert!(result.ng, "Negative flag should be set for -1");
}

#[test]
fn test_alu_pass_x() {
    // zx=0, nx=0, zy=1, ny=1, f=0, no=0 → x
    let result = run_alu(0x1234, 0x5678, "001100");
    assert_eq!(result.out, 0x1234);
}

#[test]
fn test_alu_pass_y() {
    // zx=1, nx=1, zy=0, ny=0, f=0, no=0 → y
    let result = run_alu(0x1234, 0x5678, "110000");
    assert_eq!(result.out, 0x5678);
}

#[test]
fn test_alu_not_x() {
    // zx=0, nx=0, zy=1, ny=1, f=0, no=1 → !x
    let result = run_alu(0x00FF, 0x5678, "001101");
    assert_eq!(result.out, 0xFF00);
}

#[test]
fn test_alu_not_y() {
    // zx=1, nx=1, zy=0, ny=0, f=0, no=1 → !y
    let result = run_alu(0x1234, 0x00FF, "110001");
    assert_eq!(result.out, 0xFF00);
}

#[test]
fn test_alu_negate_x() {
    // zx=0, nx=0, zy=1, ny=1, f=1, no=1 → -x
    let result = run_alu(5, 0x5678, "001111");
    assert_eq!(result.out, 0xFFFB); // -5 in two's complement
    assert!(result.ng);
}

#[test]
fn test_alu_negate_y() {
    // zx=1, nx=1, zy=0, ny=0, f=1, no=1 → -y
    let result = run_alu(0x1234, 5, "110011");
    assert_eq!(result.out, 0xFFFB); // -5
    assert!(result.ng);
}

#[test]
fn test_alu_x_plus_one() {
    // zx=0, nx=1, zy=1, ny=1, f=1, no=1 → x+1
    let result = run_alu(10, 0x5678, "011111");
    assert_eq!(result.out, 11);
}

#[test]
fn test_alu_y_plus_one() {
    // zx=1, nx=1, zy=0, ny=1, f=1, no=1 → y+1
    let result = run_alu(0x1234, 10, "110111");
    assert_eq!(result.out, 11);
}

#[test]
fn test_alu_x_minus_one() {
    // zx=0, nx=0, zy=1, ny=1, f=1, no=0 → x-1
    let result = run_alu(10, 0x5678, "001110");
    assert_eq!(result.out, 9);
}

#[test]
fn test_alu_y_minus_one() {
    // zx=1, nx=1, zy=0, ny=0, f=1, no=0 → y-1
    let result = run_alu(0x1234, 10, "110010");
    assert_eq!(result.out, 9);
}

#[test]
fn test_alu_x_plus_y() {
    // zx=0, nx=0, zy=0, ny=0, f=1, no=0 → x+y
    let result = run_alu(5, 3, "000010");
    assert_eq!(result.out, 8);
    assert!(!result.zr);
    assert!(!result.ng);
}

#[test]
fn test_alu_x_minus_y() {
    // zx=0, nx=1, zy=0, ny=0, f=1, no=1 → x-y
    let result = run_alu(10, 3, "010011");
    assert_eq!(result.out, 7);

    // Result can be negative
    let result = run_alu(3, 10, "010011");
    assert_eq!(result.out, 0xFFF9); // -7
    assert!(result.ng);
}

#[test]
fn test_alu_y_minus_x() {
    // zx=0, nx=0, zy=0, ny=1, f=1, no=1 → y-x
    let result = run_alu(3, 10, "000111");
    assert_eq!(result.out, 7);
}

#[test]
fn test_alu_x_and_y() {
    // zx=0, nx=0, zy=0, ny=0, f=0, no=0 → x&y
    let result = run_alu(0xFF00, 0x0FF0, "000000");
    assert_eq!(result.out, 0x0F00);
}

#[test]
fn test_alu_x_or_y() {
    // zx=0, nx=1, zy=0, ny=1, f=0, no=1 → x|y
    // Using De Morgan: x|y = !(!x & !y)
    let result = run_alu(0xFF00, 0x0FF0, "010101");
    assert_eq!(result.out, 0xFFF0);
}

#[test]
fn test_alu_zero_flag() {
    // 5 - 5 = 0
    let result = run_alu(5, 5, "010011");
    assert_eq!(result.out, 0);
    assert!(result.zr, "Zero flag should be set when result is 0");
    assert!(!result.ng);
}

#[test]
fn test_alu_negative_flag() {
    // 0 - 1 = -1
    let result = run_alu(0, 1, "010011");
    assert_eq!(result.out, 0xFFFF);
    assert!(!result.zr);
    assert!(result.ng, "Negative flag should be set for negative results");
}
