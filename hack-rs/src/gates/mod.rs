//! Gates module - all combinational logic built from NAND.

mod basic;
mod mux;
mod multibit;

pub use basic::{nand, not, and, or, xor};
pub use mux::{mux, dmux};
pub use multibit::{
    not16, and16, or16, mux16,
    or8way,
    mux4way16, mux8way16,
    dmux4way, dmux8way,
};
