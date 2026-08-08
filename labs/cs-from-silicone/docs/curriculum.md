# Memory From Silicon to Rust/C

This is a special track alongside the normal reading scheduler. The curriculum
starts from physical memory and climbs upward. Do not begin with `Box`, `Vec`,
heap, or stack. First build the machine model: bits, bytes, addresses,
registers, memory hierarchy, cache lines, and read/write protocols.

## Repository Shape

- `docs/`: curriculum specs, day plans, and milestone notes
- `labs/`: isolated experiments and alternate designs

## Core Artifact

From the repository root:

```bash
cd labs/cs-from-silicone/core
cargo run -- day01
cargo run -- day12
cargo run -- all
```

Test it:

```bash
cargo test
```

The runner is intentionally one Rust file with no external crates. macOS runs
the core experiments. Raspberry Pi/Linux unlocks `/proc/self/maps`, RSS
observations, and better OS-level inspection.

Optional single-file build from `labs/cs-from-silicone`:

```bash
rustc core/src/main.rs -O -o /tmp/memory_lab
/tmp/memory_lab day01
/tmp/memory_lab all
```

## Current Scope

Day 1 is ready as the canonical starting point:

- `docs/day-01-bits-bytes-words.md`
- `core/src/main.rs`, function `day01_process_memory_map`

Days 1-5 are the active pilot block. Days 6-30 still exist in the runner as
useful experiments, but their curriculum order is pending rewrite after feedback
from this first block.

The rule for each active day:

```text
physical thing
→ CPU/ISA view
→ OS/runtime view only when needed
→ Rust/C concept last
```

## How to Study Days 1-5

Use a 90-minute block:

1. Run the day's command once and read the output.
2. Say what physically exists in the machine model.
3. Predict the output before changing anything.
4. Edit only the suggested constants or values.
5. Read the listed book section.
6. Explain the result without using stack/heap language unless that day
   explicitly introduces it.

For this first block, learn these questions:

- What is the stored bit/byte/word?
- What selects the storage cell?
- What moves over data lines?
- What is in a register versus in memory?
- What does the cache line bring along?
- Which part is physics, which part is convention?

## Days 1-5: Physical Memory Before Software Memory

### Day 1: Bits, Bytes, Words, and Addressable Storage

Detailed guide: `docs/day-01-bits-bytes-words.md`

Run:

```bash
cargo run -- day01
```

Goal:

```text
Understand what memory means before stack, heap, OS, or language runtime.
```

Concept ladder:

```text
transistor/latch stores one signal
→ bit stores 0 or 1
→ byte is 8 bits
→ address selects a byte-sized storage cell
→ word is several neighboring bytes interpreted together
```

Hands-on:

- Change the byte value in `core/src/main.rs`, function
  `day01_process_memory_map`.
- Change the `ram` byte array.
- Predict the address/hex/bits table before rerunning.

Read:

- `N2T`: registers/RAM chips.
- `COD`: data representation basics.

Stop when you can say:

```text
An address is a selector/name for a storage cell.
A byte is 8 bits at one address.
A word is a multi-byte interpretation, not a new kind of storage.
```

### Day 2: Registers vs Memory

Run:

```bash
cargo run -- day02
```

Goal:

```text
Understand why CPUs have registers even though memory already stores data.
```

Concept ladder:

```text
memory stores many values
→ register file stores a few very fast values
→ ALU reads registers
→ LOAD copies memory cell into register
→ STORE copies register back into memory cell
```

Hands-on:

- Change the initial RAM bytes `[7, 5, 0, 0]` in the Day 2 function.
- Predict `R2` and `RAM[2]`.
- Add one extra load/add/store step only after you can narrate the first one.

Read:

- `N2T`: A register, D register, RAM.
- `COD`: datapath/register file/load-store idea.

Stop when you can say:

```text
Memory is storage.
Registers are the CPU's immediate working slots.
The ALU does arithmetic on register values.
Load/store is movement between the two.
```

### Day 3: SRAM, DRAM, and Why Memory Has Levels

Run:

```bash
cargo run -- day03
```

Goal:

```text
Understand why there is not one giant fastest memory.
```

Concept ladder:

```text
registers: flip-flops, tiny, fastest
→ SRAM: cache, fast, expensive per bit
→ DRAM: main memory, dense, slower, refreshed
→ SSD/disk: persistent storage, not direct CPU load/store memory
```

Hands-on:

- Change the small and large array sizes.
- Run more than once; timings are noisy.
- Explain the trend, not the exact number.

Read:

- `COD` Ch.5 memory hierarchy.
- Existing `COD` notes on hierarchy.

Stop when you can say:

```text
The memory hierarchy is not a software invention.
It exists because fast storage costs more area and power per bit.
```

### Day 4: Cache Lines and Locality

Run:

```bash
cargo run -- day04
```

Goal:

```text
Understand that the CPU often moves memory in chunks larger than the value requested.
```

Concept ladder:

```text
program asks for address X
→ cache miss asks lower level for a block
→ a cache line arrives, commonly 64 bytes
→ nearby addresses become cheap
→ strided/random access wastes fetched bytes
```

Hands-on:

- Change stride values.
- Predict which strides waste more of each cache line.
- Do not overfit the exact timing; explain the access pattern.

Read:

- `COD` Ch.5 cache blocks/lines.
- `DBI` row vs column locality notes.

Stop when you can say:

```text
Locality means using bytes near bytes you already paid to fetch.
Layout matters because cache lines move contiguous chunks.
```

### Day 5: Address, Data, and Control Signals

Run:

```bash
cargo run -- day05
```

Goal:

```text
Understand what must happen for "load/store address X" to work.
```

Concept ladder:

```text
address lines carry which cell
→ control line says READ or WRITE
→ data lines carry the byte
→ decoder selects the cell
→ memory returns or stores the bits
```

Hands-on:

- Change `address` and `value`.
- Add one more write/read pair.
- Predict address lines and data lines.

Read:

- `N2T` RAM/register chapters.
- `COD` memory/datapath basics.

Stop when you can say:

```text
A memory operation is a protocol: address, control, data.
This is below OS memory, heap, stack, and Rust ownership.
```

## Pending Curriculum Rewrite

Days 6-30 are retained as runnable experiments, but do not treat their current
order as the final learning path. After the first pilot block, rewrite them into
small day guides under `docs/` before using them as the canonical schedule.

## Raspberry Pi/Linux Add-Ons

On the Pi, compile with optimizations from `labs/cs-from-silicone`:

```bash
rustc core/src/main.rs -O -o /tmp/memory_lab
```

Useful external checks:

```bash
cat /proc/$$/maps
getconf PAGESIZE
perf stat /tmp/memory_lab day12
perf stat -e cache-references,cache-misses /tmp/memory_lab day14
```

If `perf` is not installed or lacks permission, the Rust timings are still
useful. Treat hardware counters as confirmation, not as the first source of
understanding.

## Completion Criteria

You are ready to move from this bootcamp into Linux kernel memory-management
reading when you can explain:

- Why a `Vec<T>` owner and its buffer have different addresses.
- Why moving a `Vec<T>` usually does not move the elements.
- Why stack allocation is cheap but stack space is limited.
- Why `drop` does not always reduce process RSS.
- Why sequential array access beats pointer chasing.
- Why page faults and TLB misses are different costs.
- Why arenas are fast and when they are the wrong abstraction.
- Why a user-space pointer cannot be handed to hardware as a physical address.
