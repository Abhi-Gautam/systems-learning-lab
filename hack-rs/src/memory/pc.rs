//! # Program Counter (PC)
//!
//! The PC keeps track of which instruction to execute next.
//!
//! ## Operations
//!
//! The PC supports three operations (in priority order):
//!
//! 1. **reset**: PC = 0 (restart program)
//! 2. **load**: PC = input (jump to address)
//! 3. **inc**: PC = PC + 1 (next instruction)
//!
//! If multiple signals are true, higher priority wins:
//! reset > load > inc
//!
//! ## Typical CPU Usage
//!
//! ```text
//! Normal execution:     inc=1, load=0, reset=0  → PC++
//! Jump instruction:     inc=0, load=1, reset=0  → PC = jump_address
//! Program start/reset:  inc=0, load=0, reset=1  → PC = 0
//! ```
//!
//! ## Circuit Structure
//!
//! ```text
//!                    ┌─────┐
//!         ┌─────────►│ inc │────┐
//!         │          └─────┘    │
//!         │                     ▼
//!         │          ┌─────────────┐
//!         │          │   3-way     │
//!    out ─┼──────────┤   MUX       ├────► Register ────► out
//!         │          │  (priority) │
//!         │          └─────────────┘
//!         │                ▲  ▲  ▲
//!         │               inc│load│reset
//!         │                  │    │
//!         │                  │    └── 0
//!         │                  └─── input
//!         │
//!         └───────────────────────────────────────────────┘
//! ```

use super::register::Register;
use crate::arithmetic::inc16;
use crate::gates::mux16;

/// Program Counter - tracks the address of the next instruction.
///
/// Supports three operations:
/// - reset: Set to 0
/// - load: Set to input value
/// - inc: Increment by 1
///
/// Priority: reset > load > inc
/// If none are true, the value is unchanged.
#[derive(Debug, Clone, Default)]
pub struct PC {
    register: Register,
}

impl PC {
    /// Creates a new PC initialized to 0.
    pub fn new() -> Self {
        todo!("Create a PC with a fresh register")
    }

    /// Sets the PC control signals and input.
    ///
    /// # Arguments
    /// * `input` - Value to load if load=true
    /// * `inc` - Increment the PC
    /// * `load` - Load the input value
    /// * `reset` - Reset to 0
    ///
    /// Priority: reset > load > inc
    pub fn set(&mut self, input: u16, inc: bool, load: bool, reset: bool) {
        todo!("Implement priority logic: reset > load > inc")
        // Hint: Start with the current value
        // If inc, use inc16 to get current + 1
        // If load, use input instead
        // If reset, use 0 instead
        // Then set the register with load=true if any signal is active
    }

    /// Gets the current PC value.
    pub fn get(&self) -> u16 {
        todo!("Return the register's current value")
    }

    /// Advances the clock.
    pub fn tick(&mut self) {
        todo!("Tick the internal register")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pc_initial_state() {
        let pc = PC::new();
        assert_eq!(pc.get(), 0);
    }

    #[test]
    fn test_pc_increment() {
        let mut pc = PC::new();

        // Increment several times
        for expected in 1..=5 {
            pc.set(0, true, false, false); // inc=1
            pc.tick();
            assert_eq!(pc.get(), expected);
        }
    }

    #[test]
    fn test_pc_load() {
        let mut pc = PC::new();

        pc.set(100, false, true, false); // load=1
        pc.tick();
        assert_eq!(pc.get(), 100);

        pc.set(200, false, true, false);
        pc.tick();
        assert_eq!(pc.get(), 200);
    }

    #[test]
    fn test_pc_reset() {
        let mut pc = PC::new();

        // First get to a non-zero value
        pc.set(500, false, true, false);
        pc.tick();
        assert_eq!(pc.get(), 500);

        // Reset
        pc.set(0, false, false, true); // reset=1
        pc.tick();
        assert_eq!(pc.get(), 0);
    }

    #[test]
    fn test_pc_priority_reset_over_load() {
        let mut pc = PC::new();

        // Both reset and load are true - reset should win
        pc.set(100, false, true, true);
        pc.tick();
        assert_eq!(pc.get(), 0); // reset wins
    }

    #[test]
    fn test_pc_priority_load_over_inc() {
        let mut pc = PC::new();

        // Both load and inc are true - load should win
        pc.set(100, true, true, false);
        pc.tick();
        assert_eq!(pc.get(), 100); // load wins, not 1 (inc)
    }

    #[test]
    fn test_pc_priority_reset_over_all() {
        let mut pc = PC::new();

        // All signals true - reset should win
        pc.set(100, true, true, true);
        pc.tick();
        assert_eq!(pc.get(), 0); // reset wins
    }

    #[test]
    fn test_pc_no_change_when_all_false() {
        let mut pc = PC::new();

        // Set initial value
        pc.set(50, false, true, false);
        pc.tick();
        assert_eq!(pc.get(), 50);

        // All signals false - value unchanged
        pc.set(999, false, false, false);
        pc.tick();
        assert_eq!(pc.get(), 50); // Still 50
    }

    #[test]
    fn test_pc_overflow() {
        let mut pc = PC::new();

        // Set to max value
        pc.set(0xFFFF, false, true, false);
        pc.tick();
        assert_eq!(pc.get(), 0xFFFF);

        // Increment should wrap to 0
        pc.set(0, true, false, false);
        pc.tick();
        assert_eq!(pc.get(), 0);
    }

    #[test]
    fn test_pc_typical_usage() {
        let mut pc = PC::new();

        // Simulate normal execution with a jump
        // Start at 0, increment to 3
        for _ in 0..3 {
            pc.set(0, true, false, false);
            pc.tick();
        }
        assert_eq!(pc.get(), 3);

        // Jump to address 100
        pc.set(100, false, true, false);
        pc.tick();
        assert_eq!(pc.get(), 100);

        // Continue from there
        pc.set(0, true, false, false);
        pc.tick();
        assert_eq!(pc.get(), 101);
    }
}
