//! Integration tests for memory module (Chapter 3)

use hack_rs::memory::*;

// ============================================
// DFF (Data Flip-Flop)
// ============================================

#[test]
fn test_dff_initial_state() {
    let dff = DFF::new();
    assert_eq!(dff.get(), false, "DFF should initialize to false");
}

#[test]
fn test_dff_delayed_output() {
    let mut dff = DFF::new();

    // Set input but don't tick - output unchanged
    dff.set(true);
    assert_eq!(dff.get(), false, "Output shouldn't change until tick");

    // Tick - now output changes
    dff.tick();
    assert_eq!(dff.get(), true, "Output should reflect previous input after tick");
}

#[test]
fn test_dff_sequence() {
    let mut dff = DFF::new();
    let inputs = [true, true, false, true, false, false, true];

    let mut prev_input = false;
    for &input in &inputs {
        dff.set(input);
        dff.tick();
        assert_eq!(dff.get(), prev_input, "Output should be delayed by one cycle");
        prev_input = input;
    }
}

// ============================================
// Bit (1-bit Register)
// ============================================

#[test]
fn test_bit_initial_state() {
    let bit = Bit::new();
    assert_eq!(bit.get(), false);
}

#[test]
fn test_bit_load_control() {
    let mut bit = Bit::new();

    // load=false: value shouldn't change
    bit.set(true, false);
    bit.tick();
    assert_eq!(bit.get(), false, "Value should not change when load=false");

    // load=true: value should change
    bit.set(true, true);
    bit.tick();
    assert_eq!(bit.get(), true, "Value should change when load=true");

    // load=false again: retain value
    bit.set(false, false);
    bit.tick();
    assert_eq!(bit.get(), true, "Value should be retained when load=false");
}

// ============================================
// Register (16-bit)
// ============================================

#[test]
fn test_register_initial_state() {
    let reg = Register::new();
    assert_eq!(reg.get(), 0);
}

#[test]
fn test_register_load_store() {
    let mut reg = Register::new();

    reg.set(0x1234, true);
    reg.tick();
    assert_eq!(reg.get(), 0x1234);

    reg.set(0xABCD, true);
    reg.tick();
    assert_eq!(reg.get(), 0xABCD);
}

#[test]
fn test_register_load_control() {
    let mut reg = Register::new();

    reg.set(0x1234, true);
    reg.tick();

    // Try to overwrite with load=false
    reg.set(0xFFFF, false);
    reg.tick();
    assert_eq!(reg.get(), 0x1234, "Value unchanged when load=false");

    // Overwrite with load=true
    reg.set(0x5678, true);
    reg.tick();
    assert_eq!(reg.get(), 0x5678);
}

// ============================================
// RAM8
// ============================================

#[test]
fn test_ram8_initial_state() {
    let ram = RAM8::new();
    for addr in 0..8 {
        assert_eq!(ram.get(addr), 0, "RAM8[{}] should initialize to 0", addr);
    }
}

#[test]
fn test_ram8_write_read_single() {
    let mut ram = RAM8::new();

    ram.set(0x1234, 3, true);
    ram.tick();

    assert_eq!(ram.get(3), 0x1234);
    assert_eq!(ram.get(0), 0, "Other addresses should be unchanged");
    assert_eq!(ram.get(7), 0);
}

#[test]
fn test_ram8_all_addresses() {
    let mut ram = RAM8::new();

    // Write unique values to all addresses
    for addr in 0..8 {
        ram.set(addr * 0x1111, addr, true);
        ram.tick();
    }

    // Verify all values
    for addr in 0..8 {
        assert_eq!(ram.get(addr), addr * 0x1111, "RAM8[{}] incorrect", addr);
    }
}

#[test]
fn test_ram8_no_write_when_load_false() {
    let mut ram = RAM8::new();

    ram.set(0x1234, 5, true);
    ram.tick();

    ram.set(0xFFFF, 5, false);
    ram.tick();

    assert_eq!(ram.get(5), 0x1234, "Value unchanged when load=false");
}

// ============================================
// RAM64
// ============================================

#[test]
fn test_ram64_basic() {
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

    ram.set(0x1111, 0, true);
    ram.tick();
    ram.set(0x2222, 63, true);
    ram.tick();

    assert_eq!(ram.get(0), 0x1111);
    assert_eq!(ram.get(63), 0x2222);
}

#[test]
fn test_ram64_address_isolation() {
    let mut ram = RAM64::new();

    // Write to addresses in different RAM8 banks
    ram.set(0x1000, 0, true);   // Bank 0
    ram.tick();
    ram.set(0x2000, 8, true);   // Bank 1
    ram.tick();
    ram.set(0x3000, 16, true);  // Bank 2
    ram.tick();

    assert_eq!(ram.get(0), 0x1000);
    assert_eq!(ram.get(8), 0x2000);
    assert_eq!(ram.get(16), 0x3000);
}

// ============================================
// RAM512
// ============================================

#[test]
fn test_ram512_basic() {
    let mut ram = RAM512::new();

    ram.set(0x5678, 256, true);
    ram.tick();

    assert_eq!(ram.get(256), 0x5678);
    assert_eq!(ram.get(0), 0);
    assert_eq!(ram.get(511), 0);
}

// ============================================
// RAM4K
// ============================================

#[test]
fn test_ram4k_basic() {
    let mut ram = RAM4K::new();

    ram.set(0x9ABC, 2048, true);
    ram.tick();

    assert_eq!(ram.get(2048), 0x9ABC);
    assert_eq!(ram.get(0), 0);
    assert_eq!(ram.get(4095), 0);
}

// ============================================
// RAM16K
// ============================================

#[test]
fn test_ram16k_basic() {
    let mut ram = RAM16K::new();

    ram.set(0xDEF0, 8192, true);
    ram.tick();

    assert_eq!(ram.get(8192), 0xDEF0);
    assert_eq!(ram.get(0), 0);
}

#[test]
fn test_ram16k_full_range() {
    let mut ram = RAM16K::new();

    // Test all four RAM4K banks
    let test_cases = [
        (0, 0x1111),      // Bank 0
        (4096, 0x2222),   // Bank 1
        (8192, 0x3333),   // Bank 2
        (12288, 0x4444),  // Bank 3
        (16383, 0x5555),  // Last address
    ];

    for (addr, value) in test_cases {
        ram.set(value, addr, true);
        ram.tick();
    }

    for (addr, value) in test_cases {
        assert_eq!(ram.get(addr), value, "RAM16K[{}] incorrect", addr);
    }
}

// ============================================
// PC (Program Counter)
// ============================================

#[test]
fn test_pc_initial_state() {
    let pc = PC::new();
    assert_eq!(pc.get(), 0);
}

#[test]
fn test_pc_increment() {
    let mut pc = PC::new();

    for expected in 1..=10 {
        pc.set(0, true, false, false); // inc only
        pc.tick();
        assert_eq!(pc.get(), expected);
    }
}

#[test]
fn test_pc_load() {
    let mut pc = PC::new();

    pc.set(100, false, true, false); // load only
    pc.tick();
    assert_eq!(pc.get(), 100);

    pc.set(500, false, true, false);
    pc.tick();
    assert_eq!(pc.get(), 500);
}

#[test]
fn test_pc_reset() {
    let mut pc = PC::new();

    // Get to non-zero
    pc.set(1000, false, true, false);
    pc.tick();
    assert_eq!(pc.get(), 1000);

    // Reset
    pc.set(9999, false, false, true); // reset only (input ignored)
    pc.tick();
    assert_eq!(pc.get(), 0);
}

#[test]
fn test_pc_priority_reset_beats_load() {
    let mut pc = PC::new();

    // Both reset and load
    pc.set(500, false, true, true);
    pc.tick();
    assert_eq!(pc.get(), 0, "Reset should have priority over load");
}

#[test]
fn test_pc_priority_load_beats_inc() {
    let mut pc = PC::new();

    // Both load and inc
    pc.set(500, true, true, false);
    pc.tick();
    assert_eq!(pc.get(), 500, "Load should have priority over inc");
}

#[test]
fn test_pc_priority_reset_beats_all() {
    let mut pc = PC::new();

    // All three signals
    pc.set(500, true, true, true);
    pc.tick();
    assert_eq!(pc.get(), 0, "Reset should have highest priority");
}

#[test]
fn test_pc_no_change_when_all_false() {
    let mut pc = PC::new();

    pc.set(50, false, true, false);
    pc.tick();
    assert_eq!(pc.get(), 50);

    // No signals
    pc.set(999, false, false, false);
    pc.tick();
    assert_eq!(pc.get(), 50, "Value should be unchanged");
}

#[test]
fn test_pc_overflow() {
    let mut pc = PC::new();

    pc.set(0xFFFF, false, true, false);
    pc.tick();
    assert_eq!(pc.get(), 0xFFFF);

    pc.set(0, true, false, false); // inc
    pc.tick();
    assert_eq!(pc.get(), 0, "Should wrap on overflow");
}

#[test]
fn test_pc_typical_execution() {
    let mut pc = PC::new();

    // Execute a few instructions normally
    for i in 1..=5 {
        pc.set(0, true, false, false);
        pc.tick();
        assert_eq!(pc.get(), i);
    }

    // Jump to address 100
    pc.set(100, false, true, false);
    pc.tick();
    assert_eq!(pc.get(), 100);

    // Continue from there
    pc.set(0, true, false, false);
    pc.tick();
    assert_eq!(pc.get(), 101);

    // Reset
    pc.set(0, false, false, true);
    pc.tick();
    assert_eq!(pc.get(), 0);
}
