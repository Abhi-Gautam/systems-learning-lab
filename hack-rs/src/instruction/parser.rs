//! # Instruction Parser
//!
//! Parse and encode Hack machine instructions.
//!
//! ## Binary Format Reference
//!
//! ### A-instruction: `0vvv vvvv vvvv vvvv`
//!
//! Simply loads a 15-bit value into the A register.
//!
//! ### C-instruction: `111a cccc ccdd djjj`
//!
//! ```text
//! Bit 15-13: 111 (always, identifies C-instruction)
//! Bit 12 (a): 0 = use A register, 1 = use M (RAM[A])
//! Bit 11-6 (cccccc): computation to perform
//! Bit 5-3 (ddd): destination (where to store result)
//! Bit 2-0 (jjj): jump condition
//! ```
//!
//! ## Comp Field (a + cccccc)
//!
//! ```text
//! When a=0:           When a=1:
//! comp   c1c2c3c4c5c6  comp   c1c2c3c4c5c6
//! 0      101010
//! 1      111111
//! -1     111010
//! D      001100
//! A      110000        M      110000
//! !D     001101
//! !A     110001        !M     110001
//! -D     001111
//! -A     110011        -M     110011
//! D+1    011111
//! A+1    110111        M+1    110111
//! D-1    001110
//! A-1    110010        M-1    110010
//! D+A    000010        D+M    000010
//! D-A    010011        D-M    010011
//! A-D    000111        M-D    000111
//! D&A    000000        D&M    000000
//! D|A    010101        D|M    010101
//! ```

/// A-instruction: loads a value into the A register.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AInstruction {
    /// 15-bit value (0-32767)
    pub value: u16,
}

/// Destination specifier for C-instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Dest {
    /// Store in A register
    pub a: bool,
    /// Store in D register
    pub d: bool,
    /// Store in M (RAM[A])
    pub m: bool,
}

impl Dest {
    /// Create a Dest from the 3-bit ddd field.
    pub fn from_bits(bits: u8) -> Self {
        todo!("Extract a, d, m from bits (bit 2=A, bit 1=D, bit 0=M)")
    }

    /// Convert to 3-bit ddd field.
    pub fn to_bits(&self) -> u8 {
        todo!("Combine a, d, m into 3 bits")
    }
}

/// Computation specifier for C-instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comp {
    // Constants
    Zero,       // 0
    One,        // 1
    MinusOne,   // -1

    // D register operations
    D,          // D
    NotD,       // !D
    MinusD,     // -D
    DPlusOne,   // D+1
    DMinusOne,  // D-1

    // A register operations (a=0)
    A,          // A
    NotA,       // !A
    MinusA,     // -A
    APlusOne,   // A+1
    AMinusOne,  // A-1

    // M (memory) operations (a=1)
    M,          // M
    NotM,       // !M
    MinusM,     // -M
    MPlusOne,   // M+1
    MMinusOne,  // M-1

    // D and A operations (a=0)
    DPlusA,     // D+A
    DMinusA,    // D-A
    AMinusD,    // A-D
    DAndA,      // D&A
    DOrA,       // D|A

    // D and M operations (a=1)
    DPlusM,     // D+M
    DMinusM,    // D-M
    MMinusD,    // M-D
    DAndM,      // D&M
    DOrM,       // D|M
}

impl Comp {
    /// Create a Comp from the 7-bit `a cccccc` field.
    pub fn from_bits(bits: u8) -> Option<Self> {
        todo!("Match the 7-bit pattern to the corresponding Comp variant")
        // Hint: The 'a' bit is bit 6, cccccc is bits 5-0
        // Use a match statement on the full 7 bits
    }

    /// Convert to 7-bit `a cccccc` field.
    pub fn to_bits(&self) -> u8 {
        todo!("Return the 7-bit encoding for this computation")
    }

    /// Returns true if this computation uses M (memory) instead of A.
    pub fn uses_memory(&self) -> bool {
        matches!(
            self,
            Comp::M | Comp::NotM | Comp::MinusM | Comp::MPlusOne | Comp::MMinusOne |
            Comp::DPlusM | Comp::DMinusM | Comp::MMinusD | Comp::DAndM | Comp::DOrM
        )
    }
}

/// Jump condition for C-instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Jump {
    #[default]
    Null,   // No jump (000)
    JGT,    // Jump if out > 0 (001)
    JEQ,    // Jump if out = 0 (010)
    JGE,    // Jump if out >= 0 (011)
    JLT,    // Jump if out < 0 (100)
    JNE,    // Jump if out != 0 (101)
    JLE,    // Jump if out <= 0 (110)
    JMP,    // Unconditional jump (111)
}

impl Jump {
    /// Create a Jump from the 3-bit jjj field.
    pub fn from_bits(bits: u8) -> Self {
        todo!("Match bits 0-7 to Jump variants")
    }

    /// Convert to 3-bit jjj field.
    pub fn to_bits(&self) -> u8 {
        todo!("Return 0-7 based on jump type")
    }
}

/// C-instruction: compute and optionally store/jump.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CInstruction {
    /// What to compute
    pub comp: Comp,
    /// Where to store the result
    pub dest: Dest,
    /// When to jump
    pub jump: Jump,
}

/// A Hack instruction (either A or C type).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    A(AInstruction),
    C(CInstruction),
}

/// Parse a 16-bit instruction word into an Instruction.
///
/// # Arguments
/// * `word` - The 16-bit instruction
///
/// # Returns
/// * `Some(Instruction)` if valid
/// * `None` if the instruction is malformed
pub fn parse_instruction(word: u16) -> Option<Instruction> {
    todo!("Check bit 15 to determine A vs C instruction, then parse accordingly")
    // Hint:
    // - Bit 15 = 0 → A-instruction, value is bits 14-0
    // - Bit 15 = 1 → C-instruction, extract comp/dest/jump fields
}

/// Encode an Instruction back to a 16-bit word.
pub fn encode_instruction(inst: &Instruction) -> u16 {
    todo!("Convert instruction back to 16-bit binary")
    // Hint:
    // - A-instruction: just return the value (bit 15 is 0)
    // - C-instruction: combine 111 + comp + dest + jump
}

#[cfg(test)]
mod tests {
    use super::*;

    // A-instruction tests
    #[test]
    fn test_parse_a_instruction() {
        // @0
        let inst = parse_instruction(0b0000_0000_0000_0000).unwrap();
        assert_eq!(inst, Instruction::A(AInstruction { value: 0 }));

        // @100
        let inst = parse_instruction(0b0000_0000_0110_0100).unwrap();
        assert_eq!(inst, Instruction::A(AInstruction { value: 100 }));

        // @32767 (max value)
        let inst = parse_instruction(0b0111_1111_1111_1111).unwrap();
        assert_eq!(inst, Instruction::A(AInstruction { value: 32767 }));
    }

    #[test]
    fn test_encode_a_instruction() {
        let inst = Instruction::A(AInstruction { value: 100 });
        assert_eq!(encode_instruction(&inst), 0b0000_0000_0110_0100);

        let inst = Instruction::A(AInstruction { value: 0 });
        assert_eq!(encode_instruction(&inst), 0);
    }

    // Dest tests
    #[test]
    fn test_dest_from_bits() {
        assert_eq!(Dest::from_bits(0b000), Dest { a: false, d: false, m: false });
        assert_eq!(Dest::from_bits(0b001), Dest { a: false, d: false, m: true });
        assert_eq!(Dest::from_bits(0b010), Dest { a: false, d: true, m: false });
        assert_eq!(Dest::from_bits(0b011), Dest { a: false, d: true, m: true });
        assert_eq!(Dest::from_bits(0b100), Dest { a: true, d: false, m: false });
        assert_eq!(Dest::from_bits(0b111), Dest { a: true, d: true, m: true });
    }

    #[test]
    fn test_dest_to_bits() {
        assert_eq!(Dest { a: false, d: false, m: false }.to_bits(), 0b000);
        assert_eq!(Dest { a: false, d: false, m: true }.to_bits(), 0b001);
        assert_eq!(Dest { a: false, d: true, m: false }.to_bits(), 0b010);
        assert_eq!(Dest { a: true, d: true, m: true }.to_bits(), 0b111);
    }

    // Jump tests
    #[test]
    fn test_jump_from_bits() {
        assert_eq!(Jump::from_bits(0b000), Jump::Null);
        assert_eq!(Jump::from_bits(0b001), Jump::JGT);
        assert_eq!(Jump::from_bits(0b010), Jump::JEQ);
        assert_eq!(Jump::from_bits(0b011), Jump::JGE);
        assert_eq!(Jump::from_bits(0b100), Jump::JLT);
        assert_eq!(Jump::from_bits(0b101), Jump::JNE);
        assert_eq!(Jump::from_bits(0b110), Jump::JLE);
        assert_eq!(Jump::from_bits(0b111), Jump::JMP);
    }

    #[test]
    fn test_jump_to_bits() {
        assert_eq!(Jump::Null.to_bits(), 0b000);
        assert_eq!(Jump::JGT.to_bits(), 0b001);
        assert_eq!(Jump::JMP.to_bits(), 0b111);
    }

    // Comp tests
    #[test]
    fn test_comp_from_bits() {
        // a=0 computations
        assert_eq!(Comp::from_bits(0b0_101010), Some(Comp::Zero));
        assert_eq!(Comp::from_bits(0b0_111111), Some(Comp::One));
        assert_eq!(Comp::from_bits(0b0_111010), Some(Comp::MinusOne));
        assert_eq!(Comp::from_bits(0b0_001100), Some(Comp::D));
        assert_eq!(Comp::from_bits(0b0_110000), Some(Comp::A));
        assert_eq!(Comp::from_bits(0b0_000010), Some(Comp::DPlusA));

        // a=1 computations (use M instead of A)
        assert_eq!(Comp::from_bits(0b1_110000), Some(Comp::M));
        assert_eq!(Comp::from_bits(0b1_000010), Some(Comp::DPlusM));
    }

    #[test]
    fn test_comp_to_bits() {
        assert_eq!(Comp::Zero.to_bits(), 0b0_101010);
        assert_eq!(Comp::One.to_bits(), 0b0_111111);
        assert_eq!(Comp::D.to_bits(), 0b0_001100);
        assert_eq!(Comp::M.to_bits(), 0b1_110000);
        assert_eq!(Comp::DPlusM.to_bits(), 0b1_000010);
    }

    // C-instruction tests
    #[test]
    fn test_parse_c_instruction() {
        // D=A (comp=A, dest=D, jump=null)
        // 111 0 110000 010 000
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
    fn test_parse_c_instruction_with_jump() {
        // 0;JMP (comp=0, dest=null, jump=JMP)
        // 111 0 101010 000 111
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
    fn test_encode_c_instruction() {
        let inst = Instruction::C(CInstruction {
            comp: Comp::DPlusOne,
            dest: Dest { a: false, d: true, m: false },
            jump: Jump::Null,
        });
        // D=D+1: 111 0 011111 010 000
        assert_eq!(encode_instruction(&inst), 0b1110_0111_1101_0000);
    }

    #[test]
    fn test_roundtrip() {
        // Test that parse -> encode -> parse gives same result
        let words = [
            0b0000_0000_0110_0100, // @100
            0b0111_1111_1111_1111, // @32767
            0b1110_1100_0001_0000, // D=A
            0b1110_0111_1101_0000, // D=D+1
            0b1110_1010_1000_0111, // 0;JMP
            0b1111_1100_0001_0000, // D=M
        ];

        for word in words {
            let inst = parse_instruction(word).unwrap();
            let encoded = encode_instruction(&inst);
            assert_eq!(word, encoded, "Roundtrip failed for {:#018b}", word);
        }
    }

    #[test]
    fn test_comp_uses_memory() {
        assert!(!Comp::A.uses_memory());
        assert!(!Comp::D.uses_memory());
        assert!(!Comp::DPlusA.uses_memory());

        assert!(Comp::M.uses_memory());
        assert!(Comp::DPlusM.uses_memory());
        assert!(Comp::MMinusD.uses_memory());
    }
}
