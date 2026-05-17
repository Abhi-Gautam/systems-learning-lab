# N2T Notes

_Entries follow the template at `Notes/TEMPLATE.md`. Append-only. **Newest entry at top**, immediately after this header._

---

## [2026-05-21] Hack Machine Language — Registers, Memory Model, A/C-Instructions, and the 16-Bit Instruction Encoding · pp.71–94 · Ch.4 Machine Language (full chapter: Background → Hack Language Specification → Symbolic Notation → Input/Output → Perspective)

### TL;DR
Hack's machine language is a deliberately minimal **two-register, von Neumann-ish** instruction set: every program is a stream of 16-bit words, each word being either an **A-instruction** (load a 15-bit constant into register A) or a **C-instruction** (compute on D/A/M, store to one of D/A/M, optionally jump). Memory addressing is entirely **indirect via the A register** — there's no explicit base+offset or addressing mode; you put the address in A, then refer to memory as `M`. Once you internalize the **C-instruction bit layout** (1 ttt a cccccc ddd jjj), the entire ISA fits on one page and every higher-level abstraction you've ever met can be compiled down to it.

### Intuition — "this is like…"
Hack is **assembly language stripped to its bones**: no general-purpose register file (just D, A, M), no condition flags, no addressing modes, no subroutine call instruction (you simulate it with jumps and a stack later). It's what you'd design if you had to fit a working CPU into the smallest possible chip and wanted students to be able to draw the entire datapath from memory by Friday. Every "feature" of real ISAs (x86, ARM, RISC-V) is something Hack deliberately omits to expose how little is actually required.

### Mechanics

#### 1. The three abstractions every machine language manipulates

| Abstraction | Hack incarnation | Purpose |
|---|---|---|
| **Memory** | 32K data RAM + 32K instruction ROM (separate, Harvard) | Holds data and program |
| **Processor** | one ALU computing on D, A, M | Does arithmetic / logical ops |
| **Registers** | D (data), A (address/data), M (= `RAM[A]`, virtual) | Hold operands; A doubles as memory pointer |

> **Note**: `M` is *not* a physical register. It's syntactic sugar for "the memory word at the address currently in A." Mentioning M in an instruction causes the CPU to drive RAM[A] onto the ALU input/output. This is the chapter's key sleight-of-hand: 2 registers act like 3.

#### 2. Hack memory map

```
   Address space (16-bit addresses, but only 15 used in A-instructions)
   ┌──────────────────────────────────────────────────────────────┐
   │ 0x0000 – 0x3FFF │ Data Memory (RAM)                          │  16K words
   │   0x0000–0x000F │   16 virtual registers R0–R15              │
   │   0x0010–...    │   static, stack, heap (set up by software) │
   │ 0x4000 – 0x5FFF │ Screen memory-mapped I/O (8K words = 256×512 1-bit pixels) │
   │ 0x6000          │ Keyboard memory-mapped I/O (1 word)        │
   │ 0x6001 – 0x7FFF │ unused                                     │
   └──────────────────────────────────────────────────────────────┘

   Separate instruction ROM (Harvard arch):
   ┌──────────────────────────────────────────────────────────────┐
   │ 0x0000 – 0x7FFF │ Instruction Memory (ROM)                   │  32K words
   └──────────────────────────────────────────────────────────────┘
```

**Memory-mapped I/O** is the whole I/O story — no MMIO registers, no port instructions, no DMA. To draw a pixel you store a bit at the right offset in 0x4000–0x5FFF. To read a key you load from 0x6000. *This is exactly how real systems do framebuffers* — Hack just doesn't hide it behind a driver.

#### 3. The two instruction types

| Type | First bit | Meaning | Example (symbolic) |
|---|---|---|---|
| **A-instruction** | `0` | Load 15-bit constant into A | `@42` → A = 42 |
| **C-instruction** | `1` | Compute, store, jump | `D=M+1; JGT` |

**A-instruction layout** (16 bits):
```
   bit:  15 14 13 12 11 10  9  8  7  6  5  4  3  2  1  0
        ┌──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┐
        │ 0│ v│ v│ v│ v│ v│ v│ v│ v│ v│ v│ v│ v│ v│ v│ v│
        └──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┘
          ↑  ←──────────── 15-bit constant value ─────────→
       opcode = 0
```
The A-instruction is "load this number into A" — used both for immediate values AND to set up a memory address before a subsequent `M` reference. **Same instruction, two semantic uses** — the compiler decides which.

**C-instruction layout** (16 bits, the heart of the ISA):
```
   bit:  15 14 13 │ 12 │ 11 10  9  8  7  6 │  5  4  3 │  2  1  0
        ┌──┬──┬──┼────┼──┬──┬──┬──┬──┬──┼──┬──┬──┼──┬──┬──┐
        │ 1│ 1│ 1│  a │ c1 c2 c3 c4 c5 c6│ d1 d2 d3│ j1 j2 j3│
        └──┴──┴──┴────┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┘
         ↑opcode ↑    ↑─── 6-bit comp ─→  ↑─dest─→  ↑─jump─→
                 │
                 a = 0 → ALU sees A
                 a = 1 → ALU sees M (= RAM[A])
```

| Field | Bits | Controls |
|---|---|---|
| Opcode | 15 | Must be 1 for C-instruction |
| Unused | 14, 13 | Set to 1 by convention |
| `a` | 12 | Selects A vs M as second ALU input |
| `comp` | 11–6 | Picks one of 28 ALU functions |
| `dest` | 5–3 | Bit-mask: write result to A, D, or M (any subset) |
| `jump` | 2–0 | Encodes conditional jump based on ALU output sign |

#### 4. The 28 ALU computations (the `comp` field)

| `comp` symbolic | `a c1..c6` | What ALU outputs |
|---|---|---|
| `0` | 0 101010 | constant 0 |
| `1` | 0 111111 | constant 1 |
| `-1` | 0 111010 | constant −1 |
| `D` | 0 001100 | D |
| `A` / `M` | 0/1 110000 | A or M |
| `!D` | 0 001101 | NOT D |
| `-D` | 0 001111 | −D |
| `D+1` | 0 011111 | D + 1 |
| `D+A` / `D+M` | 0/1 000010 | D + A or D + M |
| `D-A` / `D-M` | 0/1 010011 | D − A or D − M |
| `D&A` / `D&M` | 0/1 000000 | D AND A or D AND M |
| `D\|A` / `D\|M` | 0/1 010101 | D OR A or D OR M |

> **Notice what's missing**: no multiply, no divide, no shift, no floating point. Multiply is a software loop on top of `+`. Floating point doesn't exist; if you need it, build it in software.

#### 5. The `dest` and `jump` fields

```
   dest (bits 5,4,3 = d1,d2,d3):              jump (bits 2,1,0 = j1,j2,j3):
   ┌─────┬───────────────────┐                ┌─────┬─────────────────────┐
   │ 000 │ no destination    │                │ 000 │ no jump (next instr)│
   │ 001 │ M                 │                │ 001 │ JGT (out > 0)       │
   │ 010 │ D                 │                │ 010 │ JEQ (out == 0)      │
   │ 011 │ MD (both)         │                │ 011 │ JGE (out >= 0)      │
   │ 100 │ A                 │                │ 100 │ JLT (out < 0)       │
   │ 101 │ AM                │                │ 101 │ JNE (out != 0)      │
   │ 110 │ AD                │                │ 110 │ JLE (out <= 0)      │
   │ 111 │ AMD (all three)   │                │ 111 │ JMP (unconditional) │
   └─────┴───────────────────┘                └─────┴─────────────────────┘
```

`dest` is a **bitmask**, not an enum — you can store the same ALU result into any subset of {A, M, D} in a single cycle. So `AMD=D+1` is one instruction that increments D and writes the result to A, M, and D simultaneously. *Unusually parallel for a tiny ISA* — reflects independent destination latches.

#### 6. Worked example — incrementing RAM[100]

```
   @100      // A-instruction: A = 100
   M=M+1     // C-instruction: RAM[100] = RAM[100] + 1
```

Binary:
```
   @100      → 0 000000001100100         (A = 100)
                ↑↑─────15-bit value────↑
                opcode=0

   M=M+1     → 1 11 1 110111 001 000     (M = M+1)
                ↑↑↑─↑↑──────↑↑──↑↑──↑
                | | |  |    |   |
                | | |  comp |   jump (000 = no jump)
                | | |  M+1  dest = 001 = M
                | | a=1 (use M, not A)
                | unused (1)
                opcode=1
```

Two instructions, ~22 bits of program. To increment a heap variable in C requires `lea/mov/add/mov` on x86 — far more bits, far more state, but exactly the same logical operation.

#### 7. Symbolic vs binary — what the assembler does

| Symbolic shortcut | What assembler emits |
|---|---|
| `@LOOP` (label) | `@<ROM address of LOOP>` (computed in pass 1) |
| `@i` (variable) | `@<RAM address starting from 16>` (auto-allocated) |
| `@SCREEN` | `@16384` (predefined constant for 0x4000) |
| `@KBD` | `@24576` (0x6000) |
| `@R0` ... `@R15` | `@0` ... `@15` |
| `@SP, @LCL, @ARG, @THIS, @THAT` | `@0, @1, @2, @3, @4` (VM-layer pointers) |

Two-pass assembler:
```
   Pass 1: scan all (LABEL) declarations, record ROM addresses
   Pass 2: emit binary, resolving @symbol references:
            - if symbol is a label: use its ROM address
            - if symbol is a predefined constant: use that
            - else: auto-allocate next free RAM address starting at 16
```

### If you were the CPU executing one Hack instruction
You **fetch** the 16-bit instruction word from instruction ROM at `PC`. Inspect bit 15:
- If `0`: A-instruction; load bits 14..0 into A; PC++. Done.
- If `1`: C-instruction. Drive A onto address bus; if a=1, route RAM[A] (M) as ALU input y, else A; route D as ALU input x. The 6-bit `comp` field directly configures the ALU's six control inputs (zx, nx, zy, ny, f, no — see Appendix A). ALU outputs result + two status bits (`zr`=zero, `ng`=negative). The `dest` bits select which destination latches load. Finally, `jump` bits combined with `zr` and `ng` decide PC := A (jump) or PC := PC+1 (fallthrough). One cycle, no pipelining, no microcode.

### Where this shows up in real systems
- **RISC-V `ADDI x5, x6, 100`** — same idea (immediate + ALU + dest) but with a 32-register file and 32-bit instructions. Hack's "A holds memory addresses" is the degenerate case of RISC's base+offset addressing.
- **x86 `LEA` instruction** — computes an address into a register without a memory access, similar to Hack's A-instruction.
- **"A and M are address and value at that address"** is x86's `[reg]` syntax, made painfully explicit because Hack lacks a base+offset mode.
- **Memory-mapped framebuffers** in every OS — `/dev/fb0` on Linux, the Mac's IOFramebuffer service. Same model Hack uses, scaled up.
- **Compiler register allocation**: the `dest` bitmask (write to A, D, M simultaneously) is rare in real ISAs — Hack pays in instruction count to make hardware simpler.

### Diagnostic questions
1. *"Why does Hack need separate A-instructions and C-instructions instead of one unified format?"* — Because addressing 32K of memory requires 15 bits of address space, and an instruction encoding {comp, dest, jump, source-select} can't simultaneously hold a 15-bit immediate. Splitting them lets each format use all 15 bits for its specialized purpose. Real ISAs do the same — RISC-V splits I/U/J formats for the same reason. (Wrong: "two types is simpler" — no, it's a *forced* split by bit budget.)
2. *"What does the bit `a` in a C-instruction control?"* — Whether the second ALU input is the A register itself or the memory word at address A (M). It's a one-bit multiplexer select. Same `comp` bit pattern with a=0 vs a=1 gives you `D+A` vs `D+M`. (Wrong: "it's the opcode" — no, bit 15 is the opcode.)
3. *"Why can `AMD=D+1` write to three places in one cycle?"* — Because A, M, and D each have their own write-enable latch driven by an independent bit in `dest`. The ALU produces one result, fanned out to three sinks. Hardware cost: three muxes; instruction cost: zero extra cycles. (Wrong: "the ALU outputs three values" — no, one ALU output, three destinations.)
4. *"What instruction would zero out RAM[100]?"* — `@100` then `M=0`. The `0` is one of the 28 hardcoded ALU outputs; the assembler doesn't synthesize it from subtraction. (Wrong: `M=A-A` — works but wasteful.)
5. *"Why does jumping always set PC to A and never to a label directly?"* — Because the jump field is only 3 bits — it encodes the *condition*, not a target address. The target must already be in A, set up by a preceding `@LABEL` A-instruction. Real ISAs bundle target into the jump itself but pay in instruction width. (Wrong: "saves space" — actually costs an extra instruction; the trade is hardware simplicity.)

### See also
- N2T Ch.5 (next chapter) — builds the CPU that *executes* these instructions. You'll see how `dest` becomes three load-enables, how `comp` becomes the six ALU control bits, how `jump` becomes the PC select.
- COD Ch.2 (Instructions: Language of the Computer) — ARM/RISC-V version of the same story; addressing modes, register files, and immediates make a "real" ISA bigger but more efficient.
- DBI 2026-05-21 (B-Tree Mechanics) — the same idea of bit-budget compression scaled up: a B-Tree node uses every byte of a disk page to maximize fanout, just as a C-instruction uses every bit of a 16-bit word.

