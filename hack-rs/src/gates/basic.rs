//! Basic gates built from NAND.

/// NAND gate - the primitive from which all other gates are built.
pub fn nand(a: bool, b: bool) -> bool {
    !(a && b)
}

/// NOT gate - inverts input.
pub fn not(a: bool) -> bool {
    nand(a, a)
}

/// AND gate - true when both inputs are true.
pub fn and(a: bool, b: bool) -> bool {
    not(nand(a, b))
}

/// OR gate - true when any input is true.
pub fn or(a: bool, b: bool) -> bool {
    nand(not(a), not(b))
}

/// XOR gate - true when inputs differ.
pub fn xor(a: bool, b: bool) -> bool {
    and(or(a, b), nand(a, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nand() {
        assert_eq!(nand(false, false), true);
        assert_eq!(nand(false, true), true);
        assert_eq!(nand(true, false), true);
        assert_eq!(nand(true, true), false);
    }

    #[test]
    fn test_not() {
        assert_eq!(not(false), true);
        assert_eq!(not(true), false);
    }

    #[test]
    fn test_and() {
        assert_eq!(and(false, false), false);
        assert_eq!(and(false, true), false);
        assert_eq!(and(true, false), false);
        assert_eq!(and(true, true), true);
    }

    #[test]
    fn test_or() {
        assert_eq!(or(false, false), false);
        assert_eq!(or(false, true), true);
        assert_eq!(or(true, false), true);
        assert_eq!(or(true, true), true);
    }

    #[test]
    fn test_xor() {
        assert_eq!(xor(false, false), false);
        assert_eq!(xor(false, true), true);
        assert_eq!(xor(true, false), true);
        assert_eq!(xor(true, true), false);
    }
}
