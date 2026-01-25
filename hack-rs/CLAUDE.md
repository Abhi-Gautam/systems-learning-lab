# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Hack computer simulator built in Rust. Every component is constructed from the primitive NAND gate up to a working computer (Nand2Tetris implementation).

## Commands

```bash
cargo build                   # Compile
cargo run                     # Run smoke tests
cargo test                    # Full test suite
cargo test gates              # Test specific module
cargo test test_nand          # Test specific function
cargo test -- --nocapture     # Show println! output
```

## Architecture

```
gates (Ch1)  →  arithmetic (Ch2)  →  memory (Ch3)  →  instruction (Ch4)
   │                  │                   │                  │
   NAND          HalfAdder             DFF            A-instruction
   NOT           FullAdder             Bit            C-instruction
   AND           Add16                 Register
   OR            Inc16                 RAM8-16K
   XOR           ALU                   PC
   MUX/DMUX
   16-bit/multi-way
```

## Type Conventions

- `bool` - single bit
- `u16` - 16-bit value
- `[bool; N]` - multi-bit selector
- Tuples - multi-output gates
- Structs with `new()`, `set()`, `get()`, `tick()` - sequential components

## Selector Bit Ordering

Arrays are **MSB-first**: `[sel[0], sel[1]]` where `sel[0]` is the high bit.

```
[false, true]  = 01 = index 1
[true, false]  = 10 = index 2
```

## Code Style (Post-Implementation)

After a module is fully implemented and tested:
1. Remove all example comments and verbose explanations from source files
2. Keep only essential documentation (function signatures, brief purpose)
3. Structure code for long-term maintainability (decades-scale)
4. Inline tests remain; integration tests in `tests/`
