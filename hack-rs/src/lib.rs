//! # hack-rs
//!
//! A Hack computer simulator built from NAND gates.
//!
//! This is a Nand2Tetris implementation in Rust, building every component
//! from the primitive NAND gate up to a working computer.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │  instruction (Chapter 4)                │  ← A/C instruction parsing
//! ├─────────────────────────────────────────┤
//! │  memory (Chapter 3)                     │  ← DFF, Register, RAM, PC
//! ├─────────────────────────────────────────┤
//! │  arithmetic (Chapter 2)                 │  ← Adders, ALU
//! ├─────────────────────────────────────────┤
//! │  gates (Chapter 1)                      │  ← All from NAND
//! └─────────────────────────────────────────┘
//! ```

pub mod gates;
pub mod arithmetic;
pub mod memory;
pub mod instruction;
