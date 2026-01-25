//! # Data Flip-Flop (DFF)
//!
//! The DFF is the primitive sequential element, like NAND is for combinational logic.
//! In real hardware, DFF is built from NAND gates with feedback loops.
//! For our simulation, we treat it as a primitive.
//!
//! ## Behavior
//!
//! ```text
//! out(t) = in(t-1)
//! ```
//!
//! The output is always the input from the PREVIOUS clock cycle.
//! This one-cycle delay is what makes sequential logic work.
//!
//! ## Timing Diagram
//!
//! ```text
//! Clock:  ──┬───┬───┬───┬───┬───
//!           │   │   │   │   │
//!    in:    0   1   1   0   1
//!   out:    ?   0   1   1   0    ← Output follows input with 1-cycle delay
//! ```

/// A Data Flip-Flop - the primitive sequential element.
///
/// The DFF stores a single bit. Its output is always the input
/// from the previous clock cycle.
#[derive(Debug, Clone, Default)]
pub struct DFF {
    /// Current output (from previous tick)
    state: bool,
    /// Input to be committed on next tick
    next: bool,
}

impl DFF {
    /// Creates a new DFF initialized to false.
    pub fn new() -> Self {
        todo!("Initialize both state and next to false")
    }

    /// Sets the input for the next clock cycle.
    ///
    /// This doesn't change the output immediately - it stages the value
    /// to be committed on the next tick().
    pub fn set(&mut self, input: bool) {
        todo!("Store input in next (to be committed on tick)")
    }

    /// Gets the current output.
    ///
    /// This returns the value that was set BEFORE the last tick().
    pub fn get(&self) -> bool {
        todo!("Return the current state")
    }

    /// Advances the clock - commits the staged input to the output.
    ///
    /// After tick(), get() will return what was set() before the tick.
    pub fn tick(&mut self) {
        todo!("Move next to state")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dff_initial_state() {
        let dff = DFF::new();
        assert_eq!(dff.get(), false);
    }

    #[test]
    fn test_dff_set_doesnt_change_output() {
        let mut dff = DFF::new();
        dff.set(true);
        // Output shouldn't change until tick
        assert_eq!(dff.get(), false);
    }

    #[test]
    fn test_dff_tick_commits_value() {
        let mut dff = DFF::new();
        dff.set(true);
        dff.tick();
        assert_eq!(dff.get(), true);
    }

    #[test]
    fn test_dff_sequence() {
        let mut dff = DFF::new();

        // Cycle 1: set true
        dff.set(true);
        dff.tick();
        assert_eq!(dff.get(), true);

        // Cycle 2: set false
        dff.set(false);
        dff.tick();
        assert_eq!(dff.get(), false);

        // Cycle 3: set true again
        dff.set(true);
        dff.tick();
        assert_eq!(dff.get(), true);
    }

    #[test]
    fn test_dff_holds_value_without_tick() {
        let mut dff = DFF::new();
        dff.set(true);
        dff.tick();

        // Value should persist even if we set something else without ticking
        dff.set(false);
        assert_eq!(dff.get(), true); // Still true until we tick
    }
}
