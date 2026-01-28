# COD - Computer Organization and Design (ARM Edition)
## Notes

---

## [2026-01-21] Figure 1.15 - The CPU Performance Equation (The Big Picture)

### The Fundamental Formula

Execution time is the **only complete and reliable measure** of computer performance. It breaks down into three factors:

```
                    Instructions     Clock cycles      Seconds
Time (seconds) =   ───────────── × ───────────── × ─────────────
                     Program        Instruction     Clock cycle
```

Or written more compactly:

```
Execution Time = Instruction Count × CPI × Clock Cycle Time
```

Where:
- **Instruction Count** = Total instructions executed by the program
- **CPI** = Clock Cycles Per Instruction (average)
- **Clock Cycle Time** = 1 / Clock Frequency

### What Each Level Measures

```
┌─────────────────────────────────────────────────────────────────────┐
│                         PERFORMANCE FACTORS                         │
├─────────────────┬───────────────────┬───────────────────────────────┤
│     FACTOR      │   WHAT IT MEASURES │      AFFECTED BY              │
├─────────────────┼───────────────────┼───────────────────────────────┤
│ Instructions/   │ Dynamic instruction│ Algorithm, compiler,          │
│ Program         │ count              │ instruction set (ISA)         │
├─────────────────┼───────────────────┼───────────────────────────────┤
│ Clock cycles/   │ Average cycles per │ CPU organization,             │
│ Instruction     │ instruction (CPI)  │ instruction set, compiler     │
├─────────────────┼───────────────────┼───────────────────────────────┤
│ Seconds/        │ Clock period       │ Hardware technology,          │
│ Clock cycle     │ (1/frequency)      │ CPU organization              │
└─────────────────┴───────────────────┴───────────────────────────────┘
```

### The Multiplication Chain

```
Program
   │
   ▼
┌──────────────────┐
│   Instructions   │  ← How many instructions does the program execute?
│   executed       │     (Depends on algorithm + compiler + ISA)
└────────┬─────────┘
         │  × CPI
         ▼
┌──────────────────┐
│   Clock cycles   │  ← How many cycles do those instructions take?
│   total          │     (Depends on CPU design + instruction mix)
└────────┬─────────┘
         │  × Clock period
         ▼
┌──────────────────┐
│   Execution      │  ← The final answer: wall-clock time
│   Time (seconds) │
└──────────────────┘
```

### The Critical Warning: Trade-offs Are Everywhere

**Reducing one factor often increases another!**

```
EXAMPLE 1: Simplify the ISA (fewer instruction types)
┌────────────────────────────────────────────────────────┐
│  Goal: Lower CPI by having simpler instructions        │
│                                                        │
│  Trade-off: Need MORE instructions to do same work     │
│             Instruction count ↑  may offset CPI ↓      │
└────────────────────────────────────────────────────────┘

EXAMPLE 2: Complex instructions (CISC approach)
┌────────────────────────────────────────────────────────┐
│  Goal: Lower instruction count with powerful instrs    │
│                                                        │
│  Trade-off: Complex instructions take MORE cycles      │
│             CPI ↑  may offset instruction count ↓      │
└────────────────────────────────────────────────────────┘

EXAMPLE 3: Faster clock (higher frequency)
┌────────────────────────────────────────────────────────┐
│  Goal: Shorter clock cycle time                        │
│                                                        │
│  Trade-off: May need simpler pipeline stages           │
│             CPI ↑  may offset clock speed ↑            │
└────────────────────────────────────────────────────────┘
```

### Key Insight: Fewest Instructions ≠ Fastest Program

```
Program A: 1,000 instructions × 4.0 CPI × 1ns = 4,000 ns
Program B: 1,500 instructions × 2.0 CPI × 1ns = 3,000 ns  ← FASTER!

Program B executes 50% MORE instructions but runs 25% FASTER
because its CPI is much lower (simpler instructions).
```

### Worked Example

Compare two implementations running the same program:

```
Computer A:
  - Clock frequency: 2 GHz (cycle time = 0.5 ns)
  - CPI: 1.5
  - Instruction count: 10 billion

  Time = 10×10⁹ × 1.5 × 0.5×10⁻⁹ = 7.5 seconds

Computer B:
  - Clock frequency: 3 GHz (cycle time = 0.33 ns)
  - CPI: 2.0
  - Instruction count: 10 billion

  Time = 10×10⁹ × 2.0 × 0.33×10⁻⁹ = 6.6 seconds

Computer B is faster despite higher CPI because clock is 50% faster!
```

### Summary: The Only Truth is Time

```
┌─────────────────────────────────────────────────────────────┐
│  "The only complete and reliable measure of computer        │
│   performance is time."                                     │
│                                                             │
│  Don't be fooled by:                                        │
│    ✗ "More MHz/GHz" (CPI might be worse)                    │
│    ✗ "Fewer instructions" (each might take more cycles)     │
│    ✗ "Lower CPI" (might need more instructions)             │
│                                                             │
│  The product of ALL THREE factors determines performance.   │
└─────────────────────────────────────────────────────────────┘
```

**See also:** This connects to pipelining (Chapter 4) where we'll see how CPU organization tries to get CPI close to 1.

---

## [2026-01-21] What Affects CPU Performance? (Algorithm → Language → Compiler → ISA)

Performance depends on choices at **every level** of the stack. Each level affects different factors in the performance equation.

### The Full Picture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    WHAT AFFECTS PERFORMANCE?                                │
├──────────────────┬─────────────────────┬────────────────────────────────────┤
│    COMPONENT     │      AFFECTS        │              HOW?                  │
├──────────────────┼─────────────────────┼────────────────────────────────────┤
│ Algorithm        │ Instruction Count   │ Determines how much work to do     │
│                  │ CPI (possibly)      │ May favor slow/fast instructions   │
├──────────────────┼─────────────────────┼────────────────────────────────────┤
│ Programming      │ Instruction Count   │ Statements → processor instrs      │
│ Language         │ CPI                 │ Language features → instr types    │
├──────────────────┼─────────────────────┼────────────────────────────────────┤
│ Compiler         │ Instruction Count   │ Translation quality                │
│                  │ CPI                 │ Instruction selection/scheduling   │
├──────────────────┼─────────────────────┼────────────────────────────────────┤
│ ISA              │ Instruction Count   │ What operations are available      │
│                  │ CPI                 │ How complex each instruction is    │
│                  │ Clock Rate          │ How simple/complex to implement    │
└──────────────────┴─────────────────────┴────────────────────────────────────┘
```

---

### 1. ALGORITHM → Instruction Count, CPI

The algorithm determines **how much work** the program does.

```
EXAMPLE: Finding if a number is in a sorted list of 1 million elements

LINEAR SEARCH (bad algorithm):
┌─────────────────────────────────────────────────┐
│  for i in range(n):                             │
│      if list[i] == target: return True          │
│                                                 │
│  Worst case: 1,000,000 comparisons              │
│  Instructions: ~5,000,000 (load, compare, jump) │
└─────────────────────────────────────────────────┘

BINARY SEARCH (good algorithm):
┌─────────────────────────────────────────────────┐
│  while low <= high:                             │
│      mid = (low + high) / 2                     │
│      if list[mid] == target: return True        │
│      ...                                        │
│                                                 │
│  Worst case: 20 comparisons (log₂ 1,000,000)    │
│  Instructions: ~100                             │
└─────────────────────────────────────────────────┘

Same task, 50,000x fewer instructions!
```

**Algorithm can also affect CPI:**

```
Algorithm A: Mostly additions        → CPI ≈ 1
Algorithm B: Mostly divisions        → CPI ≈ 20-40
Algorithm C: Mostly memory accesses  → CPI ≈ 100+ (cache misses!)

Even with same instruction count, Algorithm A wins.
```

---

### 2. PROGRAMMING LANGUAGE → Instruction Count, CPI

Different languages compile to different amounts of machine code.

```
EXAMPLE: Creating an object and calling a method

C (direct, minimal abstraction):
┌─────────────────────────────────────────────────┐
│  struct Point { int x, y; };                    │
│  p.x = 5;  // Direct memory write               │
│                                                 │
│  Assembly: 1-2 instructions                     │
│  MOV [p+0], 5                                   │
└─────────────────────────────────────────────────┘

Java (heavy abstraction):
┌─────────────────────────────────────────────────┐
│  Point p = new Point();  // Heap allocation     │
│  p.setX(5);              // Virtual method call │
│                                                 │
│  Assembly: 20+ instructions                     │
│  - Allocate heap memory                         │
│  - Initialize object header                     │
│  - Look up vtable for setX                      │
│  - Indirect call through vtable                 │
│  - Null pointer check                           │
│  - Finally: store the value                     │
└─────────────────────────────────────────────────┘
```

**Language features that increase CPI:**

```
┌────────────────────────┬────────────────────────────────────┐
│ Language Feature       │ Why it hurts CPI                   │
├────────────────────────┼────────────────────────────────────┤
│ Virtual methods (Java) │ Indirect calls → pipeline stalls   │
│ Garbage collection     │ Unpredictable pauses               │
│ Bounds checking        │ Extra compare+branch per access    │
│ Dynamic typing (Python)│ Runtime type lookups               │
└────────────────────────┴────────────────────────────────────┘
```

---

### 3. COMPILER → Instruction Count, CPI

The compiler translates source code to machine code. A good compiler can **dramatically** reduce both instruction count and CPI.

**REAL EXAMPLE (your experiment!):**

```
Source code:
┌─────────────────────────────────────┐
│  long sum = 0;                      │
│  for (i = 0; i < 100000000; i++)    │
│      sum += i;                      │
└─────────────────────────────────────┘

Compiled with -O0 (no optimization):
┌─────────────────────────────────────┐
│  Loop executes 100,000,000 times    │
│  Each iteration: load, add, store   │
│                                     │
│  Cycles: 207,860,929                │
└─────────────────────────────────────┘

Compiled with -O2 (optimized):
┌─────────────────────────────────────┐
│  Compiler recognizes: sum = n(n-1)/2│
│  Replaces loop with single formula  │
│                                     │
│  Cycles: 3,759,080                  │
└─────────────────────────────────────┘

Same source code, 55x faster! Compiler eliminated 99.99% of work.
```

**Other compiler optimizations:**

```
┌─────────────────────────┬──────────────────────────────────────────┐
│ Optimization            │ Effect                                   │
├─────────────────────────┼──────────────────────────────────────────┤
│ Loop unrolling          │ Fewer branches → lower CPI               │
│ Inlining                │ No call/return → fewer instructions      │
│ Register allocation     │ Fewer memory accesses → lower CPI        │
│ Instruction scheduling  │ Hide latencies → lower CPI               │
│ Dead code elimination   │ Remove unused code → fewer instructions  │
│ Strength reduction      │ Replace expensive ops (mul→shift)        │
└─────────────────────────┴──────────────────────────────────────────┘
```

---

### 4. ISA (Instruction Set Architecture) → ALL THREE FACTORS

The ISA is the contract between hardware and software. It affects **everything**.

```
EXAMPLE: Compute A = B + C where B and C are in memory

RISC (ARM, RISC-V) - Simple instructions:
┌─────────────────────────────────────────────────┐
│  LDR  R1, [B]      ; Load B into register       │
│  LDR  R2, [C]      ; Load C into register       │
│  ADD  R3, R1, R2   ; Add registers              │
│  STR  R3, [A]      ; Store result               │
│                                                 │
│  4 instructions, CPI ≈ 1 each, fast clock       │
└─────────────────────────────────────────────────┘

CISC (x86) - Complex instructions:
┌─────────────────────────────────────────────────┐
│  ADD  [A], [B], [C]  ; Memory-to-memory add     │
│                      ; (hypothetical)           │
│                                                 │
│  1 instruction, but CPI ≈ 4-6, slower clock     │
└─────────────────────────────────────────────────┘
```

**How ISA affects clock rate:**

```
Simple ISA (RISC):
┌────────────────────────────────────────────────────────┐
│  Each instruction does ONE simple thing               │
│  → Hardware is simple                                  │
│  → Short critical path                                 │
│  → Can run at HIGH clock frequency (3+ GHz easily)     │
└────────────────────────────────────────────────────────┘

Complex ISA (CISC):
┌────────────────────────────────────────────────────────┐
│  Each instruction can do MANY things                   │
│  → Hardware is complex                                 │
│  → Long critical path                                  │
│  → Clock frequency LIMITED by slowest instruction      │
│                                                        │
│  (Modern x86 cheats: converts CISC → micro-ops RISC)   │
└────────────────────────────────────────────────────────┘
```

---

### The Complete Chain: From Source to Seconds

```
                        YOUR CODE
                            │
            ┌───────────────┴───────────────┐
            ▼                               │
      ┌──────────┐                          │
      │ALGORITHM │ "How much work?"         │
      └────┬─────┘                          │
           │ Determines problem size        │
           ▼                                │
      ┌──────────┐                          │
      │ LANGUAGE │ "How expressed?"         │  SOFTWARE
      └────┬─────┘                          │
           │ Abstraction overhead           │
           ▼                                │
      ┌──────────┐                          │
      │ COMPILER │ "How translated?"        │
      └────┬─────┘                          │
           │ Optimization quality           │
           ▼                               ─┤
      ┌──────────┐                          │
      │   ISA    │ "What instructions?"     │
      └────┬─────┘                          │
           │ Available operations           │  HARDWARE
           ▼                                │
      ┌──────────┐                          │
      │   CPU    │ "How executed?"          │
      └────┬─────┘                          │
           │                               ─┘
           ▼
      EXECUTION TIME (seconds)
```

---

### Summary Table with Examples

```
┌────────────┬─────────────────┬───────────────────────────────────────┐
│ Component  │ Bad Choice      │ Good Choice                           │
├────────────┼─────────────────┼───────────────────────────────────────┤
│ Algorithm  │ Bubble sort     │ Quick sort                            │
│            │ O(n²)           │ O(n log n)                            │
├────────────┼─────────────────┼───────────────────────────────────────┤
│ Language   │ Python loops    │ NumPy vectorized (calls C)            │
│            │ 100x slower     │ Near-native speed                     │
├────────────┼─────────────────┼───────────────────────────────────────┤
│ Compiler   │ -O0 (debug)     │ -O2 or -O3 (optimized)                │
│            │ 207M cycles     │ 3.7M cycles (your test!)              │
├────────────┼─────────────────┼───────────────────────────────────────┤
│ ISA        │ Stack machine   │ Register machine                      │
│            │ Many mem ops    │ Ops stay in registers                 │
└────────────┴─────────────────┴───────────────────────────────────────┘
```

**Key insight:** Performance is a **system-wide** property. You can't optimize just one layer and expect great results. The best performance comes from good choices at **every level**.

---

## [2026-01-26] From Gates to Pipelining: Understanding CPI, Latency, and Throughput

This section connects the dots from basic gates (N2T) to how modern CPUs achieve high performance.

### Part 1: Gates Take Time (The Physical Reality)

Every gate has **propagation delay** - time for output to change after input changes.

```
INPUT ───►│ NAND │───► OUTPUT
          └──────┘
              │
              └── Takes ~0.1 nanoseconds for output to appear

More gates in series = more delay:

INPUT ──►[NAND]──►[NAND]──►[NAND]──►[NAND]──► OUTPUT
             │        │        │        │
           0.1ns    0.1ns    0.1ns    0.1ns

         Total: 0.4 nanoseconds
```

**Different operations need different circuit depths:**

```
ADD:  Input ──►[10 gates deep]──► Output     (1 ns)
MUL:  Input ──►[40 gates deep]──► Output     (4 ns)
DIV:  Input ──►[200 gates deep]──► Output    (20 ns)
```

---

### Part 2: Why Clock Cycles Exist

**Problem:** Different operations go through different numbers of gates.

**Solution:** Create a CLOCK that says "everyone wait until signals settle"

```
Clock cycle = enough time for signals to propagate through gates

If clock cycle = 1 ns:
  ADD finishes in 1 cycle  (needs 1 ns, has 1 ns) ✓
  MUL needs 4 cycles       (needs 4 ns, gets 4 × 1 ns) ✓
  DIV needs 20 cycles      (needs 20 ns, gets 20 × 1 ns) ✓
```

---

### Part 3: What CPI Actually Means

CPI = **Cycles Per Instruction** = How many clock ticks until result is ready

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│   ADD instruction: Start ──► 1 tick ──► Done!     CPI = 1      │
│                                                                 │
│   MUL instruction: Start ──► tick ──► tick ──► tick ──► Done!  │
│                                        CPI = 3                  │
│                                                                 │
│   DIV instruction: Start ──► tick ──► tick ──► ... ──► Done!   │
│                                        CPI = 20                 │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

CPI is determined by HARDWARE COMPLEXITY:
  Simple circuit (few gates) → low CPI
  Complex circuit (many gates) → high CPI
```

---

### Part 4: Why Different Operations Have Different CPIs

**ADD: CPI ≈ 1 cycle**

```
ADD is SIMPLE - just wire propagation through gates

    A ──────┐
            ├──► ADDER (CLA) ──► Result
    B ──────┘

    Circuit: ~4 gate levels deep (with carry-lookahead)
    Time: Completes within 1 clock cycle
```

**MUL: CPI ≈ 3-4 cycles**

```
MUL is COMPLEX - requires multiple partial products + additions

    A × B = ?

    Step 1: Generate partial products (A × each bit of B)
    Step 2: Add all partial products together (tree of adders)

         A = 1101
       × B = 1011
       ────────────
         1101      ← A × B[0]
        1101       ← A × B[1], shifted
       0000        ← A × B[2], shifted
      1101         ← A × B[3], shifted
    ────────────
    = 10001111     ← Sum all (multiple adder stages)

    Much deeper circuit than ADD → more cycles
```

**DIV: CPI ≈ 20-80 cycles (!)**

```
DIV is ITERATIVE - like long division, one bit at a time

    A ÷ B = ?

    Cycle 1:  Can B fit into first part of A? Subtract or not.
    Cycle 2:  Shift, try again
    Cycle 3:  Shift, try again
    ...
    Cycle 64: Finally done (for 64-bit numbers)

    Unlike MUL, division is fundamentally SEQUENTIAL.
    Each step depends on the previous step's result.
    No clever hardware trick can make it fully parallel.
```

**CPI Summary by Operation:**

```
┌───────────┬─────────┬────────────────────────────────────────┐
│ Operation │ CPI     │ Why?                                   │
├───────────┼─────────┼────────────────────────────────────────┤
│ ADD/SUB   │ 1       │ Simple circuit, few gate levels        │
├───────────┼─────────┼────────────────────────────────────────┤
│ Bitwise   │ 1       │ Just AND/OR/XOR gates - trivial        │
│ (AND,OR)  │         │                                        │
├───────────┼─────────┼────────────────────────────────────────┤
│ Shift     │ 1       │ Just rewiring (barrel shifter)         │
├───────────┼─────────┼────────────────────────────────────────┤
│ MUL       │ 3-4     │ Partial products + adder tree          │
├───────────┼─────────┼────────────────────────────────────────┤
│ DIV       │ 20-80   │ Sequential bit-by-bit algorithm        │
├───────────┼─────────┼────────────────────────────────────────┤
│ Memory    │ 4-200+  │ Depends on cache hit/miss!             │
│ Load      │         │ L1: ~4, L2: ~12, L3: ~40, RAM: ~200    │
└───────────┴─────────┴────────────────────────────────────────┘
```

---

### Part 5: Pipelining (The Key to Modern CPU Performance)

Without pipelining, CPU does one instruction at a time:

```
WITHOUT PIPELINING (like N2T Hack computer):

Time:     1    2    3    4    5    6    7    8    9
         ┌─────────┐
Instr 1: │ FETCH │DECODE│EXECUTE│ WRITE │
         └─────────┘              └─────────┘
                                   ┌─────────┐
Instr 2:                           │ FETCH │DECODE│EXECUTE│ WRITE │
                                   └─────────┘              └─────────┘

Each instruction: 4 cycles
2 instructions: 8 cycles
Throughput: 1 instruction per 4 cycles (SLOW!)
```

**PIPELINING = Assembly Line for Instructions**

Think of a laundry assembly line:

```
         WASH    DRY     FOLD    PUT AWAY
         ─────   ─────   ─────   ─────────
Load 1:  ████
Load 2:         ████
Load 3:                 ████
Load 4:                         ████

After setup, ONE LOAD FINISHES EVERY CYCLE!
Even though each load takes 4 steps.
```

Applied to CPU:

```
WITH PIPELINING:

Time:        1       2       3       4       5       6       7
           ┌───────┬───────┬───────┬───────┐
Instr 1:   │FETCH  │DECODE │EXECUTE│ WRITE │
           └───────┴───────┴───────┴───────┘
                   ┌───────┬───────┬───────┬───────┐
Instr 2:           │FETCH  │DECODE │EXECUTE│ WRITE │
                   └───────┴───────┴───────┴───────┘
                           ┌───────┬───────┬───────┬───────┐
Instr 3:                   │FETCH  │DECODE │EXECUTE│ WRITE │
                           └───────┴───────┴───────┴───────┘
                                   ┌───────┬───────┬───────┬───────┐
Instr 4:                           │FETCH  │DECODE │EXECUTE│ WRITE │
                                   └───────┴───────┴───────┴───────┘

4 instructions in 7 cycles (not 16!)
After pipeline fills: 1 instruction completes EVERY cycle
```

---

### Part 6: The CPU is Physically Divided into Stages

**Key insight: Each stage is SEPARATE hardware!**

```
You might think the CPU is:

    ┌─────────────────────────────────────┐
    │           ONE BIG CPU               │
    │     (does everything together)      │
    └─────────────────────────────────────┘

But it's actually:

    ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐
    │  FETCH  │──►│ DECODE  │──►│ EXECUTE │──►│  WRITE  │
    └─────────┘   └─────────┘   └─────────┘   └─────────┘

    4 SEPARATE pieces of hardware connected by registers (DFFs!)
```

Each stage has its OWN circuits:

```
FETCH stage:        DECODE stage:       EXECUTE stage:
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│ • PC register│    │ • Instruction│    │ • ALU        │
│ • Memory read│    │   decoder    │    │ • Multiplier │
│ • Increment  │    │ • Register   │    │ • Shifter    │
│              │    │   file read  │    │              │
└──────────────┘    └──────────────┘    └──────────────┘
      │                   │                    │
      ▼                   ▼                    ▼
  Its own             Its own              Its own
  circuits!           circuits!            circuits!

They don't share! Each stage has its own gates, its own wires.
```

**How instructions flow simultaneously:**

```
Cycle 4 (all happening at the SAME time):

┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐
│   FETCH     │   │   DECODE    │   │   EXECUTE   │   │   WRITE     │
│             │   │             │   │             │   │             │
│ Working on  │   │ Working on  │   │ Working on  │   │ Working on  │
│ Instr #4    │   │ Instr #3    │   │ Instr #2    │   │ Instr #1    │
│             │   │             │   │             │   │             │
│ (ADD R5,R6) │   │ (MUL R3,R4) │   │ (SUB R1,R2) │   │ (AND R7,R8) │
└─────────────┘   └─────────────┘   └─────────────┘   └─────────────┘

4 different pieces of hardware working on 4 different instructions!
```

**DFFs (from N2T!) separate the stages:**

```
                    CLOCK TICK
                        │
                        ▼
┌─────────┐  ┌─────┐  ┌─────────┐  ┌─────┐  ┌─────────┐
│  FETCH  │──│ DFF │──│ DECODE  │──│ DFF │──│ EXECUTE │
└─────────┘  └─────┘  └─────────┘  └─────┘  └─────────┘
                │                      │
                ▼                      ▼
         "Pipeline               "Pipeline
          Register"               Register"

On each clock tick:
  - DFFs capture the output of each stage
  - Pass it to the next stage
  - Each stage starts working on NEW data
```

---

### Part 7: Latency vs Throughput

Two different ways to measure performance:

```
LATENCY = Time from START to FINISH for ONE instruction
          (How long until I get my result?)

THROUGHPUT = How many instructions COMPLETE per cycle
             (How many results per second?)
```

**Laundry analogy:**

```
LATENCY:     Each load takes 2 hours (wash → dry → fold → put away)

THROUGHPUT:  With 4 machines working as pipeline,
             1 load finishes every 30 minutes (after initial 2 hour wait)

You wait 2 hours for your FIRST clean load.
But then one load comes out every 30 minutes.
```

**CPU example (MUL with 3-stage pipeline):**

```
MUL: Latency = 3 cycles, Throughput = 1/cycle

The multiplier has 3 internal stages:

┌─────────┐   ┌─────────┐   ┌─────────┐
│ MUL     │   │ MUL     │   │ MUL     │
│ Stage 1 │──►│ Stage 2 │──►│ Stage 3 │──► Result
└─────────┘   └─────────┘   └─────────┘

Cycle 5:
┌─────────┐   ┌─────────┐   ┌─────────┐
│ MUL #4  │   │ MUL #3  │   │ MUL #2  │──► Result of MUL #2
│ starting│   │ middle  │   │ finishing│
└─────────┘   └─────────┘   └─────────┘

Each MUL takes 3 cycles to complete (latency = 3).
But after warmup, one MUL finishes every cycle (throughput = 1/cycle).
```

---

### Part 8: Modern CPU Hierarchy (Apple M1 Example)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              M1 CHIP (the whole thing)                      │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                         CPU (8 cores)                                │   │
│  │                                                                      │   │
│  │   ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐                   │   │
│  │   │ P-Core  │ │ P-Core  │ │ P-Core  │ │ P-Core  │  ← 4 Performance  │   │
│  │   │ (fast)  │ │ (fast)  │ │ (fast)  │ │ (fast)  │    Cores          │   │
│  │   └─────────┘ └─────────┘ └─────────┘ └─────────┘                   │   │
│  │                                                                      │   │
│  │   ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐                   │   │
│  │   │ E-Core  │ │ E-Core  │ │ E-Core  │ │ E-Core  │  ← 4 Efficiency   │   │
│  │   │ (slow)  │ │ (slow)  │ │ (slow)  │ │ (slow)  │    Cores          │   │
│  │   └─────────┘ └─────────┘ └─────────┘ └─────────┘                   │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌──────────────────┐    │
│  │     GPU (8 cores)   │  │   Neural Engine     │  │   Shared Memory  │    │
│  └─────────────────────┘  └─────────────────────┘  └──────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Zoom into ONE Core:**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        ONE PERFORMANCE CORE (Firestorm)                     │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                         FRONT END (Fetch + Decode)                    │  │
│  │  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────────┐   │  │
│  │  │   FETCH     │───►│   DECODE    │───►│   RENAME/SCHEDULE       │   │  │
│  │  │ Get instrs  │    │ Figure out  │    │ Assign to execution     │   │  │
│  │  │ from cache  │    │ what to do  │    │ units, handle deps      │   │  │
│  │  └─────────────┘    └─────────────┘    └─────────────────────────┘   │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                      │                                      │
│                                      ▼                                      │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                EXECUTION UNITS (many working in parallel!)            │  │
│  │                                                                       │  │
│  │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────────┐ │  │
│  │  │ ALU  │ │ ALU  │ │ ALU  │ │ ALU  │ │ MUL  │ │ MUL  │ │   DIV    │ │  │
│  │  │  #1  │ │  #2  │ │  #3  │ │  #4  │ │  #1  │ │  #2  │ │          │ │  │
│  │  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘ └──────────┘ │  │
│  │                                                                       │  │
│  │  ┌──────┐ ┌──────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐   │  │
│  │  │BRANCH│ │BRANCH│ │  LOAD    │ │  LOAD    │ │      STORE       │   │  │
│  │  │  #1  │ │  #2  │ │  UNIT #1 │ │  UNIT #2 │ │      UNIT        │   │  │
│  │  └──────┘ └──────┘ └──────────┘ └──────────┘ └──────────────────┘   │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                      │                                      │
│                                      ▼                                      │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │  WRITE BACK - Results go back to registers                            │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Why M1 can do 4 ADDs per cycle:**

```
M1 has 4 ALUs in each core!

Cycle N:
    ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐
    │ ALU  │ │ ALU  │ │ ALU  │ │ ALU  │
    │  #1  │ │  #2  │ │  #3  │ │  #4  │
    │      │ │      │ │      │ │      │
    │ ADD  │ │ SUB  │ │ AND  │ │ OR   │
    │ R1+R2│ │ R3-R4│ │ R5&R6│ │ R7|R8│
    └──────┘ └──────┘ └──────┘ └──────┘
        │        │        │        │
        ▼        ▼        ▼        ▼
    Result 1 Result 2 Result 3 Result 4

    4 instructions completed in 1 cycle!
    Throughput = 4 ADDs per cycle
```

---

### Part 9: Two Kinds of Parallelism

**MULTIPLE CORES (8 in M1):**

```
• Run completely DIFFERENT programs/threads
• Your browser on Core 1, Spotify on Core 2
• Operating system decides which core runs what

    Core 1: Running Chrome
    Core 2: Running Spotify
    Core 3: Running your C program
    Core 4: Idle
    ...
```

**MULTIPLE EXECUTION UNITS (inside each core):**

```
• Run multiple instructions from the SAME program
• CPU automatically finds independent instructions

    Your program: a = b + c;
                  d = e + f;   ← These don't depend on each other!
                  g = h + i;      CPU runs them in parallel
                  j = k + l;

    ALU 1: b + c
    ALU 2: e + f    ← Same cycle!
    ALU 3: h + i
    ALU 4: k + l
```

---

### Part 10: The Complete Hierarchy

```
┌─────────────────────────────────────────────────────────────────────────┐
│  LEVEL           │  WHAT IT IS                │  HOW MANY (M1)         │
├──────────────────┼────────────────────────────┼────────────────────────┤
│                  │                            │                        │
│  CHIP            │  The whole M1 silicon      │  1                     │
│      │           │                            │                        │
│      ▼           │                            │                        │
│  CPU             │  The processor part        │  1 (contains cores)    │
│      │           │                            │                        │
│      ▼           │                            │                        │
│  CORE            │  Independent processor     │  8 (4 fast + 4 slow)   │
│      │           │  with its own pipeline     │                        │
│      ▼           │                            │                        │
│  PIPELINE        │  Stages that process       │  1 per core            │
│      │           │  instructions in sequence  │                        │
│      ▼           │                            │                        │
│  EXECUTION UNITS │  ALUs, multipliers, etc.   │  ~12 per core          │
│      │           │  that do actual work       │                        │
│      ▼           │                            │                        │
│  GATES           │  NANDs, flip-flops         │  ~16 billion           │
│                  │                            │                        │
└─────────────────────────────────────────────────────────────────────────┘
```

---

### Key Takeaways

```
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│  1. GATES → CPI                                                         │
│     More gates in circuit = more time = higher CPI                      │
│     ADD: few gates → CPI=1, DIV: many iterations → CPI=20+              │
│                                                                         │
│  2. CLOCK exists to synchronize                                         │
│     Lets all circuits settle before reading results                     │
│                                                                         │
│  3. PIPELINING overlaps instruction execution                           │
│     Different stages work on different instructions simultaneously      │
│     DFFs separate stages (just like N2T!)                               │
│                                                                         │
│  4. LATENCY vs THROUGHPUT                                               │
│     Latency = time for ONE instruction (3 cycles for MUL)               │
│     Throughput = instructions per cycle (can be 1/cycle with pipeline)  │
│                                                                         │
│  5. MODERN CPUs have multiple execution units                           │
│     M1 has 4 ALUs per core → 4 ADDs per cycle!                          │
│                                                                         │
│  6. PARALLELISM at every level                                          │
│     Multiple cores (different programs)                                 │
│     Multiple execution units (same program, independent instructions)   │
│     Pipelining (overlapped stages)                                      │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

**See also:** N2T Chapter 3 (DFFs enable pipelining), N2T Chapter 5 (CPU architecture)

---

## [2026-01-26] How Source Code Becomes Machine Code: The Compilation Pipeline

This section explains the transformation from source code to running program across different languages.

### The General Pipeline (Simplified)

```
Source Code (.c)
      │
      ▼ Preprocessor (#include, #define)
Expanded Source
      │
      ▼ Compiler
Assembly Code (.s)
      │
      ▼ Assembler
Object Code (.o)
      │
      ▼ Linker (combines with libraries)
Executable (machine code)
      │
      ▼ Loader (OS loads into memory)
Running in Memory → CPU executes
```

Each stage transforms the code closer to what the CPU understands.

---

## Part 1: C Language Pipeline (Detailed)

### Stage 1: Preprocessor

**What it does:** Textual transformation before actual compilation

```c
// Original: hello.c
#include <stdio.h>
#define MAX 100

int main() {
    int arr[MAX];
    printf("Hello");
    return 0;
}
```

**After preprocessing (hello.i):**

```c
// All of stdio.h gets inserted here (hundreds of lines)
// #include <stdio.h> → entire file contents

// #define MAX 100 replaced everywhere
int main() {
    int arr[100];  // ← MAX replaced with 100
    printf("Hello");
    return 0;
}
```

**Preprocessor jobs:**
- `#include` → paste entire file
- `#define` → text substitution
- `#ifdef` → conditional code inclusion

**Command:** `gcc -E hello.c > hello.i`

---

### Stage 2: Compiler (High-Level → Assembly)

**What it does:** Transforms C code into CPU instructions (assembly language)

```c
// Input: hello.c (after preprocessing)
int add(int a, int b) {
    return a + b;
}

int main() {
    int result = add(5, 3);
    return result;
}
```

**Output: hello.s (ARM64 assembly, simplified)**

```asm
add:                          ; Function label
    ADD W0, W0, W1            ; Add W0 (a) + W1 (b) → W0
    RET                       ; Return W0 to caller

main:
    STP FP, LR, [SP, #-16]!   ; Save frame pointer & return address
    MOV W0, #5                ; Put 5 in W0 (first arg to add)
    MOV W1, #3                ; Put 3 in W1 (second arg to add)
    BL add                    ; Call add (branch with link)
    ; W0 now contains 8
    MOV W1, W0                ; Move result to W1
    MOV W0, #0                ; Return value
    LDP FP, LR, [SP], #16     ; Restore frame pointer & return address
    RET
```

**What the compiler does:**
1. Parse C syntax → Abstract Syntax Tree (AST)
2. Type checking & optimization passes
3. Generate assembly instructions
4. Allocate registers, manage stack

**Command:** `gcc -S hello.c` (produces hello.s)

---

### Stage 3: Assembler (Assembly → Machine Code)

**What it does:** Converts human-readable assembly into machine code (bytes)

```asm
Input (hello.s):
    ADD W0, W0, W1
    RET
```

**Output (binary, shown as hex):**

```
ADD W0, W0, W1  →  8B 01 00 11  (4 bytes of machine code)
RET             →  C0 03 5F D6  (4 bytes of machine code)
```

**The assembler also:**
- Converts instruction names to opcodes
- Resolves labels to byte offsets
- Produces object file (.o) with metadata

**Command:** `gcc -c hello.c` (produces hello.o)

```
hello.o contains:
├── Code section (.text) - the binary instructions
├── Data section (.data) - initialized variables
├── Symbol table - function/variable names + addresses (not yet final)
└── Relocation info - "this address needs to be fixed by linker"
```

---

### Stage 4: Linker (Combines Object Files + Libraries)

**Problem:** `hello.o` refers to `printf` but doesn't have its code

```asm
BL printf      ; ← Where is printf? Address unknown!
```

**What the linker does:**

1. **Collects all .o files** from your program
2. **Finds library functions** (printf from libc)
3. **Resolves undefined symbols** (fixes addresses)
4. **Arranges in memory layout**

```
Before linking (hello.o):
┌──────────────────────────────────┐
│ Code:                            │
│   main()                         │
│   add()                          │
│   BL printf  ← ??? (unresolved)  │
└──────────────────────────────────┘

After linking (hello executable):
┌──────────────────────────────────┐
│ 0x100000000:  main()             │
│ 0x100000040:  add()              │
│ 0x100000080:  BL printf          │
│              (now BL 0x1ffff000) │
├──────────────────────────────────┤
│                                  │
│ ... (other program code)         │
│                                  │
├──────────────────────────────────┤
│ 0x1ffff000:  printf() [from libc]│
│ 0x1ffff100:  strlen() [from libc]│
│ ... (standard library functions) │
└──────────────────────────────────┘
```

**Command:** `gcc hello.c -o hello` (preprocesses, compiles, assembles, links)

---

### Stage 5: Loader (OS puts executable in memory)

**What the loader does:**

```
Executable on disk (hello)
      │
      ▼ (OS system call: execve)
Loader (kernel code)
      │
      ├─ Read executable format (Mach-O on macOS, ELF on Linux)
      ├─ Allocate memory pages
      ├─ Load code section into memory
      ├─ Load data section into memory
      ├─ Create stack
      ├─ Create heap
      └─ Jump to main()
           │
           ▼
      CPU starts executing at main
```

**Memory layout after loading:**

```
┌──────────────────────────────────┐ High addresses
│         Stack ↑                  │  (grows upward)
├──────────────────────────────────┤
│         Free space               │
├──────────────────────────────────┤
│         Heap ↓                   │  (grows downward)
├──────────────────────────────────┤
│  Uninitialized data (.bss)       │
├──────────────────────────────────┤
│  Initialized data (.data)        │
├──────────────────────────────────┤
│  Code (.text)                    │
└──────────────────────────────────┘ Low addresses
```

---

## Part 2: C Compilation Summary

```
┌──────────────────────────────────────────────────────────────────┐
│                    C COMPILATION PIPELINE                        │
├──────────┬──────────┬────────────────────────────────────────────┤
│ Stage    │ Input    │ Output        │ Tool            │ What     │
├──────────┼──────────┼───────────────┼─────────────────┼──────────┤
│ 1        │ hello.c  │ hello.i       │ Preprocessor    │ Text     │
│          │          │               │ (cpp)           │ subst.   │
├──────────┼──────────┼───────────────┼─────────────────┼──────────┤
│ 2        │ hello.i  │ hello.s       │ Compiler        │ High →   │
│          │          │               │ (gcc/clang)     │ Low      │
├──────────┼──────────┼───────────────┼─────────────────┼──────────┤
│ 3        │ hello.s  │ hello.o       │ Assembler       │ Text →   │
│          │          │               │ (as)            │ Binary   │
├──────────┼──────────┼───────────────┼─────────────────┼──────────┤
│ 4        │ *.o      │ hello         │ Linker          │ Combine  │
│          │ + libs   │               │ (ld)            │ + link   │
├──────────┼──────────┼───────────────┼─────────────────┼──────────┤
│ 5        │ hello    │ Running       │ Loader (OS)     │ Load +   │
│          │          │ in memory     │ execve()        │ execute  │
└──────────┴──────────┴───────────────┴─────────────────┴──────────┘
```

---

## Part 3: TypeScript Compilation (Very Different!)

TypeScript is **NOT compiled to machine code directly**. It's compiled to JavaScript, which runs in a runtime.

### Pipeline: TypeScript → JavaScript → Execution

```
TypeScript (.ts)
      │
      ▼ TypeScript Compiler (tsc)
JavaScript (.js)
      │
      ▼ JavaScript Runtime (Node.js, Browser, Deno)
      │
      ├─ JIT Compiler (V8, SpiderMonkey)
      │  │
      │  ▼ Machine Code (optimized)
      │
      └─ Interpreter (fallback)
           │
           ▼
      CPU executes
```

### Stage 1: TypeScript Compiler (tsc)

**What it does:** Type checking + transpile to JavaScript

```typescript
// input.ts (TypeScript)
function add(a: number, b: number): number {
    return a + b;
}

const result: number = add(5, 3);
console.log(result);
```

**Output: input.js (JavaScript - types removed!)**

```javascript
function add(a, b) {
    return a + b;
}

const result = add(5, 3);
console.log(result);
```

**TypeScript compiler jobs:**
- Type checking (catches errors at compile time)
- Strip away type annotations (JS doesn't have them)
- Transpile modern features (async/await, classes, etc.)
- Output JavaScript

**Command:** `tsc input.ts` (produces input.js)

---

### Stage 2: JavaScript Runtime (Interpretation + JIT Compilation)

**Two-step process:**

**A) Interpreter reads JavaScript**

```
Node.js (or Browser V8 engine)
│
├─ Parser: JavaScript source → AST
│
├─ Interpreter: Execute AST directly (slow)
│
└─ If code runs hot (loop iterations): JIT compile → native code
```

**B) JIT Compiler optimizes hot code**

```javascript
// First few times: INTERPRETED
function fibonacci(n) {
    if (n < 2) return n;
    return fibonacci(n-1) + fibonacci(n-2);
}

for (let i = 0; i < 1000000; i++) {
    fibonacci(10);  // ← Called millions of times!
}
                    // ← Profiler sees this runs hot
                    // ← JIT compiler converts to machine code
                    // ← Suddenly 10-50x faster!
```

**Why JIT instead of static compilation?**
- JavaScript is dynamically typed (types only known at runtime)
- Can optimize based on actual values seen
- Can inline, specialize, and optimize better than static compiler

---

## Part 4: Rust Compilation (Powerful Static Compilation)

Rust compiles **all the way to machine code**, like C.

### Pipeline: Rust → LLVM IR → Assembly → Machine Code

```
Rust (.rs)
      │
      ▼ Rust Compiler (rustc) Front-end
      │  ├─ Parse Rust syntax
      │  ├─ Type checking (very strict!)
      │  ├─ Borrow checking (ownership rules)
      │
LLVM Intermediate Representation (IR)
      │
      ▼ LLVM Backend (optimization passes)
      │  ├─ High-level optimizations
      │  ├─ Platform-specific optimizations
      │
Assembly (.s)
      │
      ▼ LLVM Assembler
Object Code (.o)
      │
      ▼ Linker
Executable (machine code)
      │
      ▼ Loader (OS)
      │
      ▼
CPU executes
```

### Rust vs C Compilation

```
C:                          Rust:
Source → AST → Assembly     Source → AST → Borrow Check
         (simple)                  (complex safety checks)
         → Binary                  → LLVM IR
                                   → Assembly
                                   → Binary

Rust takes LONGER to compile but produces:
  • Memory-safe code (no buffer overflows)
  • Thread-safe code (enforced at compile time)
  • Zero-cost abstractions (compiler removes overhead)
```

### Example: Rust Code Compilation

```rust
// main.rs
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let result = add(5, 3);
    println!("Result: {}", result);
}
```

**Command:** `rustc main.rs -o main`

```
1. Rust frontend:
   - Parse syntax
   - Type check (i32 + i32 → i32)
   - Borrow check (no ownership violations)

2. Generate LLVM IR:
   define i32 @add(i32 %a, i32 %b) {
       %sum = add i32 %a, %b
       ret i32 %sum
   }

3. LLVM optimization:
   - Dead code elimination
   - Inlining
   - Loop unrolling

4. Generate machine code:
   ADD R0, R0, R1
   RET

5. Linker combines with std library

6. Executable
```

**Rust unique feature: Zero-cost abstractions**

```rust
// Rust code (looks high-level)
let vec = vec![1, 2, 3];
for x in vec {
    println!("{}", x);
}
```

**Compiles to (efficient assembly):**

```asm
; Loop unrolled and optimized by compiler
LOAD R0, [vec+0]    ; Load vec[0]
PRINT R0
LOAD R0, [vec+4]    ; Load vec[1]
PRINT R0
LOAD R0, [vec+8]    ; Load vec[2]
PRINT R0
```

---

## Part 5: Zig Compilation (Low-level with High-level Features)

Zig is like "a better C" - compiles to machine code with direct hardware control.

### Pipeline: Zig → LLVM IR → Machine Code

```
Zig (.zig)
      │
      ▼ Zig Compiler (zig) Front-end
      │  ├─ Parse Zig syntax
      │  ├─ Type checking
      │  ├─ Compile-time function execution
      │
LLVM IR
      │
      ▼ LLVM Backend
      │
Assembly (.s)
      │
      ▼ Assembler → Object Code
      │
      ▼ Linker → Executable
      │
      ▼ Loader → Running
```

### Why Zig?

```
C:          Manual memory management, unsafe
Zig:        Manual memory management, but SAFER
Rust:       Enforced safety, but complex borrow checker
Zig goal:   C's control + safety without complexity
```

### Example: Zig Code

```zig
// main.zig (Very similar to C!)
const std = @import("std");

fn add(a: i32, b: i32) i32 {
    return a + b;
}

pub fn main() void {
    const result = add(5, 3);
    std.debug.print("Result: {}\n", .{result});
}
```

**Features Zig adds over C:**
- `const` by default (immutability)
- Compile-time function execution
- Cleaner syntax (no { }; clutter)
- Better error handling
- Direct LLVM backend (like Rust)

---

## Part 6: Comparison Table

```
┌────────────┬──────────────┬─────────────────┬────────────────────┐
│ Language   │ Type System  │ Compile Target  │ Key Feature        │
├────────────┼──────────────┼─────────────────┼────────────────────┤
│ C          │ Static       │ Machine code    │ Speed + simplicity │
│            │              │ (via assembler) │ (but unsafe)       │
├────────────┼──────────────┼─────────────────┼────────────────────┤
│ TypeScript │ Static*      │ JavaScript      │ Type safety for JS │
│            │ (erased)     │ (runs in VM)    │ (*erased at link)  │
├────────────┼──────────────┼─────────────────┼────────────────────┤
│ Rust       │ Static       │ Machine code    │ Safety + speed     │
│            │ (strict)     │ (via LLVM)      │ (complex compiler) │
├────────────┼──────────────┼─────────────────┼────────────────────┤
│ Zig        │ Static       │ Machine code    │ C-like simplicity  │
│            │ (flexible)   │ (via LLVM)      │ + modern features  │
└────────────┴──────────────┴─────────────────┴────────────────────┘
```

---

## Part 7: Execution Time Comparison

For the same program (sum 1 billion integers):

```
┌────────────┬──────────────────┬─────────────────────────────┐
│ Language   │ Compilation Time │ Execution Time              │
├────────────┼──────────────────┼─────────────────────────────┤
│ C (-O2)    │ 0.5 seconds      │ 0.15 seconds (fast!)        │
├────────────┼──────────────────┼─────────────────────────────┤
│ Rust (-O)  │ 3 seconds        │ 0.14 seconds (optimized)    │
├────────────┼──────────────────┼─────────────────────────────┤
│ Zig (-O)   │ 1 second         │ 0.14 seconds (same as Rust) │
├────────────┼──────────────────┼─────────────────────────────┤
│ TypeScript │ < 0.1 seconds    │ 2.5 seconds (VM overhead)   │
│ (Node.js)  │ (tsc is fast!)   │ (even with JIT)             │
└────────────┴──────────────────┴─────────────────────────────┘

Key insight:
  C/Rust/Zig all compile to machine code → similar speed
  TypeScript compiles to JavaScript → VM/JIT adds overhead
  Rust takes longer to compile (safety checks) but same performance
```

---

## Summary

```
┌─────────────────────────────────────────────────────────────────────┐
│                    COMPILATION PIPELINE SUMMARY                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  C:         Preprocessor → Compiler → Assembler → Linker → Loader  │
│             (Low-level, manual, fast compilation)                   │
│                                                                     │
│  TypeScript: tsc → JavaScript → Runtime JIT/Interpreter             │
│             (High-level, types erased, runtime compilation)         │
│                                                                     │
│  Rust:      Compiler → LLVM IR → Assembler → Linker → Loader       │
│             (Type-safe, borrow-checked, slow compilation)           │
│                                                                     │
│  Zig:       Compiler → LLVM IR → Assembler → Linker → Loader       │
│             (C-like, but safer, modern features)                    │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**Key Takeaway:**
- C/Rust/Zig compile to **machine code** (CPU executes directly)
- TypeScript compiles to **JavaScript** (runtime interprets/JITs)
- The compilation process transforms higher-level abstractions into lower-level operations the CPU can execute
- Each language makes different trade-offs: simplicity vs safety, compile time vs runtime speed

---
