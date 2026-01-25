//! # Adder Circuits
//!
//! Binary addition built from gates.
//!
//! ## Binary Addition Basics
//!
//! Adding two bits:
//! ```text
//!   0 + 0 = 0 (sum=0, carry=0)
//!   0 + 1 = 1 (sum=1, carry=0)
//!   1 + 0 = 1 (sum=1, carry=0)
//!   1 + 1 = 10 (sum=0, carry=1)  ← "1 + 1 = 2" in binary is "10"
//! ```
//!
//! ## Two's Complement
//!
//! The Hack computer uses two's complement for signed integers:
//! - Positive numbers: normal binary (0 to 32767)
//! - Negative numbers: invert bits and add 1 (-1 to -32768)
//!
//! Example: -1 in 16-bit = 0xFFFF (all 1s)
//!
//! This means the same Add16 circuit handles both signed and unsigned!

use crate::gates::{xor, and};

/// Half Adder - adds two bits, produces sum and carry.
///
/// Called "half" because it doesn't handle a carry-in.
///
/// # Truth Table
/// ```text
/// a b | sum carry
/// ----+-----------
/// 0 0 |  0    0
/// 0 1 |  1    0
/// 1 0 |  1    0
/// 1 1 |  0    1
/// ```
///
/// Notice: sum looks like XOR, carry looks like AND.
pub fn half_adder(a: bool, b: bool) -> (bool, bool) {
    todo!("Return (sum, carry) using xor and and")
}

/// Full Adder - adds three bits (a, b, and carry-in).
///
/// This is what you chain together to add multi-bit numbers.
///
/// # Truth Table
/// ```text
/// a b c | sum carry
/// ------+-----------
/// 0 0 0 |  0    0
/// 0 0 1 |  1    0
/// 0 1 0 |  1    0
/// 0 1 1 |  0    1
/// 1 0 0 |  1    0
/// 1 0 1 |  0    1
/// 1 1 0 |  0    1
/// 1 1 1 |  1    1
/// ```
///
/// Hint: Use two half adders.
/// First: add a + b → (sum1, carry1)
/// Second: add sum1 + c → (final_sum, carry2)
/// Final carry = carry1 OR carry2
pub fn full_adder(a: bool, b: bool, c: bool) -> (bool, bool) {
    todo!("Return (sum, carry) - hint: use two half_adders and an OR")
}

/// 16-bit Adder - adds two 16-bit numbers.
///
/// This is a "ripple carry" adder: the carry ripples from bit 0 to bit 15.
///
/// ```text
///    a[15]...a[0]
///  + b[15]...b[0]
///  ──────────────
///    out[15]...out[0]
///
/// Internal carries: c0 → c1 → c2 → ... → c15
/// ```
///
/// Overflow is silently ignored (the final carry-out is discarded).
/// This is standard behavior for fixed-width integer arithmetic.
pub fn add16(a: u16, b: u16) -> u16 {
    todo!("Chain 16 full adders, bit 0 to bit 15")
}

/// Increment - adds 1 to a 16-bit number.
///
/// Useful for:
/// - Program counter increment (PC = PC + 1)
/// - Loop counters
///
/// Hint: This is just add16(a, 1), but you could also implement it more efficiently.
pub fn inc16(a: u16) -> u16 {
    todo!("Return a + 1")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_half_adder() {
        assert_eq!(half_adder(false, false), (false, false));
        assert_eq!(half_adder(false, true), (true, false));
        assert_eq!(half_adder(true, false), (true, false));
        assert_eq!(half_adder(true, true), (false, true));
    }

    #[test]
    fn test_full_adder() {
        assert_eq!(full_adder(false, false, false), (false, false));
        assert_eq!(full_adder(false, false, true), (true, false));
        assert_eq!(full_adder(false, true, false), (true, false));
        assert_eq!(full_adder(false, true, true), (false, true));
        assert_eq!(full_adder(true, false, false), (true, false));
        assert_eq!(full_adder(true, false, true), (false, true));
        assert_eq!(full_adder(true, true, false), (false, true));
        assert_eq!(full_adder(true, true, true), (true, true));
    }

    #[test]
    fn test_add16() {
        assert_eq!(add16(0, 0), 0);
        assert_eq!(add16(1, 1), 2);
        assert_eq!(add16(0x00FF, 0x0001), 0x0100);
        assert_eq!(add16(0x1234, 0x5678), 0x68AC);

        // Test overflow (wraps around)
        assert_eq!(add16(0xFFFF, 0x0001), 0x0000);
        assert_eq!(add16(0x8000, 0x8000), 0x0000);
    }

    #[test]
    fn test_add16_negative() {
        // In two's complement, 0xFFFF = -1
        // -1 + 1 = 0
        assert_eq!(add16(0xFFFF, 0x0001), 0x0000);

        // -1 + -1 = -2 (0xFFFE)
        assert_eq!(add16(0xFFFF, 0xFFFF), 0xFFFE);
    }

    #[test]
    fn test_inc16() {
        assert_eq!(inc16(0), 1);
        assert_eq!(inc16(1), 2);
        assert_eq!(inc16(0x00FF), 0x0100);
        assert_eq!(inc16(0xFFFE), 0xFFFF);
        assert_eq!(inc16(0xFFFF), 0x0000); // Overflow wraps
    }
}
