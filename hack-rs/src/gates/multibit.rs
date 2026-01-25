//! 16-bit and multi-way gate variants.

use super::basic::{not, and, or};
use super::mux::{mux, dmux};

/// 16-bit NOT.
pub fn not16(a: u16) -> u16 {
    let mut result: u16 = 0;
    for i in 0..16 {
        let bit = (a >> i) & 1 == 1;
        if not(bit) {
            result |= 1 << i;
        }
    }
    result
}

/// 16-bit AND.
pub fn and16(a: u16, b: u16) -> u16 {
    let mut result: u16 = 0;
    for i in 0..16 {
        let bit_a = (a >> i) & 1 == 1;
        let bit_b = (b >> i) & 1 == 1;
        if and(bit_a, bit_b) {
            result |= 1 << i;
        }
    }
    result
}

/// 16-bit OR.
pub fn or16(a: u16, b: u16) -> u16 {
    let mut result: u16 = 0;
    for i in 0..16 {
        let bit_a = (a >> i) & 1 == 1;
        let bit_b = (b >> i) & 1 == 1;
        if or(bit_a, bit_b) {
            result |= 1 << i;
        }
    }
    result
}

/// 16-bit MUX - selects a (sel=0) or b (sel=1).
pub fn mux16(a: u16, b: u16, sel: bool) -> u16 {
    let mut result: u16 = 0;
    for i in 0..16 {
        let bit_a = (a >> i) & 1 == 1;
        let bit_b = (b >> i) & 1 == 1;
        if mux(bit_a, bit_b, sel) {
            result |= 1 << i;
        }
    }
    result
}

/// 8-way OR - true if any bit is set.
pub fn or8way(input: u8) -> bool {
    let b0 = (input >> 0) & 1 == 1;
    let b1 = (input >> 1) & 1 == 1;
    let b2 = (input >> 2) & 1 == 1;
    let b3 = (input >> 3) & 1 == 1;
    let b4 = (input >> 4) & 1 == 1;
    let b5 = (input >> 5) & 1 == 1;
    let b6 = (input >> 6) & 1 == 1;
    let b7 = (input >> 7) & 1 == 1;
    or(or(or(b0, b1), or(b2, b3)), or(or(b4, b5), or(b6, b7)))
}

/// 4-way 16-bit MUX. sel[0]=high bit, sel[1]=low bit.
pub fn mux4way16(a: u16, b: u16, c: u16, d: u16, sel: [bool; 2]) -> u16 {
    let ab = mux16(a, b, sel[1]);
    let cd = mux16(c, d, sel[1]);
    mux16(ab, cd, sel[0])
}

/// 8-way 16-bit MUX. sel[0]=high bit, sel[2]=low bit.
pub fn mux8way16(
    a: u16, b: u16, c: u16, d: u16,
    e: u16, f: u16, g: u16, h: u16,
    sel: [bool; 3]
) -> u16 {
    let abcd = mux4way16(a, b, c, d, [sel[1], sel[2]]);
    let efgh = mux4way16(e, f, g, h, [sel[1], sel[2]]);
    mux16(abcd, efgh, sel[0])
}

/// 4-way DMUX. sel[0]=high bit, sel[1]=low bit.
pub fn dmux4way(input: bool, sel: [bool; 2]) -> (bool, bool, bool, bool) {
    let (top, bottom) = dmux(input, sel[0]);
    let (a, b) = dmux(top, sel[1]);
    let (c, d) = dmux(bottom, sel[1]);
    (a, b, c, d)
}

/// 8-way DMUX. sel[0]=high bit, sel[2]=low bit.
pub fn dmux8way(input: bool, sel: [bool; 3]) -> (bool, bool, bool, bool, bool, bool, bool, bool) {
    let (top, bottom) = dmux(input, sel[0]);
    let (a, b, c, d) = dmux4way(top, [sel[1], sel[2]]);
    let (e, f, g, h) = dmux4way(bottom, [sel[1], sel[2]]);
    (a, b, c, d, e, f, g, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not16() {
        assert_eq!(not16(0x0000), 0xFFFF);
        assert_eq!(not16(0xFFFF), 0x0000);
        assert_eq!(not16(0x00FF), 0xFF00);
        assert_eq!(not16(0xAAAA), 0x5555);
    }

    #[test]
    fn test_and16() {
        assert_eq!(and16(0xFFFF, 0xFFFF), 0xFFFF);
        assert_eq!(and16(0x0000, 0xFFFF), 0x0000);
        assert_eq!(and16(0xCCCC, 0xAAAA), 0x8888);
    }

    #[test]
    fn test_or16() {
        assert_eq!(or16(0x0000, 0x0000), 0x0000);
        assert_eq!(or16(0xFFFF, 0x0000), 0xFFFF);
        assert_eq!(or16(0xCCCC, 0xAAAA), 0xEEEE);
    }

    #[test]
    fn test_mux16() {
        assert_eq!(mux16(0x1234, 0x5678, false), 0x1234);
        assert_eq!(mux16(0x1234, 0x5678, true), 0x5678);
    }

    #[test]
    fn test_or8way() {
        assert_eq!(or8way(0b00000000), false);
        assert_eq!(or8way(0b00000001), true);
        assert_eq!(or8way(0b10000000), true);
        assert_eq!(or8way(0b11111111), true);
        assert_eq!(or8way(0b00010000), true);
    }

    #[test]
    fn test_mux4way16() {
        let (a, b, c, d) = (0x1111, 0x2222, 0x3333, 0x4444);
        assert_eq!(mux4way16(a, b, c, d, [false, false]), a);
        assert_eq!(mux4way16(a, b, c, d, [false, true]), b);
        assert_eq!(mux4way16(a, b, c, d, [true, false]), c);
        assert_eq!(mux4way16(a, b, c, d, [true, true]), d);
    }

    #[test]
    fn test_mux8way16() {
        let v = [0x1111, 0x2222, 0x3333, 0x4444, 0x5555, 0x6666, 0x7777, 0x8888];
        assert_eq!(mux8way16(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], [false, false, false]), v[0]);
        assert_eq!(mux8way16(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], [false, false, true]), v[1]);
        assert_eq!(mux8way16(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], [false, true, false]), v[2]);
        assert_eq!(mux8way16(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], [false, true, true]), v[3]);
        assert_eq!(mux8way16(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], [true, false, false]), v[4]);
        assert_eq!(mux8way16(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], [true, false, true]), v[5]);
        assert_eq!(mux8way16(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], [true, true, false]), v[6]);
        assert_eq!(mux8way16(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], [true, true, true]), v[7]);
    }

    #[test]
    fn test_dmux4way() {
        assert_eq!(dmux4way(true, [false, false]), (true, false, false, false));
        assert_eq!(dmux4way(true, [false, true]), (false, true, false, false));
        assert_eq!(dmux4way(true, [true, false]), (false, false, true, false));
        assert_eq!(dmux4way(true, [true, true]), (false, false, false, true));
        assert_eq!(dmux4way(false, [true, true]), (false, false, false, false));
    }

    #[test]
    fn test_dmux8way() {
        assert_eq!(dmux8way(true, [false, false, false]), (true, false, false, false, false, false, false, false));
        assert_eq!(dmux8way(true, [false, false, true]), (false, true, false, false, false, false, false, false));
        assert_eq!(dmux8way(true, [false, true, false]), (false, false, true, false, false, false, false, false));
        assert_eq!(dmux8way(true, [false, true, true]), (false, false, false, true, false, false, false, false));
        assert_eq!(dmux8way(true, [true, false, false]), (false, false, false, false, true, false, false, false));
        assert_eq!(dmux8way(true, [true, false, true]), (false, false, false, false, false, true, false, false));
        assert_eq!(dmux8way(true, [true, true, false]), (false, false, false, false, false, false, true, false));
        assert_eq!(dmux8way(true, [true, true, true]), (false, false, false, false, false, false, false, true));
    }
}
