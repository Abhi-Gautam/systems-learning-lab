# Day 1 — Bits, Bytes, Words, and Addressable Storage

## Unit Built

One tiny byte-level memory model inside the no-dependency Rust runner:

```rust
let byte = 0b1010_1100_u8;
let ram = [0x12_u8, 0x34, 0x56, 0x78];
```

This is not stack, heap, ownership, or OS virtual memory yet. It is only:

```text
address → byte-sized storage cell → 8 stored bits
```

## Run

From the repository root:

```bash
cd labs/cs-from-silicone/core
cargo run -- day01
```

Optional direct build from `labs/cs-from-silicone`:

```bash
rustc core/src/main.rs -O -o /tmp/memory_lab
/tmp/memory_lab day01
```

## Concept Ladder

```text
transistor/latch stores one signal
→ bit stores 0 or 1
→ byte is 8 bits
→ address selects a byte-sized storage cell
→ word is several neighboring bytes interpreted together
```

## Physical Model

```text
address selector
      │
      ▼
┌─────────────┬─────────────┬─────────────┬─────────────┐
│ address 0   │ address 1   │ address 2   │ address 3   │
│ one byte    │ one byte    │ one byte    │ one byte    │
│ 0001_0010   │ 0011_0100   │ 0101_0110   │ 0111_1000   │
└─────────────┴─────────────┴─────────────┴─────────────┘
```

A word is not a separate physical storage kind. It is an interpretation over
neighboring bytes, such as saying addresses `0..4` together form a 32-bit value.

## Output Trace

The Day 1 runner prints three things:

1. One byte in hex and binary.
2. Common word sizes: `u8`, `u16`, `u32`, `u64`.
3. A tiny RAM dump with address, hex value, and bit pattern.

Expected first byte:

```text
one byte value: 0xac
bit positions:  7 6 5 4 3 2 1 0
bit values:     1010 1100
```

Expected tiny storage idea:

```text
address  hex   bits
0x00    0x12  0001_0010
0x01    0x34  0011_0100
0x02    0x56  0101_0110
0x03    0x78  0111_1000
```

## Hands-On Edits

Edit `core/src/main.rs`, inside `day01_process_memory_map` only.

1. Change the single byte:

```rust
let byte = 0b1010_1100_u8;
```

Before rerunning, predict:

- hex output
- bit positions that become `1`
- bit positions that become `0`

2. Change the tiny RAM bytes:

```rust
let ram = [0x12_u8, 0x34, 0x56, 0x78];
```

Before rerunning, predict the whole address/hex/bits table.

## Day 1 Rules

Do say:

```text
address
bit
byte
word interpretation
neighboring bytes
selector/name for a storage cell
```

Do not use yet:

```text
stack
heap
Box
Vec ownership
virtual memory
page table
allocator
runtime object layout
```

Those concepts come later. Day 1 is deliberately below them.

## Reading

- `N2T`: register/RAM chip chapters.
- `COD`: data representation basics.

## Stop When You Can Say

```text
An address is a selector/name for a storage cell.
A byte is 8 bits at one address.
A word is a multi-byte interpretation, not a new kind of storage.
```

## Day 1 Ready Checklist

- [ ] `cargo test` passes in `labs/cs-from-silicone/core`.
- [ ] `cargo run -- day01` runs once unchanged.
- [ ] You can explain every output line without saying stack or heap.
- [ ] You changed `byte`, predicted hex/bits, then reran.
- [ ] You changed `ram`, predicted the address table, then reran.
