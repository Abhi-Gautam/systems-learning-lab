//! # Bit and Register
//!
//! Building up from DFF to 16-bit storage.
//!
//! ## Bit vs DFF
//!
//! DFF: Always stores input on every tick
//! Bit: Only stores input when load=1
//!
//! ```text
//! DFF:                          Bit (with load control):
//!
//!   in ──────► DFF ──► out        in ──┐
//!                                       ├──[MUX]──► DFF ──► out
//!                                 out ──┘     ▲           │
//!                                            load         │
//!                                              ◄──────────┘
//!
//! The MUX selects between:
//!   load=0: keep current output (feedback)
//!   load=1: take new input
//! ```
//!
//! ## Register
//!
//! A Register is just 16 Bits in parallel, all sharing the same load signal.

use super::dff::DFF;
use crate::gates::mux;

/// A 1-bit register with load control.
///
/// Only updates its value when load=true.
/// This is the building block for larger registers.
#[derive(Debug, Clone, Default)]
pub struct Bit {
    dff: DFF,
}

impl Bit {
    /// Creates a new Bit initialized to false.
    pub fn new() -> Self {
        todo!("Create a new Bit with a fresh DFF")
    }

    /// Sets the input and load signal.
    ///
    /// If load is true, the input will be stored on the next tick.
    /// If load is false, the current value will be retained.
    pub fn set(&mut self, input: bool, load: bool) {
        todo!("Use mux to select between current value and new input based on load")
    }

    /// Gets the current stored value.
    pub fn get(&self) -> bool {
        todo!("Return the DFF's current output")
    }

    /// Advances the clock.
    pub fn tick(&mut self) {
        todo!("Tick the internal DFF")
    }
}

/// A 16-bit register with load control.
///
/// Stores a 16-bit value. Only updates when load=true.
#[derive(Debug, Clone)]
pub struct Register {
    bits: [Bit; 16],
}

impl Default for Register {
    fn default() -> Self {
        Self::new()
    }
}

impl Register {
    /// Creates a new Register initialized to 0.
    pub fn new() -> Self {
        todo!("Create an array of 16 Bits")
    }

    /// Sets the 16-bit input and load signal.
    ///
    /// If load is true, the input will be stored on the next tick.
    pub fn set(&mut self, input: u16, load: bool) {
        todo!("Set each of the 16 bits with its corresponding input bit")
    }

    /// Gets the current stored 16-bit value.
    pub fn get(&self) -> u16 {
        todo!("Combine the 16 bits into a u16")
    }

    /// Advances the clock.
    pub fn tick(&mut self) {
        todo!("Tick all 16 internal bits")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Bit tests
    #[test]
    fn test_bit_initial_state() {
        let bit = Bit::new();
        assert_eq!(bit.get(), false);
    }

    #[test]
    fn test_bit_load_false_keeps_value() {
        let mut bit = Bit::new();
        bit.set(true, false);
        bit.tick();
        assert_eq!(bit.get(), false); // Unchanged because load=false
    }

    #[test]
    fn test_bit_load_true_stores_value() {
        let mut bit = Bit::new();
        bit.set(true, true);
        bit.tick();
        assert_eq!(bit.get(), true);
    }

    #[test]
    fn test_bit_holds_value() {
        let mut bit = Bit::new();

        // Store true
        bit.set(true, true);
        bit.tick();
        assert_eq!(bit.get(), true);

        // Try to overwrite with false, but load=false
        bit.set(false, false);
        bit.tick();
        assert_eq!(bit.get(), true); // Still true

        // Now actually overwrite
        bit.set(false, true);
        bit.tick();
        assert_eq!(bit.get(), false);
    }

    // Register tests
    #[test]
    fn test_register_initial_state() {
        let reg = Register::new();
        assert_eq!(reg.get(), 0);
    }

    #[test]
    fn test_register_load() {
        let mut reg = Register::new();

        // Load a value
        reg.set(0x1234, true);
        reg.tick();
        assert_eq!(reg.get(), 0x1234);

        // Try to overwrite without load
        reg.set(0xFFFF, false);
        reg.tick();
        assert_eq!(reg.get(), 0x1234); // Unchanged

        // Overwrite with load
        reg.set(0x5678, true);
        reg.tick();
        assert_eq!(reg.get(), 0x5678);
    }

    #[test]
    fn test_register_all_bits() {
        let mut reg = Register::new();

        // Test all 1s
        reg.set(0xFFFF, true);
        reg.tick();
        assert_eq!(reg.get(), 0xFFFF);

        // Test all 0s
        reg.set(0x0000, true);
        reg.tick();
        assert_eq!(reg.get(), 0x0000);

        // Test pattern
        reg.set(0xAAAA, true);
        reg.tick();
        assert_eq!(reg.get(), 0xAAAA);
    }
}
