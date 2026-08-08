# cs-from-silicone

Bottom-up computer-systems learning track for building a machine model from
physical storage and signals up to CPU behavior, OS/runtime behavior, and
language-level memory concepts.

## Structure

- `core/`: canonical no-dependency Rust lab runner
- `docs/`: curriculum specs, day plans, and milestone notes
- `labs/`: isolated experiments and alternate designs
- `references/`: notes from books or upstream systems used for study
- `external/`: local read-only reference material, if needed later
- `bench/`: optional timing runs and benchmark notes
- `tools/`: helper scripts, if needed later

## Current Focus

**Day 1 — Bits, Bytes, Words, and Addressable Storage**

Start here:

- Day 1 guide: `docs/day-01-bits-bytes-words.md`
- Full curriculum: `docs/curriculum.md`
- Runnable artifact: `core/src/main.rs`

## Quick Start

From the repository root:

```bash
cd labs/cs-from-silicone/core
cargo run -- day01
cargo test
```

The runner intentionally uses only the Rust standard library. If you want to run
it without Cargo, compile the single file from `labs/cs-from-silicone`:

```bash
rustc core/src/main.rs -O -o /tmp/memory_lab
/tmp/memory_lab day01
```

## Study Rule

Do not start Day 1 with `Box`, `Vec`, heap, stack, ownership, or runtime terms.
Use this ladder:

```text
physical thing
→ CPU/ISA view
→ OS/runtime view only when needed
→ Rust/C/JS concept last
```

For Day 1, stop at: bit, byte, address, neighboring bytes, and word
interpretation.
