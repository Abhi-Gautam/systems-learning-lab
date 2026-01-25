//! Integration tests for instruction module (Chapter 4)

use hack_rs::instruction::*;

// ============================================
// A-instruction Parsing
// ============================================

#[test]
fn test_parse_a_instruction_zero() {
    let inst = parse_instruction(0b0000_0000_0000_0000).unwrap();
    assert_eq!(inst, Instruction::A(AInstruction { value: 0 }));
}

#[test]
fn test_parse_a_instruction_small() {
    let inst = parse_instruction(0b0000_0000_0110_0100).unwrap(); // 100
    assert_eq!(inst, Instruction::A(AInstruction { value: 100 }));
}

#[test]
fn test_parse_a_instruction_max() {
    let inst = parse_instruction(0b0111_1111_1111_1111).unwrap(); // 32767
    assert_eq!(inst, Instruction::A(AInstruction { value: 32767 }));
}

#[test]
fn test_parse_a_instruction_various() {
    let cases = [
        (0b0000_0000_0000_0001, 1),
        (0b0000_0000_0000_1111, 15),
        (0b0000_0000_1111_1111, 255),
        (0b0000_1111_1111_1111, 4095),
        (0b0100_0000_0000_0000, 16384),
    ];

    for (word, expected) in cases {
        let inst = parse_instruction(word).unwrap();
        assert_eq!(inst, Instruction::A(AInstruction { value: expected }));
    }
}

// ============================================
// A-instruction Encoding
// ============================================

#[test]
fn test_encode_a_instruction() {
    let cases = [
        (0, 0b0000_0000_0000_0000),
        (1, 0b0000_0000_0000_0001),
        (100, 0b0000_0000_0110_0100),
        (32767, 0b0111_1111_1111_1111),
    ];

    for (value, expected) in cases {
        let inst = Instruction::A(AInstruction { value });
        assert_eq!(encode_instruction(&inst), expected);
    }
}

// ============================================
// Dest Field
// ============================================

#[test]
fn test_dest_from_bits() {
    // Test all 8 combinations
    let cases = [
        (0b000, (false, false, false)), // null
        (0b001, (false, false, true)),  // M
        (0b010, (false, true, false)),  // D
        (0b011, (false, true, true)),   // DM
        (0b100, (true, false, false)),  // A
        (0b101, (true, false, true)),   // AM
        (0b110, (true, true, false)),   // AD
        (0b111, (true, true, true)),    // ADM
    ];

    for (bits, (a, d, m)) in cases {
        let dest = Dest::from_bits(bits);
        assert_eq!(dest, Dest { a, d, m }, "Dest::from_bits({:#05b})", bits);
    }
}

#[test]
fn test_dest_to_bits() {
    let cases = [
        (Dest { a: false, d: false, m: false }, 0b000),
        (Dest { a: false, d: false, m: true }, 0b001),
        (Dest { a: false, d: true, m: false }, 0b010),
        (Dest { a: false, d: true, m: true }, 0b011),
        (Dest { a: true, d: false, m: false }, 0b100),
        (Dest { a: true, d: true, m: true }, 0b111),
    ];

    for (dest, expected) in cases {
        assert_eq!(dest.to_bits(), expected, "{:?}.to_bits()", dest);
    }
}

// ============================================
// Jump Field
// ============================================

#[test]
fn test_jump_from_bits() {
    let cases = [
        (0b000, Jump::Null),
        (0b001, Jump::JGT),
        (0b010, Jump::JEQ),
        (0b011, Jump::JGE),
        (0b100, Jump::JLT),
        (0b101, Jump::JNE),
        (0b110, Jump::JLE),
        (0b111, Jump::JMP),
    ];

    for (bits, expected) in cases {
        assert_eq!(Jump::from_bits(bits), expected, "Jump::from_bits({:#05b})", bits);
    }
}

#[test]
fn test_jump_to_bits() {
    let cases = [
        (Jump::Null, 0b000),
        (Jump::JGT, 0b001),
        (Jump::JEQ, 0b010),
        (Jump::JGE, 0b011),
        (Jump::JLT, 0b100),
        (Jump::JNE, 0b101),
        (Jump::JLE, 0b110),
        (Jump::JMP, 0b111),
    ];

    for (jump, expected) in cases {
        assert_eq!(jump.to_bits(), expected, "{:?}.to_bits()", jump);
    }
}

// ============================================
// Comp Field
// ============================================

#[test]
fn test_comp_from_bits_a_zero() {
    // a=0 (use A register)
    let cases = [
        (0b0_101010, Comp::Zero),
        (0b0_111111, Comp::One),
        (0b0_111010, Comp::MinusOne),
        (0b0_001100, Comp::D),
        (0b0_110000, Comp::A),
        (0b0_001101, Comp::NotD),
        (0b0_110001, Comp::NotA),
        (0b0_001111, Comp::MinusD),
        (0b0_110011, Comp::MinusA),
        (0b0_011111, Comp::DPlusOne),
        (0b0_110111, Comp::APlusOne),
        (0b0_001110, Comp::DMinusOne),
        (0b0_110010, Comp::AMinusOne),
        (0b0_000010, Comp::DPlusA),
        (0b0_010011, Comp::DMinusA),
        (0b0_000111, Comp::AMinusD),
        (0b0_000000, Comp::DAndA),
        (0b0_010101, Comp::DOrA),
    ];

    for (bits, expected) in cases {
        assert_eq!(Comp::from_bits(bits), Some(expected), "Comp::from_bits({:#09b})", bits);
    }
}

#[test]
fn test_comp_from_bits_a_one() {
    // a=1 (use M register)
    let cases = [
        (0b1_110000, Comp::M),
        (0b1_110001, Comp::NotM),
        (0b1_110011, Comp::MinusM),
        (0b1_110111, Comp::MPlusOne),
        (0b1_110010, Comp::MMinusOne),
        (0b1_000010, Comp::DPlusM),
        (0b1_010011, Comp::DMinusM),
        (0b1_000111, Comp::MMinusD),
        (0b1_000000, Comp::DAndM),
        (0b1_010101, Comp::DOrM),
    ];

    for (bits, expected) in cases {
        assert_eq!(Comp::from_bits(bits), Some(expected), "Comp::from_bits({:#09b})", bits);
    }
}

#[test]
fn test_comp_to_bits() {
    let cases = [
        (Comp::Zero, 0b0_101010),
        (Comp::One, 0b0_111111),
        (Comp::MinusOne, 0b0_111010),
        (Comp::D, 0b0_001100),
        (Comp::A, 0b0_110000),
        (Comp::M, 0b1_110000),
        (Comp::DPlusA, 0b0_000010),
        (Comp::DPlusM, 0b1_000010),
    ];

    for (comp, expected) in cases {
        assert_eq!(comp.to_bits(), expected, "{:?}.to_bits()", comp);
    }
}

#[test]
fn test_comp_uses_memory() {
    // A-register operations
    assert!(!Comp::Zero.uses_memory());
    assert!(!Comp::D.uses_memory());
    assert!(!Comp::A.uses_memory());
    assert!(!Comp::DPlusA.uses_memory());

    // M-register operations
    assert!(Comp::M.uses_memory());
    assert!(Comp::NotM.uses_memory());
    assert!(Comp::DPlusM.uses_memory());
    assert!(Comp::MMinusD.uses_memory());
}

// ============================================
// C-instruction Parsing
// ============================================

#[test]
fn test_parse_c_instruction_d_equals_a() {
    // D=A: comp=A, dest=D, jump=null
    // 111 0 110000 010 000 = 0xE308... wait let me recalculate
    // 111 a=0 c1c2c3c4c5c6=110000 d1d2d3=010 j1j2j3=000
    // 1110 1100 0001 0000 = 0xEC10
    let word = 0b1110_1100_0001_0000;
    let inst = parse_instruction(word).unwrap();

    if let Instruction::C(c) = inst {
        assert_eq!(c.comp, Comp::A);
        assert_eq!(c.dest, Dest { a: false, d: true, m: false });
        assert_eq!(c.jump, Jump::Null);
    } else {
        panic!("Expected C-instruction");
    }
}

#[test]
fn test_parse_c_instruction_unconditional_jump() {
    // 0;JMP: comp=0, dest=null, jump=JMP
    // 111 0 101010 000 111 = 0xEA07
    let word = 0b1110_1010_1000_0111;
    let inst = parse_instruction(word).unwrap();

    if let Instruction::C(c) = inst {
        assert_eq!(c.comp, Comp::Zero);
        assert_eq!(c.dest, Dest { a: false, d: false, m: false });
        assert_eq!(c.jump, Jump::JMP);
    } else {
        panic!("Expected C-instruction");
    }
}

#[test]
fn test_parse_c_instruction_with_memory() {
    // D=M: comp=M, dest=D, jump=null
    // 111 1 110000 010 000
    let word = 0b1111_1100_0001_0000;
    let inst = parse_instruction(word).unwrap();

    if let Instruction::C(c) = inst {
        assert_eq!(c.comp, Comp::M);
        assert_eq!(c.dest, Dest { a: false, d: true, m: false });
        assert_eq!(c.jump, Jump::Null);
    } else {
        panic!("Expected C-instruction");
    }
}

#[test]
fn test_parse_c_instruction_conditional_jump() {
    // D;JGT: comp=D, dest=null, jump=JGT
    // 111 0 001100 000 001
    let word = 0b1110_0011_0000_0001;
    let inst = parse_instruction(word).unwrap();

    if let Instruction::C(c) = inst {
        assert_eq!(c.comp, Comp::D);
        assert_eq!(c.dest, Dest { a: false, d: false, m: false });
        assert_eq!(c.jump, Jump::JGT);
    } else {
        panic!("Expected C-instruction");
    }
}

// ============================================
// C-instruction Encoding
// ============================================

#[test]
fn test_encode_c_instruction_d_plus_one() {
    let inst = Instruction::C(CInstruction {
        comp: Comp::DPlusOne,
        dest: Dest { a: false, d: true, m: false },
        jump: Jump::Null,
    });
    // D=D+1: 111 0 011111 010 000
    assert_eq!(encode_instruction(&inst), 0b1110_0111_1101_0000);
}

#[test]
fn test_encode_c_instruction_m_equals_d_plus_m() {
    let inst = Instruction::C(CInstruction {
        comp: Comp::DPlusM,
        dest: Dest { a: false, d: false, m: true },
        jump: Jump::Null,
    });
    // M=D+M: 111 1 000010 001 000
    assert_eq!(encode_instruction(&inst), 0b1111_0000_1000_1000);
}

// ============================================
// Roundtrip Tests
// ============================================

#[test]
fn test_roundtrip_a_instructions() {
    let values = [0, 1, 100, 1000, 16384, 32767];

    for value in values {
        let word = value; // A-instruction is just the value with bit 15 = 0
        let inst = parse_instruction(word).unwrap();
        let encoded = encode_instruction(&inst);
        assert_eq!(word, encoded, "Roundtrip failed for A-instruction @{}", value);
    }
}

#[test]
fn test_roundtrip_c_instructions() {
    let words = [
        0b1110_1010_1000_0000, // 0
        0b1110_1111_1100_0000, // 1
        0b1110_1110_1000_0000, // -1
        0b1110_0011_0000_0000, // D
        0b1110_1100_0000_0000, // A
        0b1111_1100_0000_0000, // M
        0b1110_0011_0001_0000, // D+1 to D
        0b1110_0000_1001_0000, // D+A to D
        0b1111_0000_1000_1000, // D+M to M
        0b1110_1010_1000_0111, // 0;JMP
        0b1110_0011_0000_0001, // D;JGT
    ];

    for word in words {
        let inst = parse_instruction(word).unwrap();
        let encoded = encode_instruction(&inst);
        assert_eq!(word, encoded, "Roundtrip failed for {:#018b}", word);
    }
}

// ============================================
// Common Assembly Patterns
// ============================================

#[test]
fn test_common_pattern_load_constant() {
    // @17 (load 17 into A)
    let inst = parse_instruction(17).unwrap();
    if let Instruction::A(a) = inst {
        assert_eq!(a.value, 17);
    } else {
        panic!("Expected A-instruction");
    }
}

#[test]
fn test_common_pattern_d_equals_a() {
    // D=A
    let word = 0b1110_1100_0001_0000;
    let inst = parse_instruction(word).unwrap();
    if let Instruction::C(c) = inst {
        assert_eq!(c.comp, Comp::A);
        assert!(c.dest.d);
        assert!(!c.dest.a);
        assert!(!c.dest.m);
    } else {
        panic!("Expected C-instruction");
    }
}

#[test]
fn test_common_pattern_increment_d() {
    // D=D+1
    let word = 0b1110_0111_1101_0000;
    let inst = parse_instruction(word).unwrap();
    if let Instruction::C(c) = inst {
        assert_eq!(c.comp, Comp::DPlusOne);
        assert!(c.dest.d);
    } else {
        panic!("Expected C-instruction");
    }
}

#[test]
fn test_common_pattern_jump_if_zero() {
    // D;JEQ
    let word = 0b1110_0011_0000_0010;
    let inst = parse_instruction(word).unwrap();
    if let Instruction::C(c) = inst {
        assert_eq!(c.comp, Comp::D);
        assert_eq!(c.jump, Jump::JEQ);
    } else {
        panic!("Expected C-instruction");
    }
}
