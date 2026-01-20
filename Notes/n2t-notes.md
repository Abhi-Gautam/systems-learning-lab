# N2T - Nand2Tetris
## Notes

---

## [2026-01-09] Chapter 2 - Hardware Implementation: Half Adder → Full Adder → Adder → ALU

### 1. Half Adder (adds 2 bits)

```
         a ───┬───────┐
              │      XOR──── sum
         b ───┼───┬───┘
              │   │
              └───AND────── carry
                  │

TRUTH TABLE:
a  b │ sum carry
0  0 │  0    0
0  1 │  1    0
1  0 │  1    0
1  1 │  0    1
```

### 2. Full Adder (adds 3 bits - handles carry-in)

```
              ┌───────────┐
    a ───────>│           │
              │   HALF    ├───> sum1 ──┐
    b ───────>│   ADDER   │            │    ┌──────────┐
              │           ├─> carry1 ──┼───>│          │
              └───────────┘            │    │    OR    ├──> carry-out
                                       │    │          │
              ┌───────────┐            │    └────┬─────┘
  sum1 ──────>│           │            │         │
              │   HALF    ├──> sum ────┼─────────┘
carry-in ────>│   ADDER   │            │
              │           ├─> carry2 ──┘
              └───────────┘

a  b  cin │ sum  cout
0  0   0  │  0    0
0  0   1  │  1    0
0  1   0  │  1    0
0  1   1  │  0    1
1  0   0  │  1    0
1  0   1  │  0    1
1  1   0  │  0    1
1  1   1  │  1    1
```

### 3. 16-bit Adder (chain of full adders - Ripple Carry Adder)

```
   a[0] b[0]    a[1] b[1]    a[2] b[2]         a[15] b[15]
     │   │       │   │        │   │              │    │
     ▼   ▼       ▼   ▼        ▼   ▼              ▼    ▼
   ┌───────┐   ┌───────┐   ┌───────┐          ┌───────┐
0─>│ FULL  │──>│ FULL  │──>│ FULL  │── ··· ──>│ FULL  │──> overflow
   │ ADDER │   │ ADDER │   │ ADDER │          │ ADDER │    (ignored)
   └───┬───┘   └───┬───┘   └───┬───┘          └───┬───┘
       │           │           │                  │
       ▼           ▼           ▼                  ▼
    out[0]      out[1]      out[2]            out[15]

Carry ripples through each full adder in sequence.
```

**Problem:** Ripple Carry is SLOW - O(n) time complexity. Each bit must wait for the previous carry.

### 3b. Carry-Lookahead Adder (CLA) - How Real CPUs Do It

Instead of waiting for carries to ripple, **predict** all carries in parallel.

**Core Concept: Generate and Propagate**

```
For each bit position i:

GENERATE (Gi) = Ai AND Bi
  → This bit WILL produce a carry (regardless of carry-in)

PROPAGATE (Pi) = Ai XOR Bi
  → This bit WILL pass through a carry (if one comes in)
```

**The Carry Equations (computed in parallel):**

```
C0 = input carry (usually 0)
C1 = G0 + P0·C0
C2 = G1 + P1·G0 + P1·P0·C0
C3 = G2 + P2·G1 + P2·P1·G0 + P2·P1·P0·C0
C4 = G3 + P3·G2 + P3·P2·G1 + P3·P2·P1·G0 + P3·P2·P1·P0·C0
```

**Why This is Parallel:**

```
RIPPLE: Sequential           LOOKAHEAD: Parallel

C0 ──> C1 ──> C2 ──> C3      Step 1: All G,P computed simultaneously
       wait   wait   wait    Step 2: All carries computed simultaneously
                             Step 3: All sums computed simultaneously

Time: O(n) ~32 delays        Time: O(log n) ~4 delays
```

**4-bit CLA Hardware:**

```
A[3:0] ─────────┬─────────────────────────────────────┐
                │                                     │
B[3:0] ─────────┼──────────────────┐                  │
                │                  │                  │
                ▼                  ▼                  │
         ┌─────────────────────────────┐             │
         │   G/P GENERATOR             │             │
         │  G0 = A0 & B0  P0 = A0^B0   │             │
         │  G1 = A1 & B1  P1 = A1^B1   │             │
         │  G2 = A2 & B2  P2 = A2^B2   │             │
         │  G3 = A3 & B3  P3 = A3^B3   │             │
         └──────────┬──────────────────┘             │
                    │                                 │
                    ▼                                 │
         ┌─────────────────────────────┐             │
         │   CARRY LOOKAHEAD UNIT      │             │
    C0──>│  C1 = G0 + P0·C0            │             │
         │  C2 = G1 + P1·G0 + ...      │             │
         │  C3 = G2 + ...              │             │
         │  C4 = G3 + ...              │──> Cout     │
         └──────────┬──────────────────┘             │
                    │ C0,C1,C2,C3                    │
                    ▼                                ▼
         ┌─────────────────────────────────────────────┐
         │   SUM GENERATORS: Si = Pi ⊕ Ci             │
         └─────────────────────────────────────────────┘
                    │
                    ▼
               S[3:0] (Sum output)
```

**Worked Example: 7 + 3 = 10**

```
A = 0111 (7)
B = 0011 (3)

Step 1: Generate and Propagate (parallel)
┌─────┬────┬────┬────────┬────────┐
│ Bit │ Ai │ Bi │ Gi=A&B │ Pi=A^B │
├─────┼────┼────┼────────┼────────┤
│  0  │  1 │  1 │   1    │   0    │
│  1  │  1 │  1 │   1    │   0    │
│  2  │  1 │  0 │   0    │   1    │
│  3  │  0 │  0 │   0    │   0    │
└─────┴────┴────┴────────┴────────┘

Step 2: Compute carries (parallel, C0=0)
  C1 = G0 + P0·C0 = 1 + 0 = 1
  C2 = G1 + P1·G0 = 1 + 0 = 1
  C3 = G2 + P2·G1 = 0 + 1 = 1
  C4 = G3 + P3·G2 = 0 + 0 = 0

Step 3: Compute sums (parallel)
  S0 = P0 ⊕ C0 = 0 ⊕ 0 = 0
  S1 = P1 ⊕ C1 = 0 ⊕ 1 = 1
  S2 = P2 ⊕ C2 = 1 ⊕ 1 = 0
  S3 = P3 ⊕ C3 = 0 ⊕ 1 = 1

Result: 1010 = 10 ✓
```

**16-bit: Hierarchical CLA (two levels)**

```
        ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
A,B ───>│ 4-bit   │ │ 4-bit   │ │ 4-bit   │ │ 4-bit   │
[15:0]  │  CLA    │ │  CLA    │ │  CLA    │ │  CLA    │
        │ [15:12] │ │ [11:8]  │ │  [7:4]  │ │  [3:0]  │
        └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘
             │ PG       │ PG       │ PG       │ PG
             └───────────┴───────────┴───────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │  2nd Level CLA   │
                    │  (Group carries) │
                    └──────────────────┘
                              │
                              ▼
                     C4, C8, C12, C16
```

**Comparison:**

| Adder Type | Speed | Hardware | Used In |
|------------|-------|----------|---------|
| Ripple Carry | O(n) ~32 delays | Minimal | N2T, education |
| Carry Lookahead | O(log n) ~4 delays | More gates | Real CPUs |

**Why N2T uses Ripple Carry:** Simpler to understand and implement. At the Hack computer's clock speed, it's fast enough.

### 4. ALU (Arithmetic Logic Unit)

**6 Control Bits:**
| Bit | Name | If = 1 |
|-----|------|--------|
| zx | Zero X | Set x = 0 |
| nx | Negate X | Set x = !x (bitwise) |
| zy | Zero Y | Set y = 0 |
| ny | Negate Y | Set y = !y (bitwise) |
| f | Function | 1 = ADD, 0 = AND |
| no | Negate Out | Negate the output |

**ALU Block Diagram:**

```
    x[16]                                    y[16]
      │                                        │
      ▼                                        ▼
 ┌─────────┐                             ┌─────────┐
 │  MUX    │◄── zx (zero x)              │   MUX   │◄── zy (zero y)
 │ x or 0  │                             │ y or 0  │
 └────┬────┘                             └────┬────┘
      │                                       │
      ▼                                       ▼
 ┌─────────┐                             ┌─────────┐
 │   MUX   │◄── nx (negate x)            │   MUX   │◄── ny (negate y)
 │ x or !x │                             │ y or !y │
 └────┬────┘                             └────┬────┘
      │                                       │
      └──────────────┬────────────────────────┘
                     │
         ┌───────────┴───────────┐
         ▼                       ▼
    ┌─────────┐            ┌─────────┐
    │  ADDER  │            │   AND   │
    │  x + y  │            │  x & y  │
    └────┬────┘            └────┬────┘
         │                      │
         └──────────┬───────────┘
                    ▼
               ┌─────────┐
               │   MUX   │◄── f (select ADD or AND)
               └────┬────┘
                    │
                    ▼
               ┌─────────┐
               │   MUX   │◄── no (negate output)
               └────┬────┘
                    │
                    ▼
                out[16] ──────┬────────────────┐
                              │                │
                              ▼                ▼
                        ┌──────────┐    ┌───────────┐
                        │ out==0 ? │    │ out[15]   │
                        └────┬─────┘    └─────┬─────┘
                             │                │
                             ▼                ▼
                            zr               ng
                       (zero flag)      (negative flag)
```

**ALU Truth Table (18 useful operations):**

```
┌────┬────┬────┬────┬───┬────┬─────────────┐
│ zx │ nx │ zy │ ny │ f │ no │   Output    │
├────┼────┼────┼────┼───┼────┼─────────────┤
│ 1  │ 0  │ 1  │ 0  │ 1 │ 0  │      0      │
│ 1  │ 1  │ 1  │ 1  │ 1 │ 1  │      1      │
│ 1  │ 1  │ 1  │ 0  │ 1 │ 0  │     -1      │
│ 0  │ 0  │ 1  │ 1  │ 0 │ 0  │      x      │
│ 1  │ 1  │ 0  │ 0  │ 0 │ 0  │      y      │
│ 0  │ 0  │ 1  │ 1  │ 0 │ 1  │     !x      │
│ 1  │ 1  │ 0  │ 0  │ 0 │ 1  │     !y      │
│ 0  │ 0  │ 1  │ 1  │ 1 │ 1  │     -x      │
│ 1  │ 1  │ 0  │ 0  │ 1 │ 1  │     -y      │
│ 0  │ 1  │ 1  │ 1  │ 1 │ 1  │    x + 1    │
│ 1  │ 1  │ 0  │ 1  │ 1 │ 1  │    y + 1    │
│ 0  │ 0  │ 1  │ 1  │ 1 │ 0  │    x - 1    │
│ 1  │ 1  │ 0  │ 0  │ 1 │ 0  │    y - 1    │
│ 0  │ 0  │ 0  │ 0  │ 1 │ 0  │    x + y    │
│ 0  │ 1  │ 0  │ 0  │ 1 │ 1  │    x - y    │
│ 0  │ 0  │ 0  │ 1  │ 1 │ 1  │    y - x    │
│ 0  │ 0  │ 0  │ 0  │ 0 │ 0  │    x & y    │
│ 0  │ 1  │ 0  │ 1  │ 0 │ 1  │    x | y    │
└────┴────┴────┴────┴───┴────┴─────────────┘
```

**Key Insight:** The x | y operation uses De Morgan's Law: `!(!x & !y) = x | y`

### Complete Hierarchy

```
NAND gates
    │
    ▼
NOT, AND, OR, XOR, MUX
    │
    ▼
Half Adder (XOR + AND)
    │
    ▼
Full Adder (2 Half Adders + OR)
    │
    ▼
16-bit Adder (16 Full Adders chained)
    │
    ▼
ALU (Adder + AND + MUXes + NOT)
    │
    ▼
CPU (ALU + Registers + Control)
```

---

## [2026-01-10] Chapter 3 - Data Flip-Flop (DFF) and Sequential Logic

### The Fundamental Shift: Combinational → Sequential

```
COMBINATIONAL (Chapters 1-2)       SEQUENTIAL (Chapter 3+)

  Output = f(current inputs)       Output = f(inputs, history)
  No memory                        Has memory
  No concept of time               Clock introduces time

  Examples: AND, OR, ALU           Examples: DFF, Register, RAM
```

### What a DFF Does

```
        ┌───────────┐
  in ──>│           │
        │    DFF    ├──> out
 clk ──>│           │
        └───────────┘

Rule: out(t) = in(t-1)

"The output NOW equals the input from ONE clock cycle ago"
```

### Timing Diagram

```
        │   │   │   │   │   │   │   │
 clk ───┘   └───┘   └───┘   └───┘   └───  (clock ticks)
        t0  t1  t2  t3  t4  t5  t6  t7

 in  ─────┐       ┌───────────┐   ┌───
          └───────┘           └───┘
          1   0   1   1   1   0   1

out  ─────────┐       ┌───────────┐   ┌─
              └───────┘           └───┘
              1   0   1   1   1   0   1

              ↑
              Delayed by 1 clock cycle!
```

### Edge-Triggered Behavior

```
Clock signal:
      ┌───┐   ┌───┐   ┌───┐
      │   │   │   │   │   │
  ────┘   └───┘   └───┘   └───
      ↑       ↑       ↑
   Rising  Rising  Rising
    Edge    Edge    Edge

DFF captures input ONLY at the rising edge.
Between edges, input can change freely - doesn't matter.
```

### Inside a DFF (Conceptual - N2T treats as primitive)

```
MASTER-SLAVE D FLIP-FLOP

         MASTER              SLAVE
        (Latch)             (Latch)
      ┌─────────┐         ┌─────────┐
D ───>│         │────────>│         │───> Q
      │  Latch  │         │  Latch  │
clk ─>│         │    !clk>│         │
      └─────────┘         └─────────┘

      Opens when           Opens when
      clk = 0              clk = 1

Result: Input captured precisely at rising edge
```

### 1-Bit Register (DFF with Load control)

```
                    ┌───────────────┐
                    │               │
          ┌─────┐   │   ┌─────┐     │
   in ───>│     │   │   │     │     │
          │ MUX ├───┴──>│ DFF ├─────┼───> out
  out ───>│     │       │     │     │
    │     └──┬──┘       └─────┘     │
    │        │                      │
    └────────┼──────────────────────┘
             │
  load ──────┘

If load=1: out(t) = in(t-1)    ← store new value
If load=0: out(t) = out(t-1)   ← keep current value (feedback loop)
```

### Building Memory: The Hierarchy

```
DFF (1 bit, no load control)
    │
    ▼
Bit (1-bit register with load)
    │
    ▼
Register (16 Bits in parallel)
    │
    │   in[0] ──>│Bit│──> out[0]
    │   in[1] ──>│Bit│──> out[1]
    │   in[2] ──>│Bit│──> out[2]
    │     ...     ...     ...
    │   in[15]──>│Bit│──> out[15]
    │
    ▼
RAM8 (8 Registers + 3-bit address decoder)
    │
    │   ┌──────────┐
    │   │ Register │ ← address 000
    │   ├──────────┤
    │   │ Register │ ← address 001
    │   ├──────────┤
    │   │   ...    │
    │   ├──────────┤
    │   │ Register │ ← address 111
    │   └──────────┘
    │
    ▼
RAM64 (8 × RAM8)
    │
    ▼
RAM512 (8 × RAM64)
    │
    ▼
RAM4K (8 × RAM512)
    │
    ▼
RAM16K (4 × RAM4K) ← Hack computer's data memory
```

### RAM Addressing

```
RAM8 example (8 registers, 3-bit address):

       address[3]
           │
           ▼
      ┌─────────┐
      │  DMUX   │ (1-to-8 demultiplexer)
      │  8-way  │
      └────┬────┘
           │ load signals (only one is 1)
   ┌───┬───┼───┬───┬───┬───┬───┐
   ▼   ▼   ▼   ▼   ▼   ▼   ▼   ▼
 ┌───┬───┬───┬───┬───┬───┬───┬───┐
 │R0 │R1 │R2 │R3 │R4 │R5 │R6 │R7 │  (8 registers)
 └─┬─┴─┬─┴─┬─┴─┬─┴─┬─┴─┬─┴─┬─┴─┬─┘
   │   │   │   │   │   │   │   │
   └───┴───┴───┴───┴───┴───┴───┘
                 │
                 ▼
            ┌─────────┐
            │   MUX   │ (8-way-16bit)
            │  8-way  │
            └────┬────┘
                 │
                 ▼
              out[16]
```

### Key Insight: Time as a Dimension

```
COMBINATIONAL LOGIC              SEQUENTIAL LOGIC

  Space only                     Space + Time
  (gates, wires)                 (gates + clock)

  f(input) = output              f(input, history) = output

  Can't compute loops            CAN compute loops
  Can't store state              CAN store state
  No feedback allowed            Feedback through DFF is safe
```

### Why DFF is Special

DFF introduces **time** into the computer, enabling:

| Component | Uses DFF For |
|-----------|--------------|
| Registers | Store ALU results, addresses |
| RAM | Store program data |
| Program Counter | Track current instruction |
| CPU State | Remember what phase of instruction we're in |

Without DFF, computers would be pure calculators with no memory!

---

## [2026-01-10] Page 43 - Building a 16-bit Register from Primitives

### The Complete Build Path

```
NAND (primitive)
    │
    ▼
NOT, AND, OR, MUX (Chapter 1)
    │
    ▼
DFF (primitive - given in Chapter 3)
    │
    ▼
Bit (1-bit register) = MUX + DFF
    │
    ▼
Register (16-bit) = 16 Bits in parallel
```

### Step 1: The Bit (1-bit Register)

The Bit is the first thing YOU build in Chapter 3. It combines:
- **MUX** (from Chapter 1) - to select between old value and new input
- **DFF** (given primitive) - to store the selected value

```
IMPLEMENTATION:

                         ┌─────────────────────┐
                         │                     │
           ┌─────┐       │    ┌─────┐          │
    in ───>│ a   │       │    │     │          │
           │     ├───────┴───>│ DFF ├──────────┼───> out
   out ───>│ MUX │            │     │          │
     │     │ b   │            └─────┘          │
     │     └──┬──┘                             │
     │        │                                │
     └────────┼────────────────────────────────┘
              │
   load ──────┘

HDL (what you write):
    Mux(a=feedback, b=in, sel=load, out=muxout);
    DFF(in=muxout, out=out, out=feedback);
```

**How it works:**

| load | Action | What MUX outputs |
|------|--------|------------------|
| 0 | HOLD | Previous out (feedback loop) |
| 1 | LOAD | New input value |

### Step 2: The 16-bit Register

Simply 16 Bits working in parallel, all sharing the same `load` signal:

```
IMPLEMENTATION:

        in[0] ──>┌─────┐──> out[0]
                 │ Bit │
        load ───>└─────┘

        in[1] ──>┌─────┐──> out[1]
                 │ Bit │
        load ───>└─────┘

        in[2] ──>┌─────┐──> out[2]
                 │ Bit │
        load ───>└─────┘
           .        .        .
           .        .        .
           .        .        .
        in[15]──>┌─────┐──> out[15]
                 │ Bit │
        load ───>└─────┘


HDL (what you write):
    Bit(in=in[0],  load=load, out=out[0]);
    Bit(in=in[1],  load=load, out=out[1]);
    Bit(in=in[2],  load=load, out=out[2]);
    ...
    Bit(in=in[15], load=load, out=out[15]);
```

### Component Count

```
1 Register (16-bit)
    │
    ├── 16 × Bit
    │       │
    │       ├── 1 × MUX (built from ~4 NANDs)
    │       └── 1 × DFF (primitive)
    │
    └── Total: 16 MUXes + 16 DFFs
              ≈ 64 NANDs + 16 DFFs
```

### Key Insight: Parallel vs Sequential

```
REGISTER: All bits load in PARALLEL

   Clock tick!
      │
      ▼
   ┌──┴──┬──┴──┬──┴──┬─────┬──┴──┐
   │Bit0 │Bit1 │Bit2 │ ... │Bit15│  ← All update simultaneously
   └─────┴─────┴─────┴─────┴─────┘

All 16 bits see the SAME clock edge.
All 16 bits load (or hold) at the SAME instant.
This is why we can treat a register as storing a single 16-bit value.
```

### Behavioral Summary

```
Register Behavior:

if load(t-1) == 1:
    out(t) = in(t-1)     // Store new value
else:
    out(t) = out(t-1)    // Keep old value

The one-cycle delay comes from the DFFs inside each Bit.
```

---

## [2026-01-10] Page 43 - Purpose of Load Bit in RAM

A classical RAM device accepts three inputs: **data**, **address**, and **load**.

### Load Determines Read vs Write

```
load = 0  →  READ   (output what's at address, don't change anything)
load = 1  →  WRITE  (store data input into the addressed location)
```

### Why Load is Necessary

Without load, RAM would overwrite data every clock cycle:

```
WITHOUT LOAD (broken):                 WITH LOAD (correct):

Every clock tick:                      load=0: Read mode
  address → location                     address → location → out
  data → overwrites location             (data input ignored)

  Can never just "look" at data!       load=1: Write mode
                                         address → location
                                         data → stored in location
```

### Filing Cabinet Analogy

```
address = which drawer to open
data    = the document you might want to file
load    = are you FILING (1) or just LOOKING (0)?

You always need to specify the drawer (address).
You always get to see what's in it (out).
But you only replace the contents if load=1.
```

### Inside RAM: Load Routing via DMUX

The load bit propagates down to individual registers:

```
        load ────────────────────────┐
                                     │
        address ──> DMUX ────────────┼─────────┐
                      │              │         │
                 ┌────┴────┐    ┌────┴────┐    │
                 │ load=0  │    │ load=1  │    │  (only ONE register
                 ▼         ▼    ▼         ▼    │   gets load=1)
              ┌─────┐   ┌─────┐   ┌─────┐     │
              │Reg 0│   │Reg 1│   │Reg 2│ ... │
              └─────┘   └─────┘   └─────┘     │
```

The DMUX uses the address to route the load signal to exactly one register. All other registers keep `load=0` and hold their values.

---

## [2026-01-10] Page 43 - Clock Cycle Length and Signal Propagation

### The Timing Guarantee

The clock cycle must be **slightly longer** than the time it takes a bit to travel the longest distance from one chip to another in the architecture.

```
WHY THIS MATTERS:

Clock Cycle N                          Clock Cycle N+1
     │                                       │
     ▼                                       ▼
┌─────────────────────────────────────┐     ┌───
│  ALU computes    →  signal travels  │     │ Register captures
│  result             through wires   │     │ the valid input
└─────────────────────────────────────┘     └───
     │                                       │
     │◄──── must complete before ──────────►│
                                          rising edge
```

### The Problem We're Solving

```
TOO SHORT clock cycle:              CORRECT clock cycle:

  clk ─┐   ┌─┐   ┌─                  clk ─┐       ┌─┐       ┌─
       └───┘ └───┘                        └───────┘ └───────┘

  Signal still traveling             Signal has arrived
  when clock ticks!                  and stabilized
       │                                   │
       ▼                                   ▼
  Register captures                  Register captures
  GARBAGE (old/partial)              VALID result
```

### Propagation Delay Chain

```
Longest path example (ALU → Register):

  Input ──► ALU ──► wires ──► MUX ──► Register
  Registers
       │      │       │        │         │
       t0    +20ns   +5ns    +10ns      capture

  Total propagation: ~35ns
  Clock cycle must be: >35ns (e.g., 50ns = 20MHz)
```

### Key Insight

This is why faster chips need:
1. **Shorter wires** (signals travel faster)
2. **Faster gates** (less propagation delay)
3. **Pipelining** (break long paths into stages)

The clock doesn't "push" data through the circuit—it just guarantees that by the time it ticks, all combinational logic has settled to valid values.

---

## [2026-01-20] Chapter 3 - Future Exercise: Build Sequential Chips in Rust

### Goal
Implement all Chapter 3 chips (DFF → Bit → Register → RAM → PC) in Rust to solidify understanding of sequential logic and clock behavior.

### Chips to Build

| Chip | What it does | Built from |
|------|--------------|------------|
| DFF | 1-bit memory primitive | (simulated with state + tick) |
| Bit | 1-bit register with load | DFF + Mux |
| Register | 16-bit register | 16 Bits |
| RAM8 | 8 registers, addressable | 8 Registers + Mux8Way16 + DMux8Way |
| RAM64 | 64 registers | 8 RAM8s |
| RAM512 | 512 registers | 8 RAM64s |
| RAM4K | 4096 registers | 8 RAM512s |
| RAM16K | 16384 registers | 4 RAM4Ks |
| PC | Program Counter | Register + Inc16 + Mux16s |

### Key Rust Pattern: Simulating Clock

```rust
struct DFF {
    state: bool,      // current output (what you read)
    next_state: bool, // staged input (waiting for tick)
}

impl DFF {
    fn set(&mut self, input: bool) {
        self.next_state = input;  // stage the input
    }

    fn get(&self) -> bool {
        self.state  // return current state
    }

    fn tick(&mut self) {
        self.state = self.next_state;  // commit on clock edge
    }
}
```

### The Pattern for All Sequential Chips

```
1. set() - configure inputs (combinational logic settles)
2. tick() - clock edge commits state changes
3. get() - read outputs (reflects new state)
```

### Why This Exercise Matters

- Forces understanding of t vs t-1 timing
- Makes clock edge behavior tangible
- Builds intuition for how RAM addressing works
- Prepares for CPU implementation in Chapter 5

### Status: PENDING

Complete after finishing Chapter 3 reading.

---
