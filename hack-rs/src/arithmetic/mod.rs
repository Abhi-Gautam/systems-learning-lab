//! # Arithmetic Module (Chapter 2)
//!
//! Arithmetic chips built from gates.
//!
//! ## Building Order
//!
//! ```text
//! Gates (from Chapter 1)
//!   ├── HalfAdder (a + b, no carry-in)
//!   ├── FullAdder (a + b + c, with carry-in)
//!   ├── Add16 (16-bit ripple-carry adder)
//!   ├── Inc16 (increment by 1)
//!   └── ALU (the heart of the CPU)
//! ```

mod adder;
mod alu;

pub use adder::{half_adder, full_adder, add16, inc16};
pub use alu::{alu, AluOutput};
