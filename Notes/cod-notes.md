# COD Notes

_Entries follow the template at `Notes/TEMPLATE.md`. Append-only. **Newest entry at top**, immediately after this header._

---

## [2026-05-15] Yield, Performance, and the CPU Time Equation · pp.50–60 · Ch.1 §1.5 → §1.6

### TL;DR
The chapter closes §1.5 with the **economics of silicon** — chips are sliced from a round wafer, defects are random and unavoidable, so the cost of an IC scales *super-linearly* with die area through three coupled equations (`cost/die = cost/wafer ÷ (dies/wafer × yield)`). It then opens §1.6 by warning that "performance" is not one number: **response time** (how long *this* task takes) and **throughput** (how many tasks per second) are different objectives, often pulled in different directions by the same change. The section lands on the chapter's single most-used formula, the **CPU performance equation** in its primitive form `CPU time = clock cycles ÷ clock rate`, soon to be refactored into the three-axis form `IC × CPI × clock period` — the equation every later chapter ultimately attacks.

### History — "why does this exist?"
The **yield equation** in the Elaboration box (p.53) traces back to **B. T. Murphy's 1964 RCA paper** on integrated-circuit defect statistics, which modeled defect distributions as Poisson — the `(1 + (defects×area)/2)⁻²` form is the empirical refinement Carver Mead and his students settled on at Caltech in the 1970s and that AMD/Intel still use for first-order cost projections today. The **response-time vs throughput distinction** was crystallized by the timesharing systems of the 1960s (CTSS at MIT 1961, Multics 1965): when one CPU serves dozens of users, the *system manager's* goal (maximize jobs/hour) diverges from the *user's* goal (minimize wait for *my* job). **Gene Amdahl's 1967 keynote** ("Validity of the single-processor approach") was the first paper to argue that you must name your metric *before* you optimize. The CPU performance equation in the form `T = IC × CPI × τ` was popularized by **Hennessy & Patterson's first edition (1990)** and is now the universal vocabulary of computer architecture papers.

### Intuition — "this is like…"
A **bakery on a delivery rush**. Yield is "how many cookies in a tray come out unburnt" — the bigger each cookie (die size), the more likely *any one of them* gets the burnt spot, so the bigger the cookie, the lower the percentage that survive. Response time vs throughput is the difference between "how long until *my* order arrives" (a courier on a moped, fast for one) and "how many orders does the bakery ship per hour" (a delivery van full of orders, slow for any *one* customer but high total bandwidth). The CPU performance equation just names the three knobs the bakery has: number of cookies you have to bake (instruction count), average time per cookie (CPI), and how fast the oven cycles (clock period).

### Mechanics

**Yield: why big dies are punishingly expensive.**

```
                wafer area
dies/wafer  ≈  ───────────
                die area

           1
yield  ≈ ─────────────────────
         (1 + (D × A)/2)²

cost/die = cost/wafer ÷ (dies/wafer × yield)
```

Double the die area, and cost roughly *quadruples* — fewer dies fit, **and** each is more likely to contain a defect. This is the economic engine behind every "chiplet" architecture (AMD Zen, Apple M-series Ultra): split one giant die into N smaller dies and stitch them together with a fabric. Smaller dies → higher yield → far lower cost-per-good-mm².

**Response time vs throughput, the four-cell truth table:**

| Change | Response time | Throughput |
|---|---|---|
| Faster CPU (same core count) | ↓ improves | ↑ improves |
| Add cores, separate tasks | unchanged for any one task | ↑ improves |
| Add cores, parallelize one task | ↓ improves | ↑ improves |
| Faster disk (CPU-bound workload) | unchanged | unchanged |

The pedagogical trap the chapter sets: students assume "throughput up = response time down." Sometimes (case 1). Not always (case 2). Knowing which cell you're in is half the job.

**The CPU performance equation, four equivalent forms:**

```
CPU time = clock cycles / clock rate          (form 1)
         = clock cycles × clock period        (form 2)
         = IC × CPI × clock period            (form 3, the canonical one)
         = IC × CPI / clock rate              (form 4)

where:
  IC   = instruction count (compiler + program)
  CPI  = clock cycles per instruction (microarchitecture + program)
  τ    = clock period (circuit design + process node)
```

The three axes are owned by **different teams**:
- **IC**: compiler + algorithm choice (a better `O(n log n)` sort beats every µarch optimization)
- **CPI**: microarchitecture — pipelining, caches, branch prediction, ILP
- **τ**: process node + circuit design — 7nm → 5nm shrinks τ but raises NRE cost

Worked example (the textbook's, p.59–60): a program runs in 10 s on A (2 GHz). Target on B is 6 s but B requires 1.2× the cycles. Required clock rate: `cycles_A = 10 × 2×10⁹ = 2×10¹⁰`; `cycles_B = 1.2 × 2×10¹⁰ = 2.4×10¹⁰`; `rate_B = 2.4×10¹⁰ / 6 = 4 GHz`. **B must be twice A's clock just to be 1.67× faster overall** — the 1.2× CPI inflation eats almost half the clock gain.

### If you were the architect…
You're handed a 30% performance target with a fixed transistor budget. Which axis do you push? **CPI** has the most slack at modern process nodes — clock rates have been pinned near 4–5 GHz since 2005 (the "power wall"), and IC is mostly the compiler's job. So you spend the budget on bigger caches, deeper out-of-order windows, and better branch predictors. Apple's M-series chips and AMD's Zen are textbook executions of this: they barely beat Intel on clock rate, but they crush it on IPC (1/CPI) — through 8-wide decode, 600+ in-flight instructions, and L1 caches twice the size of Intel's.

### Cross-language view
The equation is language-agnostic but **IC is language-dependent**:
```c
// C — one machine instruction per source operation, ~1.0× baseline IC
for (int i=0; i<n; i++) sum += a[i];
```
```go
// Go — adds a bounds check per index, ~1.2–1.4× IC unless escape-analyzed away
for i := 0; i < n; i++ { sum += a[i] }
```
```python
# Python — each += is a dictionary lookup + PyObject alloc, ~50–200× IC
for x in a: sum += x          # CPython interpreter
```
The same algorithm at the same big-O can differ by **two orders of magnitude in IC** across these — which is why the equation predicts Python is slow without any reference to microarchitecture.

### Where this shows up in real systems
- **AMD chiplets (Zen 2, 2019).** Instead of one 400 mm² monolithic die, AMD shipped eight 74 mm² compute chiplets plus an I/O die. Yield math directly explains why this slashed cost per core ~40% at the same node.
- **TPC-C benchmarks vs TPC-H.** TPC-C is response-time-bounded (transactional OLTP) — vendors tune for low latency, small batches. TPC-H is throughput-bounded (analytical) — vendors tune for large scans and parallel query. Same hardware, different numbers, because the *metric is the spec*.
- **`perf stat` output.** Every line — `instructions`, `cycles`, `IPC` — is a direct measurement of the chapter's equation in production. Senior perf engineers read these the way doctors read EKGs.

### Diagnostic questions
1. **Q:** Why does shrinking a die from 400 mm² → 100 mm² often cut cost more than 4×?
   *Wrong-answer trap:* "Fewer transistors." The real reason: yield is super-linear — smaller dies catch fewer defects, so the *fraction* of good dies rises sharply, multiplying the 4× area gain.
2. **Q:** A web service adds a second CPU. Response time per request: unchanged. Did the upgrade fail?
   *Wrong-answer trap:* "Yes." If the workload was queue-bound (requests piling up), throughput went up *and* effective response time fell because queue wait dropped. You need to measure under load, not in isolation.
3. **Q:** GPU has lower clock rate than CPU but 100× the throughput on matrix multiply. Why doesn't the CPU equation predict this?
   *Wrong-answer trap:* "The equation is wrong." It isn't — the equation describes *one* execution stream. GPUs win by running thousands of streams in parallel; you'd apply the equation per-stream and then multiply by the number of streams.
4. **Q:** Two CPUs have identical IC and CPI on a program; CPU A is faster. What's the only remaining knob?
   *Wrong-answer trap:* "More cores." A single program with fixed IC and CPI runs on one stream; the only remaining term is τ (clock period). A has the higher clock.

### See also
- COD §1.7 (next chunk): **Amdahl's Law** — the formal limit on what speeding up one axis can buy you.
- DDIA Ch.1: response time percentiles (p50, p99) — the practitioner's refinement of "response time" once you admit it has a distribution.
- OSTEP Ch.7 (scheduling): the same throughput/response-time tension at the OS level — schedulers pick between SJF (response time) and round-robin (throughput).

---

## [2026-05-14] The Translation Stack — From C to Bits, and Why the ISA Is the Contract · pp.39–49 · Ch.1 §1.3–1.4

### TL;DR
Patterson & Hennessy walk down the abstraction tower from a `swap()` function in C to the actual 32-bit machine words an ARM CPU executes, naming each translation step and the program that performs it: **compiler** turns high-level source into assembly, **assembler** turns assembly into binary machine code. The chapter then opens the case of an iPad 2 to show what that binary actually runs on — processor (datapath + control), DRAM main memory, SRAM cache, flash secondary memory — and lands on the chapter's structural climax: the **Instruction Set Architecture (ISA)** is the abstract interface that makes this whole tower portable, because every layer above it depends only on the ISA's *contract*, not on which silicon implements it. The ABI extends that contract with the OS's calling and syscall conventions, which is why a binary compiled for ARMv8 Linux runs on a Raspberry Pi and a Graviton server without recompilation.

### History — "why does this exist?"
The first programmers (ENIAC, 1945; Manchester Baby, 1948) wrote programs as literal binary patterns on patch panels or paper tape. **Kathleen Booth's assembler (1947)** and **Grace Hopper's A-0 compiler (1952)** were the two breakthroughs that made the tower possible — Hopper's slogan was "the computer is going to help program the computer." **John Backus's FORTRAN (1957)** was the first widely used high-level compiler and is the moment "writing assembly by default" died for scientific code. The **ISA-as-contract** idea was named by **Fred Brooks and Gene Amdahl on the IBM System/360 (1964)**: instead of a new instruction set per machine, IBM shipped six different implementations (Models 30, 40, 50, 65, 75, 91) of one ISA at different price/performance points — the first time customers could buy "the same computer, faster" without recompiling. Every CPU family since (x86 1978, ARM 1985, RISC-V 2010) is a continuation of that 1964 decision.

### Intuition — "this is like…"
A **legal contract translated through three languages**. The original deal is written in English (your C source) — that's what the human parties negotiate in. A paralegal (the compiler) translates it into formal legalese (assembly) so it's unambiguous. A clerk (the assembler) types the legalese into a structured filing system (binary machine code) so the courthouse computer can index it. The **ISA is the legal jurisdiction** — it specifies which clauses are enforceable and which aren't. As long as your binary respects the jurisdiction, it doesn't care whether the courthouse is in Boston or Phoenix (Cortex-A53 phone vs. Graviton3 server). Change the jurisdiction (x86 → ARM) and your binary becomes meaningless paper.

### Mechanics

**The full translation chain, from the chapter's Figure 1.4:**

```
  ┌──────────────────────────┐
  │ High-level source (C):   │
  │   swap(int v[], int k){  │
  │     int temp = v[k];     │   ← human-readable, portable
  │     v[k]    = v[k+1];    │
  │     v[k+1]  = temp;      │
  │   }                      │
  └──────────────┬───────────┘
                 │  compiler (clang, gcc, rustc)
                 ▼
  ┌──────────────────────────┐
  │ Assembly (ARMv8):        │
  │   LSL  X10, X1, 3        │   ← still text, but now
  │   ADD  X10, X0, X10      │     one-to-one with hardware ops
  │   LDUR X9,  [X10, 0]     │
  │   LDUR X11, [X10, 8]     │
  │   STUR X11, [X10, 0]     │
  │   STUR X9,  [X10, 8]     │
  │   BR   X10               │
  └──────────────┬───────────┘
                 │  assembler (as, llvm-mc)
                 ▼
  ┌──────────────────────────┐
  │ Machine code (binary):   │
  │   00000000101000100000…  │   ← bit patterns the CPU
  │   00000000100000100001…  │     directly fetches & decodes
  │   10001101111000100000…  │
  └──────────────────────────┘
```

**Three things the compiler buys you (and one it doesn't):**

| Property | What you gain | What it costs |
|---|---|---|
| Natural notation | `A + B` instead of `ADD A,B` plus address arithmetic | The compiler has to be correct — every miscompilation is silent |
| Productivity | Fewer lines per idea; domain languages (Fortran/scientific, Cobol/business, Lisp/symbolic) | You don't see the actual instructions the CPU runs |
| **Portability** | Same source, different ISA = recompile, not rewrite | You inherit the compiler's view of "what's fast" |
| Performance | *not* automatic — modern compilers approach hand-tuned asm only with `-O2`/`-O3` and PGO | Compilation time, debug complexity |

**The Big Picture: ISA as the load-bearing interface.**

```
┌─────────────────────────────────────────┐
│  Application (your program)             │
├─────────────────────────────────────────┤
│  Libraries, runtime, OS userspace       │
│  ─── ABI boundary (calling conv,        │
│       syscall numbers, ELF layout) ───  │
│  OS kernel                              │
├─────────────────────────────────────────┤  ← THE ISA LINE
│  ISA: instructions, registers,          │
│       memory model, I/O, exceptions     │
├─────────────────────────────────────────┤
│  Microarchitecture (Apple M1, Graviton, │
│   Cortex-A53) — pipeline, caches,       │
│   branch predictor, all hidden          │
└─────────────────────────────────────────┘
```

The **ISA is the only interface in this stack that has both an "above" and a "below" maintained by *different organizations on different schedules*.** Anything above it (compilers, OS, apps) treats the ISA as a fixed contract. Anything below it (the silicon team) can change wildly — Apple's M1 has 8 decode lanes and 600+ in-flight instructions, the Cortex-M0 has none of that — and the *same binary* runs on both because both honor ARMv8-A.

### If you were the CPU designer…

You're told: "ship a chip that runs every existing ARMv8 binary 30% faster, no recompilation allowed." What can you change, and what can't you?

You **can** change anything below the ISA line: add more pipeline stages, wider issue, smarter branch prediction, larger caches, new physical register file. You **can't** change anything above: not the instruction encodings, not the register count, not the memory ordering model, not the exception behavior. This asymmetry is exactly why ISAs evolve so slowly (ARMv7 → ARMv8 took ~20 years) while microarchitectures churn yearly. The ISA is *load-bearing for the world's software*; the microarchitecture is load-bearing for nothing outside the chip. **Conservatism at the ISA, aggression at the µarch** — that's the whole post-1964 industry strategy in one sentence.

### Cross-language view

```c
// C — compiles directly to ARMv8 in the chapter's Fig 1.4
void swap(int *v, int k) { int t=v[k]; v[k]=v[k+1]; v[k+1]=t; }
```
```rust
// Rust — same machine code shape, but the *type system* forbids
// the bounds violations C silently allows
fn swap(v: &mut [i32], k: usize) { v.swap(k, k+1); }
```
```go
// Go — slices carry length, so the runtime injects a bounds check
// (one extra compare-and-branch in the emitted ARM asm)
func swap(v []int, k int) { v[k], v[k+1] = v[k+1], v[k] }
```

What the stdlib actually does: `clang -O2` on the C version emits exactly the six instructions in Figure 1.4 with no prologue. Rust's `[T]::swap` lowers to the same after monomorphization with `--release`. Go's `swap` adds ~3 instructions per index access for the bounds check; `-gcflags="-B"` removes them but is rarely used. **The HLL choice changes what gets emitted, not what the ISA accepts.**

### Where this shows up in real systems

- **Apple Rosetta 2 (2020).** When Apple moved Macs from x86-64 to ARM64, Rosetta translated x86-64 binaries to ARM64 ahead-of-time. It worked because the ISA contract is *complete* — Rosetta only needed to honor x86's documented semantics, not Intel's microarchitectural quirks. Software running through Rosetta hit ~70% of native speed without source.
- **AWS Graviton vs. Intel.** A Java service compiled to JVM bytecode is doubly-abstracted: bytecode → ARM via HotSpot JIT, or bytecode → x86 via the same JIT. The ISA difference is invisible to the application; the cost is paid once at JIT time. This is why AWS could announce 40% better price/performance on Graviton without asking customers to rewrite anything.
- **`objdump -d` and Compiler Explorer (godbolt.org).** Every time a senior engineer pastes C into godbolt.org and stares at the assembly, they're physically performing the chapter's Figure 1.4 in reverse — using the assembler-disassembler symmetry to verify the compiler did what they wanted. This loop is the day-to-day work of perf engineers.
- **Spectre/Meltdown and the ISA's limits.** The ISA contract specifies *architectural* state (register values, memory contents). It does **not** specify microarchitectural state (cache lines, branch predictor entries). Spectre exploits exactly this gap — leaking via the unspecified layer. The fix is hard precisely because the ISA never promised anything about it.

### Diagnostic questions

1. **Q:** Why is the assembler still a separate program from the compiler in most toolchains, given that modern compilers can emit binary directly?
   *Wrong-answer trap:* "Historical accident." The real reason: assembly is the **debuggable, human-readable artifact** that lets you inspect what the compiler decided. Skipping it means you can't read `-S` output, can't write inline asm, can't run `objdump`. Some compilers (Go's gc) *do* emit binary directly and the lack of stable text assembly has been a persistent pain point.

2. **Q:** A program compiled for ARMv7 won't run on an ARMv8-only chip. Why isn't this a violation of ISA-as-portable-contract?
   *Wrong-answer trap:* "Because ARMv8 is a new ISA." ARMv8-A *includes* ARMv7 as the AArch32 mode — a v7 binary can run on a v8 chip that enables it. The actual failure mode is **AArch64-only chips** (Apple M-series after M1, recent Graviton): the chip vendor *dropped* the v7 contract. The contract is portable only across implementations that *honor it*.

3. **Q:** Why does Patterson call cache memory "another type of memory" rather than "part of the ISA"?
   *Wrong-answer trap:* "Because it's small." Caches are deliberately **outside the ISA** — the architectural state doesn't mention them. This is why CPU vendors can change cache sizes, levels, and policies every generation without breaking software. The cost: cache-side-channel attacks exploit exactly this *unspecified* layer.

4. **Q:** If int is 32 bits on a 32-bit CPU and 64 bits on a 64-bit CPU (per LGO p.41), how is that compatible with "binary portability"?
   *Wrong-answer trap:* "It's not — that's why we have explicit int32." Binary portability holds **within one ABI**. The ARM64 SysV ABI defines `int = 32 bits` regardless of word size — what changes is `long` and pointers. A binary compiled against one ABI is portable to all chips honoring that ABI; cross-ABI requires recompilation.

5. **Q:** Why does flash memory wear out (100K–1M writes) while DRAM doesn't?
   *Wrong-answer trap:* "Flash is slower." Flash stores bits as **trapped electrons in a floating gate**; each write tunnels electrons through an oxide layer that physically degrades. DRAM stores bits as charge on a capacitor that's refreshed every ~64ms — no oxide breakdown. This is also why SSDs need **wear-leveling firmware**: the FTL re-maps logical blocks to physical cells to spread the damage.

### See also
- [cod-notes.md](Notes/cod-notes.md) 2026-05-13 — *Eight Great Ideas* — Abstraction (Idea #2) is precisely the principle this entry instantiates; the ISA Big Picture is the canonical example.
- [cod-notes.md](Notes/cod-notes.md) 2026-05-11 — *Hardware Pipelining* — pipelining lives entirely *below the ISA line*; this entry explains why that's possible.
- [n2t-notes.md](Notes/n2t-notes.md) — Nand2Tetris's Hack ISA is the same kind of contract at toy scale; building its assembler in Project 6 makes the compiler/assembler split viscerally clear.
- [lgo-notes.md](Notes/lgo-notes.md) — Go's `int` sizing rules (p.41) are a concrete ABI consequence of this chapter's principles.

---

## [2026-05-13] The Eight Great Ideas — Computer Architecture's Periodic Table · pp.28–38 · Ch.1 §1.1–1.2

### TL;DR
Patterson & Hennessy distill 60 years of computer-architecture practice into **eight recurring ideas** — Moore's Law, Abstraction, Make-the-Common-Case-Fast, Parallelism, Pipelining, Prediction, Memory Hierarchy, and Dependability via Redundancy — and they treat these like a periodic table: every later chapter is labeled with the icons of whichever ideas it instantiates. The list is curated and *intentionally short* (eight, not eighty), which is the real pedagogical move: it forces the student to see Intel's Tomasulo algorithm, ARM's branch predictor, and DRAM caches as **three faces of the same idea (prediction, or hierarchy)**, not as three unrelated tricks. Mastering COD is largely learning to recognize which of these eight is on the table at any moment.

### History — "why does this exist?"
The "great ideas" framing didn't exist in the 1990 *Computer Architecture: A Quantitative Approach* — that book argued *quantitatively* from benchmarks. By the 2008/2014 editions of COD, Patterson had spent a decade watching students drown in micro-architectural detail and had reorganized the undergraduate book around a smaller, more memorizable spine. The list is post-hoc induction over the history of the field: every idea on it was either invented or fully realized between **1945 (the von Neumann report) and 1985 (the original RISC papers)**, and every CPU built since has been a recombination of these eight. The 2016 ARM edition kept the list identical to the 2014 edition — Patterson's claim is that the periodic table is **complete enough that no new element has needed adding in 30 years**.

### Intuition — "this is like…"
The eight great ideas are to computer architecture what **the SOLID principles are to OOP, or the 12-factor app rules are to cloud services**: a curated minimum that lets practitioners argue about specific designs at the right level of abstraction. You don't *derive* a CPU from them, the way you don't derive a class hierarchy from SOLID — but if someone proposes a design that violates one (e.g., "let's redesign without a memory hierarchy"), the burden of proof is on them, not on you.

### Mechanics

**The eight, with what each is really claiming:**

| # | Idea | Icon | The real claim | Where it lives in the rest of COD |
|---|------|------|----------------|----------------------------------|
| 1 | **Design for Moore's Law** | "up-and-right" arrow | Resources double every 18–24 mo, so design for *next-year's* transistor budget — never *today's* | Ch.5 (cache sizes growing); Ch.6 (core counts) |
| 2 | **Abstraction** | abstract painting | Hide lower levels behind a contract so you can change either side independently | Ch.2 (ISA); Ch.4 (microarchitecture vs ISA); Ch.5 (virtual memory) |
| 3 | **Make the common case fast** | sports car | Optimize the 90% path; tolerate slow rare paths | Ch.4 (branch prediction); Ch.5 (caches); §1.6 (Amdahl's law) |
| 4 | **Performance via parallelism** | 4 jet engines | Do N things at once when serial is too slow | Ch.6 (multicore, SIMD, GPU) |
| 5 | **Performance via pipelining** | sequence of pipe segments | A *special* parallelism — overlap stages of the same task, bucket-brigade style | Ch.4 (the entire chapter) |
| 6 | **Performance via prediction** | crystal ball | Guess and start work; rollback cheaper than waiting | Ch.4 (branch prediction, speculative execution) |
| 7 | **Hierarchy of memories** | layered triangle | Stack memories of different speed/size/cost so the *common* access is fast and *all* data fits | Ch.5 (the entire chapter) |
| 8 | **Dependability via redundancy** | dual-tire truck | Add spare components so a single failure doesn't kill the system | Ch.5 (ECC, RAID); Ch.6 (distributed systems) |

**The structural punch line — three groups, not eight:**

```
                ┌──────────────────────────────────┐
                │   1. Moore's Law  (the driver)   │
                └──────────────────────────────────┘
                              │
            ┌─────────────────┴─────────────────┐
            │                                   │
   ┌────────▼────────┐               ┌──────────▼─────────┐
   │ 2. Abstraction  │               │   PERFORMANCE      │
   │ (manage growth) │               │ 3. Common case     │
   └─────────────────┘               │ 4. Parallelism     │
                                     │ 5. Pipelining      │
                                     │ 6. Prediction      │
                                     │ 7. Memory hierarchy│
                                     └──────────┬─────────┘
                                                │
                                     ┌──────────▼─────────┐
                                     │ 8. Dependability   │
                                     │  (don't let speed  │
                                     │   kill correctness)│
                                     └────────────────────┘
```

One driver (Moore), one productivity weapon (Abstraction), five performance plays (Common-Case, Parallelism, Pipelining, Prediction, Hierarchy), and one safety net (Redundancy). Once you see this grouping, you stop memorizing eight things and start carrying **three questions** into any architecture discussion:

1. What's the abstraction here, and where's it leaking?
2. Which performance idea (or combination) is being applied?
3. What does the redundancy story look like when a transistor flips?

**Why "the common case fast" is the deepest of the eight.**
Ideas 4–7 are *techniques*. Idea 3 is the **judgment** that decides when to apply them. It also implies §1.6's measurement discipline: you cannot make the common case fast unless you know what the common case *is* — which requires benchmarks, profiling, and humility about your priors. Every junior engineer who optimizes the wrong loop is violating Idea 3, not Idea 4.

### If you were the architect of a brand-new ISA in 2026…

You're staring at a clean sheet (say, you're a startup designing a domain-specific accelerator for transformer inference). Which of the eight bind you, and which don't?

The textbook's answer: **all eight still bind you, but their weights have shifted.** Pipelining (Idea 5) and prediction (Idea 6) matter less for a throughput-oriented accelerator with predictable dataflow — branchless GEMM kernels don't need a 4-stage branch predictor. **Parallelism (4) and hierarchy (7) dominate**: the entire NVIDIA H100 / Apple Neural Engine design space is "how much parallelism can we land, and how aggressively can we tier the SRAM/HBM/DRAM hierarchy." Dependability (8) gets pushed *up* into the system level (replicas, checkpointing) rather than the chip level. So the eight aren't static weights — they're a *checklist* whose relative importance is itself a design output.

### Cross-language view
*(n/a — architectural-themes entry, not code.)*

### Where this shows up in real systems

- **Apple M-series memory hierarchy** — Idea 7 in extremis: registers → L1 (192 KB/core on M3) → L2 (per-cluster shared) → **system-level cache** (the unusual one — SLC sits between L2 and DRAM and is shared across CPU + GPU + Neural Engine) → unified DRAM. Five tiers, not three. Apple cashes in Idea 7 harder than Intel because their workloads (Metal compute + AVX-equivalent + neural inference) all share the same data.
- **Speculative-execution security flaws (Spectre/Meltdown, 2018)** — the moment Ideas 6 (prediction) and 2 (abstraction) collide. Prediction speculatively reads memory; the ISA's abstraction (privilege levels) promises you can't see kernel memory; the cache (Idea 7) leaks the speculatively-read value as a timing signal. A textbook lesson that great ideas have **interaction effects** the simple list doesn't show.
- **Hyperscaler dependability (Idea 8) is now mostly software** — Google's Borg, Kubernetes, and Spanner treat individual servers as disposable, and the redundancy lives at the cluster level. The single-machine ECC + RAID story COD teaches still matters inside the rack, but the *operative* redundancy at Google scale is replication across racks/regions. Idea 8 has migrated up the stack.

### Diagnostic questions

1. **Q:** Why is "make the common case fast" listed separately from "performance via parallelism"? Isn't parallelism just *one way* of making the common case fast?
   *Wrong-answer trap:* "They're redundant." They aren't: Idea 3 is a **decision rule** (focus optimization effort here), Idea 4 is a **mechanism** (do things concurrently). You can violate 3 while applying 4 — e.g., parallelizing a path your profiler shows is taken 0.1% of the time.

2. **Q:** Memory hierarchy (7) and prediction (6) are sometimes called "the two great lies of computer architecture." What's being lied about?
   *Wrong-answer trap:* "Latency." It's not just latency — it's the **single-cycle, sequential-execution programming model** the ISA promises. The CPU does *not* fetch from one giant flat memory, and it does *not* execute one instruction at a time; the hierarchy and the predictor maintain the *illusion* that it does. Most performance bugs are the illusion leaking.

3. **Q:** Where does the eighth idea (dependability via redundancy) buy you protection — at the gate level, the chip level, or the system level?
   *Wrong-answer trap:* "All three, equally." Historically it lived at the gate level (TMR triple-modular redundancy in 1960s mainframes), migrated to the chip level (ECC RAM, RAID), and is now dominantly at the system level (replication, consensus). Knowing which level a given system relies on tells you which failure modes it actually tolerates.

4. **Q:** Suppose someone proposes a CPU design that has *no* branch predictor (Idea 6). What's the steel-man case?
   *Wrong-answer trap:* "There is no case." There is one: **deterministic real-time systems** (e.g., automotive safety controllers, hard-real-time DSPs). Branch prediction makes worst-case execution time unanalyzable; dropping it trades average throughput for tight latency bounds. The list of eight isn't "always apply all eight" — it's "know which you're using and which you're refusing, and why."

### See also

- [cod-notes.md](Notes/cod-notes.md) 2026-05-12 — *ARMv7 → ARMv8 → LEGv8* — the AArch64 cleanup is itself an instance of Idea 2 (abstraction) cashing in on a generation boundary.
- [cod-notes.md](Notes/cod-notes.md) 2026-05-11 — *The Hardware/Software Interface as a Discipline* — that entry's "six-chapter spine" maps roughly 1-to-1 onto Ideas 2/5/7/4 in this entry's table.
- DDIA Ch.5 (Replication) — Idea 8 in its modern, distributed-systems incarnation.
- OSTEP Ch. on virtual memory — Idea 2 (abstraction) and Idea 7 (hierarchy) running simultaneously; reading them as instances of the great ideas makes VM feel less arbitrary.

---

## [2026-05-12] ARMv7 → ARMv8 → LEGv8: An ISA Reform Case Study · pp.17–27 · Preface

### TL;DR
ARMv7 was a 32-bit RISC encrusted with two decades of features that made every instruction harder to decode (per-instruction predication, the PC living in the register file, load-multiple). When ARM moved to 64 bits in 2011, they used the discontinuity to amputate those features — and the result (AArch64) looked far more like 1985's MIPS than like its own predecessor. COD's authors define **LEGv8**, a ~50-instruction teaching subset of ARMv8, because that subset is genuinely cleaner than ARMv7 ever was — the pedagogical and the production ISA finally agree on what's worth teaching.

### History — "why does this exist?"
ARM started in 1985 at Acorn Computers as a clean RISC; by ARMv5 (~1999) the spec had absorbed Thumb (16-bit encodings), Jazelle (Java bytecode in hardware), per-instruction conditional execution, and load/store-multiple — features added to win specific design wins (mobile code density, set-top boxes, embedded). By 2010 ARMv7 was the world's most-shipped ISA *and* one of the most baroque. The **ARMv8-A specification (2011, first silicon Apple A7 in 2013)** treated the 32→64-bit jump as a once-per-generation opportunity to break compatibility, dropping the legacy cruft into a separate "AArch32" execution mode that AArch64 cores aren't required to implement. By 2016, when this edition of COD was written, **14 billion ARM cores shipped in 2015 alone**, and the authors could no longer justify teaching MIPS to students whose phones, watches, and laptops would run ARM.

### Intuition — "this is like…"
ARMv7 → ARMv8 is the ISA-design equivalent of a **major-version language break** — Python 2 → 3, or Perl 5 → 6. You only get to do it every 20 years, you pay a migration tax, and the only justification is that the new version drops things the old version couldn't drop without breaking the world. ARM cashed in their once-per-generation cleanup budget, and the textbook authors immediately said "thank you" and switched.

### Mechanics

**The seven amputations** — what ARMv8 dropped from ARMv7 (COD p.xvii):

| Feature ARMv7 had | What it cost | Why ARMv8 dropped it |
|---|---|---|
| **16 general-purpose registers** (R0–R15) | High register pressure; lots of spills | Expanded to **32 GPRs** — matches MIPS/RISC-V |
| **PC is one of the registers (R15)** | Every `ADD` could be a branch; decode logic must check for PC writes | PC is now its own thing; `BR Xn` is the only branch-via-register |
| **Per-instruction conditional execution** (`ADDEQ`, `MOVNE`…) | Every instruction carries a 4-bit condition; predication blocks pipeline optimizations | Dropped; only `B.cond` and the new `CSEL` (conditional-select) remain |
| **`LDM`/`STM`** (load/store multiple registers in one instruction) | Variable-length micro-op expansion; interrupt-state nightmare | Replaced with **`LDP`/`STP`** (load/store *pair*, fixed) |
| **Short PC-relative branches** (`±32 MB`) | Trampolines & linker veneers in large binaries | New `B`/`BL` reach `±128 MB`; ADR-pair reaches anywhere |
| **Inconsistent addressing modes** across load/store | Decode tables per opcode | One unified addressing model — every load/store has the same modes |
| **Many instructions set condition flags by default** | Implicit flag dependencies stall the pipeline | Flag-setting is now opt-in (`ADDS` vs `ADD`) |

Each line on that list is a feature that *seemed clever* in 1995 (when transistor budgets were tight and code density mattered) and *seemed expensive* in 2010 (when out-of-order pipelines and wide superscalar decoders made every irregularity costly).

**The LEGv8 reduction** — what COD strips out of ARMv8 to teach:

```
ARMv8-A full reference manual ......................... 5,400 pages
ARMv8 instruction count ............................... ~1,000+
LEGv8 instructions covered in Chapters 2–4 ............ ~50
LEGv8 "Real Stuff" survey sections (Ch.2, 3, 5) ....... 100+ instructions
                                                       (cultural literacy,
                                                        not testable)
```

LEGv8 is to ARMv8 what RISC-V's RV64I base ISA is to RV64GC: a teaching/research-friendly subset where every instruction earns its keep by demonstrating a concept. Crucially, LEGv8 is a *strict subset* — code that runs on LEGv8 runs on every ARMv8 core in the world. The reverse is not true, and the chapters labelled "Real Stuff" are honest about the gap.

**The hardware-vs-software reading fork** the preface introduces (p.xviii table) also tells you something about ISA design: the chapters most affected by the MIPS→LEGv8 switch are exactly Ch.2 (instructions), Ch.3 (arithmetic), the VM section of Ch.5, and the small VMIPS example in Ch.6. The pipeline (Ch.4), the cache hierarchy (Ch.5 sans VM), and the parallelism material (Ch.6) are essentially ISA-agnostic — which is itself the punch line of RISC: **once your ISA is regular, the implementation is a separable concern**.

### If you were the ARM architect in 2008…

You're staring at the ARMv7 instruction encoding and you have a once-in-a-generation opportunity. Do you (a) extend cleanly to 64 bits, preserving binary compatibility, or (b) start over and break ABI?

The ARMv8 answer: **start over for AArch64, but keep AArch32 as an execution mode the hardware optionally supports**. This is Great Idea #2 (abstraction to simplify design) at the ISA level — by making the new mode and the legacy mode formally separate, you stop paying the legacy tax on every decode in the new mode. Apple's M-series chips eventually went further and **dropped AArch32 entirely** from their cores (M1 onward) — the first commercially significant chip family to be AArch64-only. The cleanup is finally cashing in across the industry.

### Cross-language view
*(n/a — this is an ISA-design entry. Code-level cross-language views will come when Ch.2 introduces specific LEGv8 instructions.)*

### Where this shows up in real systems

- **Apple Silicon's wide decoder.** The M1/M2/M3 cores famously decode 8 instructions per cycle while Intel/AMD struggle past 4–6. The reason is exactly the amputation list above: every ARMv8 instruction is 32 bits, regular, with no predication and no implicit flags. Decode width is what the cleanup *bought*.
- **RISC-V's pitch.** RISC-V (started ~2010 at Berkeley by Patterson's students) is essentially the same argument run a second time — even ARMv8 still carries some baggage (NEON, complex addressing modes), so let's start over from scratch and license the result for free. The book's author Patterson is also RISC-V's most prominent advocate; the LEGv8 / RISC-V resemblance is not coincidental.
- **Compatibility-mode silicon costs.** Qualcomm, Samsung, and MediaTek cores still implement AArch32 because Android shipped a long tail of 32-bit-only apps. Each generation's silicon dedicates non-trivial area to a mode 99% of new code never touches. The cleanup isn't done in the wild even though the *spec* finished it a decade ago.

### Diagnostic questions

1. **Q:** Why is "PC in the register file" a problem in a pipelined CPU?
   *Wrong-answer trap:* "Because branches are slow." It's worse than that — **any** `ADD R15, ...` is a branch in ARMv7, so the decode stage must speculate on every register-writing instruction. That's a per-instruction tax to handle a rare case.

2. **Q:** Predicated execution (`ADDEQ`) was designed to *avoid* branches. Why did ARMv8 drop it?
   *Wrong-answer trap:* "Because branches got cheap." Branch *prediction* got near-perfect; predication only saved cycles when the branch was unpredictable, and in those rare cases `CSEL` (conditional select) gives you the same benefit with one specific instruction instead of polluting every instruction's encoding with a 4-bit condition.

3. **Q:** Why does LEGv8 omit the "Real Stuff" instructions but still cover them in survey sections?
   *Wrong-answer trap:* "To avoid scaring students." It's a *split contract*: students need a tractable subset to *understand* (the ~50 LEGv8 ops), but a literate engineer should *recognize* the surrounding hundreds when they show up in disassembly. The survey sections are reading-comprehension training, not skill-building.

4. **Q:** If you only learned LEGv8, what's the single biggest concept you'd miss from real ARMv8?
   *Wrong-answer trap:* "SIMD." It's **system registers and exception levels (EL0–EL3)** — LEGv8 stays in user mode. Most of the OS-kernel/hypervisor/secure-monitor split that defines real hardware is invisible from LEGv8; OSTEP and DDIA's storage-layer chapters will surface this gap.

### See also

- [cod-notes.md](Notes/cod-notes.md) 2026-05-11 — *The Hardware/Software Interface as a Discipline* (pp.6–16) — the broader scaffolding entry; the LEGv8 idea was introduced there in passing and is fully unpacked here.
- N2T Chapter 4 (Machine Language) — designing the Hack ISA from scratch is the experiential version of this entry; you feel why regularity matters when you have to write the decoder.
- RISC-V's official ISA manual frames every design choice as an explicit *response* to ARMv7-style baggage — reading its preface back-to-back with COD's preface is a free architecture seminar.

---

## [2026-05-11] The Hardware/Software Interface as a Discipline · pp.6–16 · Front Matter & TOC

### TL;DR
*Computer Organization and Design* is organized around a single thesis: every software engineer — not just hardware designers — needs to understand the abstractions between transistors and `printf`, because the post-2005 multicore world makes that abstraction leaky. The book's table of contents is itself the discipline's map: six chapters that walk from "what is a computer" down through instructions, arithmetic, the pipelined processor, the memory hierarchy, and finally parallelism. This entry is a scaffolding note — the chapter graph that every subsequent COD entry will hang off of.

### History — "why does this exist?"
Patterson and Hennessy wrote the original (MIPS) edition in 1989 as a deliberate split from their graduate-level *Computer Architecture: A Quantitative Approach* (1990) — that book was for future CPU designers, and they realized compilers, OS people, and database people needed a different, less ceremonial book. The ARMv8 edition (2016) replaced MIPS with a stripped-down ARMv8 subset they call **LEGv8**, chosen because ARM had finally — in moving to 64-bit — dropped enough baroque features (per-instruction predication, PC-in-register-file, load/store-multiple) that it began to resemble MIPS. The replacement matters because **14 billion ARM cores shipped in 2015**; teaching MIPS in 2016 was teaching a fossil while every phone in the room ran ARM.

### Intuition — "this is like…"
Think of computing as a stack of translators in a tall building. The top floor speaks Python; the basement speaks voltages on a wire. Every floor in between — bytecode, assembly, machine code, microarchitecture, gates, transistors — translates "I want to add two numbers" into something the floor below can act on. COD is the elevator tour: it doesn't make you live in the basement, but it shows you what's actually being done in your name on every floor you don't see.

### Mechanics

**The six-chapter spine of the discipline:**

```
 ┌────────────────────────────────────────────────────────────────┐
 │ Ch.1  Abstractions & Technology   ← Why hardware matters       │
 │         · Eight Great Ideas                                    │
 │         · Performance, Power Wall, multicore turning point     │
 ├────────────────────────────────────────────────────────────────┤
 │ Ch.2  Instructions (LEGv8)        ← The contract: ISA          │
 │         · Operations, operands, addressing modes               │
 │         · How a function call becomes bits                     │
 ├────────────────────────────────────────────────────────────────┤
 │ Ch.3  Arithmetic                  ← What ALUs actually do      │
 │         · Two's complement, multiply, divide, IEEE-754         │
 │         · Subword parallelism (SIMD)                           │
 ├────────────────────────────────────────────────────────────────┤
 │ Ch.4  The Processor               ← The 5-stage pipeline       │
 │         · Datapath, control, hazards, forwarding               │
 │         · ILP, exceptions, real Cortex-A53 / Core i7 pipelines │
 ├────────────────────────────────────────────────────────────────┤
 │ Ch.5  Memory Hierarchy            ← Caches, TLB, virtual mem   │
 │         · Locality, miss penalty, coherence, RAID              │
 ├────────────────────────────────────────────────────────────────┤
 │ Ch.6  Parallel Processors         ← From core to warehouse     │
 │         · SIMD/MIMD/SPMD, GPUs, clusters, WSCs                 │
 └────────────────────────────────────────────────────────────────┘
```

**The book's central pedagogical bet — a running matrix-multiply example.** Patterson/Hennessy thread one example through chapters 3-6, compounding speedups so the reader can *feel* the stack:

```
Baseline matrix multiply in C ............................ 1.0×
+ Ch.3 subword parallelism (SIMD intrinsics) ............. ~4×
+ Ch.4 loop unrolling (instruction-level parallelism) .... ~2× more → ~8×
+ Ch.5 cache blocking (memory-hierarchy awareness) ....... ~2× more → ~16×
+ Ch.6 thread-level parallelism (16-core OpenMP) ......... ~14× more → ~200×

Total added C code: 24 lines.
```

That last fact is the entire reason the book exists: **24 lines of code, written by someone who understands the hardware, beat a naive implementation by ~200×** — on the same silicon. The authors are arguing that abstraction-blindness has a price, and that price is your throughput.

**The Eight Great Ideas (introduced in §1.2, the destination of this scaffolding entry).** Per the preface, every concept in the book maps back to one of eight recurring themes, with margin icons:

1. Design for Moore's Law (anticipate the silicon you'll have, not the silicon you have)
2. Use abstraction to simplify design
3. Make the common case fast
4. **Performance via parallelism**
5. **Performance via pipelining**
6. **Performance via prediction**
7. Hierarchy of memories
8. Dependability via redundancy

The first three are general engineering wisdom; the next three (4-6) are the trio the authors call the most cited in the book — together they describe how a single CPU core in 2026 squeezes ~10 instructions per cycle out of a clock that's barely faster than 2005's. Hierarchy (7) is why your cache misses dominate your wall-clock time. Dependability (8) is why ECC memory exists.

**Hardware-vs-software reading paths.** The preface gives an explicit fork — software-focused readers can skip the Verilog appendices (4.13, 5.12) and skim the logic-design appendix (A); hardware-focused readers should read those carefully and read Ch.2 as review. This note treats both paths as equally valid — but the cross-references in every future COD entry will lean software-side, since that's where most of the book's "you'll never need this until suddenly you do" payoff lives.

### If you were the textbook author…

You inherited a successful MIPS edition. ARM offers you 14 billion-units-of-relevance but also a 5,400-page reference manual. Do you switch?

The Patterson/Hennessy answer is **switch, but lie about it** — define a subset (LEGv8) that omits ARMv8's true complexity, and treat the omitted parts as "Real Stuff" survey sections in chapters 2, 3, and 5. The pedagogical contract is preserved (a clean RISC-style ISA), the marketing problem is solved (the cover says ARM), and the few places where ARMv8's quirks leak in get isolated to clearly-labeled real-world sections. This is itself an instance of Great Idea #2 (abstraction to simplify design) applied to *teaching* the discipline.

### Cross-language view
*(n/a — this is a meta/structural entry. Cross-language code patterns will appear when individual concepts are covered: e.g. how Rust's `[T; N]` array vs Go's slice vs C's pointer-arithmetic map to Chapter 2's addressing modes, or how Python's GIL relates to Chapter 6's shared-memory multiprocessing.)*

### Where this shows up in real systems

- **The "fast Python" stack — NumPy, PyTorch, JAX.** Each of these libraries' speedups over pure Python is a tour through COD's table of contents in order: NumPy's vectorized inner loops use Ch.3 SIMD, PyTorch's `torch.compile` uses Ch.4 ILP/fusion, JAX's `jit` plus device placement covers Ch.5 (memory layout for cache locality) and Ch.6 (multi-GPU). A user who can read COD's six chapters can read these libraries' performance docs without translation.
- **Apple Silicon's success.** The M-series chips' wins are textbook COD: a wide superscalar pipeline (Ch.4), aggressive caches and unified memory (Ch.5), and SVE2-style vector units (Ch.3). The fact that ARMv8 — the book's ISA — now defines the high-performance laptop market makes the 2016 ISA switch look prescient.
- **The "you can't ignore the hardware anymore" thesis in practice.** Every database engineer tuning a hot loop, every ML engineer fighting for tokens/sec, every game-engine programmer optimizing a cache line — they are all working inside the abstractions this book makes legible. The book's claim that "for at least the next decade, most programmers are going to have to understand the hardware/software interface" was written in 2016 and is more true in 2026 than it was then.

### Diagnostic questions

1. **Q:** Why is Chapter 4 (The Processor) considered the book's centerpiece by both software-focused and hardware-focused readers?
   *Wrong-answer trap:* "Because the CPU is the most important component." It's because pipelining and hazards are where every other chapter's ideas (parallelism, prediction, memory) converge — the pipeline is the *integration test* of the discipline.

2. **Q:** Why did the authors define LEGv8 instead of just teaching ARMv8?
   *Wrong-answer trap:* "Because students aren't ready for production ISAs." It's a pedagogical-complexity argument, not a capability one: ARMv8 has ~1000+ instructions; teaching all of them obscures the ~50 that demonstrate every architectural idea. The "Real Stuff" sections cover the rest as cultural literacy.

3. **Q:** Three of the Eight Great Ideas — parallelism, pipelining, prediction — appear constantly. What's the common pattern?
   *Wrong-answer trap:* "They all make things faster." More specifically: all three break the apparent sequential semantics of a program by doing work *speculatively or out of order*, while preserving the illusion that the program executed top-to-bottom. Every modern performance bug (Spectre, cache-timing leaks, false sharing) is debt owed by this illusion.

4. **Q:** If you only had time to read four chapters, which four and in what order?
   *Wrong-answer trap:* "1, 2, 3, 4 — front to back." Authors' own preface recommends **1 → 4 → 5 → 6** for the software-focused: get the framing (1), then the pipeline (4), then memory (5), then parallelism (6). Skip arithmetic and ISA details until you need them.

### See also

- [cod-notes.md](Notes/cod-notes.md) 2026-05-11 — *Pipelining: The Big Idea* (Ch.4 §4.5) — the canonical entry style and the chapter this scaffolding points toward as the book's centerpiece.
- N2T Chapter 4 (Machine Language) is COD Chapter 2 in miniature — designing your own ISA before reading about LEGv8 makes the trade-offs in §2.1-2.5 visceral.
- OSTEP's intro chapters lean on COD Chapter 1's abstraction hierarchy — when OSTEP says "the OS virtualizes the CPU," it's virtualizing exactly the LEGv8 machine COD builds.

---

## [2026-05-11] Hardware Pipelining — The Big Idea · pp.367–380 · Ch.4 §4.5

### TL;DR
Pipelining overlaps the execution of multiple instructions so that the CPU finishes one new instruction per cycle in steady state, even though each instruction still takes several cycles end-to-end. It improves **throughput** (instructions/second), not **latency** (time for any single instruction). The win is bounded by the slowest stage and by hazards that force the pipeline to stall or flush.

### History — "why does this exist?"
Pre-1960s CPUs executed one instruction at a time end-to-end — fetch, decode, execute, repeat — wasting most of the silicon every cycle. **IBM's Stretch (7030) in 1961** first overlapped fetch with execution; the **IBM System/360 Model 91 (1967)** combined pipelining with Tomasulo's out-of-order scheduling. The technique stayed niche-mainframe until the **MIPS R2000 (1985)** made the clean 5-stage pipeline the textbook standard for RISC, which is why every modern CPU — and this textbook — uses it as the teaching model. Intel's Pentium 4 (2000) tried to push to 20+ stages and proved the depth limit, pivoting the industry back to balanced ~14-stage cores.

### Intuition — "this is like…"
A laundromat with **one washer, one dryer, one folding table, one closet**. Doing laundry sequentially (wash → dry → fold → put away → next load) wastes 75% of your machines at any moment. Pipelining says: **once the washer is free, start the next load**. After the pipeline fills, *every* time-slice finishes one full load even though any individual load still takes 4 time-slices end-to-end. You didn't make the washer faster — you stopped letting it sit idle. That is the entire idea. Everything else (hazards, forwarding, branch prediction) is paying down the debt that this idea takes on.

### Mechanics

**The 5-stage MIPS/ARM pipeline** — every instruction passes through:

```
 ┌─────┐   ┌─────┐   ┌─────┐   ┌─────┐   ┌─────┐
 │ IF  │ → │ ID  │ → │ EX  │ → │ MEM │ → │ WB  │
 └─────┘   └─────┘   └─────┘   └─────┘   └─────┘
   ↑         ↑         ↑         ↑         ↑
 fetch    decode    execute    memory   write-back
 instr.   + read    (ALU)     (load/    (result
 from     regs                store)    to reg)
 i-cache
```

**Single-cycle vs pipelined — the throughput shift:**

```
Single-cycle: 1 instruction per long cycle (clock = longest path through datapath)

 t=0 ─────[IF │ ID │ EX │ MEM │ WB ]─────►  instr 1 done at t=1
 t=1 ─────[IF │ ID │ EX │ MEM │ WB ]─────►  instr 2 done at t=2
                                                       (CPI=1, cycle = slow)

Pipelined: 1 instruction completes every short cycle, after fill

 t=1: [IF₁]
 t=2: [IF₂][ID₁]
 t=3: [IF₃][ID₂][EX₁]
 t=4: [IF₄][ID₃][EX₂][MEM₁]
 t=5: [IF₅][ID₄][EX₃][MEM₂][WB₁]   ← first instr retires
 t=6: [IF₆][ID₅][EX₄][MEM₃][WB₂]   ← steady state: 1 retire/cycle
```

**Speedup formula (idealized).** For N instructions through a K-stage pipeline:

```
              N × T_single                        K × N
Speedup =  ────────────────  ≈  K     as N → ∞   (since cycles = K + N - 1)
            (K + N - 1) × T_p
```

So a 5-stage pipeline approaches **5× throughput** of a single-cycle design with the same logic — **without changing the algorithm or the silicon's gate count**. The cost: the clock cycle is now bounded by the *slowest stage*, not the average. Imbalanced stages waste pipeline capacity.

**The trade-off you must internalize:**

```
┌────────────────────────────────────────────────────────────┐
│  Single-cycle:  LATENCY good, THROUGHPUT bad               │
│                 (clock = slow, but result in 1 cycle)      │
│                                                            │
│  Pipelined:     LATENCY same-or-worse, THROUGHPUT great    │
│                 (clock = fast, but result in 5 cycles —    │
│                  and you must pay register-file overhead   │
│                  between every stage)                      │
└────────────────────────────────────────────────────────────┘
```

A pipelined instruction *individually* takes longer wall-clock time than a single-cycle one, because each stage now ends with a latch write. You're trading per-instruction latency for the ability to run five instructions concurrently.

### If you were the CPU designer…

You're given a single-cycle datapath whose critical path is 800ps and asked to make it faster without changing the logic gates. You split the path into 5 stages of 160ps each plus a 20ps latch — clock cycle becomes 180ps. Question: did you make instructions faster?

**No — you made them slower** (each instruction now takes 5 × 180ps = 900ps end-to-end vs. 800ps before). But you can now have **five instructions in flight at once**, so instructions retire every 180ps instead of every 800ps. **Throughput went from 1.25 GIPS to 5.55 GIPS** while individual latency got *worse* by 12%. This is why pipelining is an architectural commitment, not a free lunch — and why server CPUs (throughput-bound) embrace deeper pipelines than embedded CPUs (latency-bound).

### Cross-language view
*(n/a — pipelining is a hardware concept. Software analogs exist — software pipelining in compilers, async/await event loops as cooperative pipelining — see the **Where this shows up** section below.)*

### Where this shows up in real systems

- **Spectre / Meltdown (2018).** Modern CPUs go far beyond 5-stage in-order pipelines into **speculative out-of-order execution** with branch prediction. Spectre exploits that mis-predicted speculative loads leave footprints in the cache — so an attacker can leak kernel memory by training the predictor. The bug is *not* in the architecture's correctness; it's in pipelining's microarchitectural side effects. Every cycle since 2018 has been paying down this debt with KPTI, retpolines, and IBRS.
- **GPU SIMT pipelines.** NVIDIA's SM (streaming multiprocessor) has a **deeply pipelined warp scheduler** that issues one instruction per cycle from a warp of 32 threads. When one warp stalls on memory, another fills the slot — pipelining at the warp level. Same idea as the laundromat, scaled to thousands of "loads".
- **Compiler-level software pipelining.** GCC's `-funroll-loops` and Polly's loop transformations rearrange loop bodies so iterations overlap — this is *software* pipelining of operations that the hardware then *also* pipelines. Two layers of the same idea.
- **Async/await as cooperative pipelining.** When you `await fetch(...)` inside a `for` loop without batching, you're running a single-cycle program. `Promise.all([...])` or Go's `errgroup` is the pipelined version: the dispatcher refills the slot while the previous request is in-flight on the network.

### Diagnostic questions

1. **Q:** Why does pipelining improve throughput but not (or even hurt) latency?
   *Wrong-answer trap:* "Because pipelining makes each stage faster." It doesn't — the gates are the same. The win is overlap, not stage speed.

2. **Q:** If we double the number of pipeline stages from 5 to 10, do we get 2× the throughput?
   *Wrong-answer trap:* "Yes, by the K factor in the speedup formula." Only if (a) stages stay balanced, (b) latch overhead stays small relative to stage delay, and (c) hazards don't multiply. In practice deeper pipelines hit the **clock-overhead wall** around stage 15–20 (the Pentium 4 lesson).

3. **Q:** A load instruction is followed immediately by an instruction that uses its result. What stage detects the problem?
   *Wrong-answer trap:* "WB stage" — by then the dependent instruction is already 4 stages into the pipeline. The hazard must be detected in **ID** (decode), which is the only stage that knows about register dependencies.

4. **Q:** Why is a single-cycle design's clock determined by the load instruction specifically?
   *Wrong-answer trap:* "Because loads access memory." Many instructions access memory. Loads are slowest because they use **five functional units in series**: I-mem → regs → ALU → D-mem → regs. The clock = sum of the worst path, not the worst single unit.

5. **Q:** In what specific scenario would pipelining *not* be worth it?
   *Wrong-answer trap:* "Simple ISAs." Wrong — simplicity helps pipelining. The right answer is **very short instruction sequences** (N ≪ K) where the fill-cost (K-1 cycles of bubbles) dominates the steady-state win. Embedded ISRs and bootloaders often run on shallow pipelines for this reason.

### See also
- **COD Ch.1** — the CPU performance equation (`Time = IC × CPI × cycle`). Pipelining drives CPI toward 1 *without* lengthening the cycle — that's its whole point in the equation.
- **OSTEP Ch.6 — Limited Direct Execution.** OS context switches flush the pipeline; same throughput-vs-latency tension at the software layer.
- **N2T Ch.5 — Computer Architecture.** The Hack CPU is a single-cycle design; building one shows you *viscerally* why the clock has to be slow.
- **CLRS — Amortized analysis.** The "fill cost" of pipelining is structurally identical to amortized cost arguments: high per-op cost amortized over many ops.
