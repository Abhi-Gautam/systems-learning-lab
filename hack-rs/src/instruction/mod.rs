//! # Instruction Module (Chapter 4)
//!
//! Understanding Hack machine language.
//!
//! ## Hack Instruction Set
//!
//! The Hack computer has only TWO instruction types:
//!
//! ### A-instruction (Address)
//!
//! ```text
//! Format: 0vvv vvvv vvvv vvvv
//!         │└─────────────────┴─── 15-bit value (0-32767)
//!         └─── 0 indicates A-instruction
//!
//! Effect: A = value
//!
//! Example: @100 → 0000 0000 0110 0100 → A = 100
//! ```
//!
//! ### C-instruction (Compute)
//!
//! ```text
//! Format: 111a cccc ccdd djjj
//!         ││││ ││││ ││││ │└┴┴─── jump (3 bits)
//!         ││││ ││││ ││└┴─┴────── dest (3 bits)
//!         ││││ │└┴┴─┴─────────── comp (6 bits)
//!         │││└─┴──────────────── a-bit (part of comp)
//!         └┴┴─────────────────── 111 indicates C-instruction
//!
//! Effect: dest = comp; jump
//!
//! Example: D=A+1;JGT → 1110 1101 1001 0001
//! ```
//!
//! ## Why This Matters
//!
//! Understanding instruction encoding is crucial for:
//! - Writing assemblers (Chapter 6)
//! - Understanding how CPUs decode instructions
//! - Debugging at the binary level

mod parser;

pub use parser::{
    Instruction,
    AInstruction,
    CInstruction,
    Dest,
    Comp,
    Jump,
    parse_instruction,
    encode_instruction,
};
