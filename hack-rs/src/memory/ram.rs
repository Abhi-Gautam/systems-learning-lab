//! # RAM (Random Access Memory)
//!
//! Building up memory from registers using address decoding.
//!
//! ## How RAM Works
//!
//! RAM = array of registers + address decoder
//!
//! ```text
//! Address bits select which register to read/write:
//!
//!        ┌─────────────────────────────┐
//!        │         RAM8                │
//!        │                             │
//!  in ───┤───► Reg0 ◄─────────┐       │
//!        │───► Reg1 ◄────┐    │       │
//!        │───► Reg2      │    │       │
//!        │───► Reg3      │    │       │
//!        │───► Reg4    DMUX   MUX ────┼──► out
//!        │───► Reg5      │    │       │
//!        │───► Reg6      │    │       │
//!        │───► Reg7 ◄────┘    │       │
//!        │         ▲          │       │
//!        └─────────┼──────────┼───────┘
//!                  │          │
//!               address    address
//!                 (3 bits for RAM8)
//! ```
//!
//! ## RAM Sizes
//!
//! | Name   | Registers | Address bits | Built from |
//! |--------|-----------|--------------|------------|
//! | RAM8   | 8         | 3            | 8 Registers |
//! | RAM64  | 64        | 6            | 8 RAM8s    |
//! | RAM512 | 512       | 9            | 8 RAM64s   |
//! | RAM4K  | 4096      | 12           | 8 RAM512s  |
//! | RAM16K | 16384     | 14           | 4 RAM4Ks   |
//!
//! The Hack computer has 16K words of RAM (addresses 0-16383).

use super::register::Register;
use crate::gates::{mux8way16, dmux8way, mux4way16, dmux4way};

/// Extracts bits from an address for indexing.
///
/// This is a helper for address decoding.
fn addr_bits_3(addr: u16) -> [bool; 3] {
    [
        (addr & 0b100) != 0,
        (addr & 0b010) != 0,
        (addr & 0b001) != 0,
    ]
}

fn addr_bits_2(addr: u16) -> [bool; 2] {
    [
        (addr & 0b10) != 0,
        (addr & 0b01) != 0,
    ]
}

/// RAM8 - 8 registers (24 bytes).
///
/// Address: 3 bits (0-7)
#[derive(Debug, Clone)]
pub struct RAM8 {
    registers: [Register; 8],
}

impl Default for RAM8 {
    fn default() -> Self {
        Self::new()
    }
}

impl RAM8 {
    pub fn new() -> Self {
        todo!("Create 8 registers")
    }

    /// Sets input for a specific address.
    ///
    /// If load is true, the value at `address` will be updated on tick.
    /// Address is 3 bits (0-7).
    pub fn set(&mut self, input: u16, address: u16, load: bool) {
        todo!("Use dmux8way to route load signal to correct register")
    }

    /// Gets value at a specific address.
    pub fn get(&self, address: u16) -> u16 {
        todo!("Use mux8way16 to select output from correct register")
    }

    pub fn tick(&mut self) {
        todo!("Tick all 8 registers")
    }
}

/// RAM64 - 64 registers (192 bytes).
///
/// Address: 6 bits (0-63)
/// Structure: 8 RAM8s, upper 3 bits select which RAM8, lower 3 bits index within
#[derive(Debug, Clone)]
pub struct RAM64 {
    rams: [RAM8; 8],
}

impl Default for RAM64 {
    fn default() -> Self {
        Self::new()
    }
}

impl RAM64 {
    pub fn new() -> Self {
        todo!("Create 8 RAM8s")
    }

    pub fn set(&mut self, input: u16, address: u16, load: bool) {
        todo!("Upper 3 bits select RAM8, lower 3 bits are passed through")
    }

    pub fn get(&self, address: u16) -> u16 {
        todo!("Upper 3 bits select RAM8, lower 3 bits select register within")
    }

    pub fn tick(&mut self) {
        todo!("Tick all 8 RAM8s")
    }
}

/// RAM512 - 512 registers (~1.5 KB).
///
/// Address: 9 bits (0-511)
#[derive(Debug, Clone)]
pub struct RAM512 {
    rams: [RAM64; 8],
}

impl Default for RAM512 {
    fn default() -> Self {
        Self::new()
    }
}

impl RAM512 {
    pub fn new() -> Self {
        todo!("Create 8 RAM64s")
    }

    pub fn set(&mut self, input: u16, address: u16, load: bool) {
        todo!("Upper 3 bits select RAM64, lower 6 bits passed through")
    }

    pub fn get(&self, address: u16) -> u16 {
        todo!("Upper 3 bits select RAM64, lower 6 bits select within")
    }

    pub fn tick(&mut self) {
        todo!("Tick all 8 RAM64s")
    }
}

/// RAM4K - 4096 registers (~12 KB).
///
/// Address: 12 bits (0-4095)
#[derive(Debug, Clone)]
pub struct RAM4K {
    rams: [RAM512; 8],
}

impl Default for RAM4K {
    fn default() -> Self {
        Self::new()
    }
}

impl RAM4K {
    pub fn new() -> Self {
        todo!("Create 8 RAM512s")
    }

    pub fn set(&mut self, input: u16, address: u16, load: bool) {
        todo!("Upper 3 bits select RAM512, lower 9 bits passed through")
    }

    pub fn get(&self, address: u16) -> u16 {
        todo!("Upper 3 bits select RAM512, lower 9 bits select within")
    }

    pub fn tick(&mut self) {
        todo!("Tick all 8 RAM512s")
    }
}

/// RAM16K - 16384 registers (~48 KB).
///
/// Address: 14 bits (0-16383)
///
/// Note: This uses 4 RAM4Ks (not 8) because 16K = 4 * 4K.
/// So we only need 2 address bits for selection.
#[derive(Debug, Clone)]
pub struct RAM16K {
    rams: [RAM4K; 4],
}

impl Default for RAM16K {
    fn default() -> Self {
        Self::new()
    }
}

impl RAM16K {
    pub fn new() -> Self {
        todo!("Create 4 RAM4Ks")
    }

    pub fn set(&mut self, input: u16, address: u16, load: bool) {
        todo!("Upper 2 bits select RAM4K, lower 12 bits passed through")
    }

    pub fn get(&self, address: u16) -> u16 {
        todo!("Upper 2 bits select RAM4K, lower 12 bits select within")
    }

    pub fn tick(&mut self) {
        todo!("Tick all 4 RAM4Ks")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RAM8 tests
    #[test]
    fn test_ram8_initial_state() {
        let ram = RAM8::new();
        for addr in 0..8 {
            assert_eq!(ram.get(addr), 0);
        }
    }

    #[test]
    fn test_ram8_write_read() {
        let mut ram = RAM8::new();

        // Write to address 3
        ram.set(0x1234, 3, true);
        ram.tick();
        assert_eq!(ram.get(3), 0x1234);

        // Other addresses unchanged
        assert_eq!(ram.get(0), 0);
        assert_eq!(ram.get(7), 0);
    }

    #[test]
    fn test_ram8_all_addresses() {
        let mut ram = RAM8::new();

        // Write different values to all addresses
        for addr in 0..8 {
            ram.set(addr * 0x1000, addr, true);
            ram.tick();
        }

        // Verify all values
        for addr in 0..8 {
            assert_eq!(ram.get(addr), addr * 0x1000);
        }
    }

    // RAM64 tests
    #[test]
    fn test_ram64_write_read() {
        let mut ram = RAM64::new();

        ram.set(0xABCD, 42, true);
        ram.tick();
        assert_eq!(ram.get(42), 0xABCD);
        assert_eq!(ram.get(0), 0);
        assert_eq!(ram.get(63), 0);
    }

    #[test]
    fn test_ram64_boundaries() {
        let mut ram = RAM64::new();

        // First address
        ram.set(0x1111, 0, true);
        ram.tick();

        // Last address
        ram.set(0x2222, 63, true);
        ram.tick();

        assert_eq!(ram.get(0), 0x1111);
        assert_eq!(ram.get(63), 0x2222);
    }

    // RAM512 tests
    #[test]
    fn test_ram512_write_read() {
        let mut ram = RAM512::new();

        ram.set(0x5678, 256, true);
        ram.tick();
        assert_eq!(ram.get(256), 0x5678);
        assert_eq!(ram.get(0), 0);
    }

    // RAM4K tests
    #[test]
    fn test_ram4k_write_read() {
        let mut ram = RAM4K::new();

        ram.set(0x9ABC, 2048, true);
        ram.tick();
        assert_eq!(ram.get(2048), 0x9ABC);
        assert_eq!(ram.get(0), 0);
    }

    // RAM16K tests
    #[test]
    fn test_ram16k_write_read() {
        let mut ram = RAM16K::new();

        ram.set(0xDEF0, 8192, true);
        ram.tick();
        assert_eq!(ram.get(8192), 0xDEF0);
        assert_eq!(ram.get(0), 0);
    }

    #[test]
    fn test_ram16k_boundaries() {
        let mut ram = RAM16K::new();

        // First address
        ram.set(0x1111, 0, true);
        ram.tick();

        // Last address (16383 = 0x3FFF)
        ram.set(0x2222, 16383, true);
        ram.tick();

        // Middle
        ram.set(0x3333, 8000, true);
        ram.tick();

        assert_eq!(ram.get(0), 0x1111);
        assert_eq!(ram.get(16383), 0x2222);
        assert_eq!(ram.get(8000), 0x3333);
    }

    #[test]
    fn test_ram_no_load_no_change() {
        let mut ram = RAM8::new();

        // Write initial value
        ram.set(0x1234, 5, true);
        ram.tick();

        // Try to overwrite without load
        ram.set(0xFFFF, 5, false);
        ram.tick();

        // Value unchanged
        assert_eq!(ram.get(5), 0x1234);
    }
}
