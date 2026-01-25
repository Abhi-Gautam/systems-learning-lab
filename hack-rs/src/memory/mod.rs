//! # Memory Module (Chapter 3)
//!
//! Sequential logic - components that remember state.
//!
//! ## Key Concept: The Clock
//!
//! Unlike combinational logic (gates), sequential logic has STATE.
//! State changes happen at clock ticks, not instantly.
//!
//! ```text
//! Time:    t0      t1      t2      t3
//!           │       │       │       │
//! Clock:  ──┴───────┴───────┴───────┴──
//!           ▲       ▲       ▲       ▲
//!         tick    tick    tick    tick
//!
//! DFF: output at time t = input at time t-1
//! ```
//!
//! ## Building Order
//!
//! ```text
//! DFF (primitive - given to us)
//!   ├── Bit (1-bit register)
//!   ├── Register (16-bit register)
//!   ├── RAM8 (8 registers)
//!   ├── RAM64 (8 RAM8s)
//!   ├── RAM512 (8 RAM64s)
//!   ├── RAM4K (8 RAM512s)
//!   ├── RAM16K (4 RAM4Ks)
//!   └── PC (Program Counter)
//! ```
//!
//! ## The tick() Pattern
//!
//! All sequential components follow the same pattern:
//! 1. Set inputs during the cycle
//! 2. Call tick() to commit changes
//! 3. Read outputs (which reflect the previous tick's inputs)

mod dff;
mod register;
mod ram;
mod pc;

pub use dff::DFF;
pub use register::{Bit, Register};
pub use ram::{RAM8, RAM64, RAM512, RAM4K, RAM16K};
pub use pc::PC;
