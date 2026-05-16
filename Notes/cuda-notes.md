# CUDA Notes

_Entries follow the template at `Notes/TEMPLATE.md`. Append-only. **Newest entry at top**, immediately after this header._

---

## [2026-05-19] The Three-Phase Pedagogy, the Final-Project Apparatus, and What This Tells You About How To Learn CUDA · pp.14–17 · Front Matter — Preface §§ "How to Use" + Final Project + Online Supplements

### TL;DR
This chunk is the operational manual for the **ECE498AL course** the book was extracted from — and once you decode it, the Preface is *covertly* the most useful chapter in the book, because it tells you the **learning order Kirk & Hwu found actually works** after teaching CUDA to thousands of students. The pedagogy is split into **three phases**: (1) **One lecture, basic CUDA**, end-state = "students can write a naïve matmul kernel in a couple of hours" (Ch.3); (2) **Ten lectures, performance scaffolding** — memory model, threading model, hardware performance features, common parallel patterns (Chs.4–7); end-state = "10× speedup on the same matmul through tuning, plus assignments on convolution, vector reduction, prefix scan"; (3) **Remaining lectures, computational thinking + breadth** — broader execution models, parallel programming principles, case studies (Chs.8–11). The **final project apparatus** — mentoring, project workshops, design documents, clinics, reports, symposium — is six interlocking learning artifacts each calibrated to surface a different failure mode (scope too big/small, prior work missed, algorithm wrong for CUDA, performance baseline weak, presentation skills atrophied, individual contribution invisible). The chunk's deeper teaching is *meta*: **don't read this book straight through**. Phase 1's one chapter unlocks Phase 2's ten chapters, which unlock Phase 3's case studies. Write the naïve kernel in Ch.3 before you finish reading; come back, tune; come back, tune again. The book is structured as a **spiral curriculum** (Bruner, 1960) — same concepts revisited at increasing depth — and the Preface is the chapter that tells you so.

### History — "why does this exist?"
The course this book documents — **ECE498AL** at UIUC, since renamed **ECE408 ("Applied Parallel Programming")** — was taught for the first time in **Spring 2007**, mere months after CUDA 1.0 shipped (June 2007). David Kirk (then NVIDIA's Chief Scientist) commuted weekly to Urbana from California to co-teach with Wen-mei Hwu; the original cohort was **52 students under NDA** because CUDA itself was still pre-public. The Preface narrates this scrappy origin (p.12 from yesterday's read; this chunk continues the same operational story). By the time this book was published (2010), the course had been taught five times — three one-semester runs and two intensive one-week boot camps — and the three-phase structure documented here is the *result of those iterations*, not a guess. The **final-project apparatus** (workshop / design doc / clinic / report / symposium) is itself a borrowing from older engineering-school traditions — UIUC's **ECE senior design** course (which has run since the 1950s) uses an almost identical structure, and Kirk & Hwu adapted it for the rapid feedback the GPGPU world demanded. The bigger historical context: this is the **first university course on heterogeneous parallel programming**, period. Before 2007, parallel programming was taught for clusters (MPI, OpenMP) or for tiny embedded multicores; nobody had a curriculum for the *desktop-scale data-parallel* regime that GPUs opened. Modern courses — **Stanford's CS149 (Kayvon Fatahalian)**, **CMU's 15-418**, **Illinois's continued ECE408**, **UC Davis's ECS158** — all inherit the three-phase structure documented here, sometimes acknowledging it explicitly.

### Intuition — "this is like…"
The three-phase pedagogy is the **flying-lessons-versus-flight-simulator-versus-actual-flight-time** progression. **Phase 1** is "get them in the air *today*" — one lesson, takeoff and landing, no fancy maneuvers. The student must finish the day having flown a plane. **Phase 2** is "now learn what the dials and pedals actually do, why the engine behaves the way it does, what wind shear feels like, what stalls feel like" — the long middle where mechanical proficiency turns into operational competence. **Phase 3** is "fly a real mission with a real cargo, make real decisions about route and weather, deal with real consequences" — the case-study / final-project phase where judgment is the thing being trained. **A student who skips Phase 1 and starts at Phase 2 never gets in the air** (the concepts are detached from any artifact they've built). **A student who finishes Phase 1 and skips Phase 2** can fly straight lines on a calm day but crashes the first time conditions change. **A student who finishes Phase 2 and skips Phase 3** has all the skills but no judgment about *when* to apply them. Kirk & Hwu's three-phase ordering is the answer to "how do we get them from zero to operational in 14 weeks?"

### Mechanics

**1. Phase 1 — One lecture, Chapter 3, "write a working kernel today" (p.14).** The full claim, restated:

```
   Phase 1: ONE LECTURE  →  Ch.3 only
   ─────────────────────────────────
   Lecture covers:
     - basic CUDA memory model (host vs device, malloc/memcpy/free)
     - basic CUDA threading model (grid → block → thread, indexing)
     - CUDA C language extensions (__global__, <<<grid, block>>>, threadIdx, …)
     - basic programming/debugging tools (nvcc, cuda-gdb, compute-sanitizer)

   End-state target:
     "After the lecture, students can write a naïve parallel
      matrix multiplication code in a couple of hours."

   Performance? Not yet. Correctness + working kernel is the bar.
```

**Why one lecture is enough for Phase 1:** CUDA's *programming model* is small. The whole novelty over C is: (a) `__global__` annotation marks kernel functions; (b) `<<<grid, block>>>` launches them; (c) `threadIdx.x`, `blockIdx.x`, `blockDim.x` give each thread its identity. Everything else (memory model details, performance, parallel patterns) is *optimization*, not the model itself. The book intentionally separates **"can you write any kernel that produces correct output"** from **"can you write a fast kernel"** — the first is one lecture, the second is the rest of the course.

**2. Phase 2 — Ten lectures, Chapters 4–7, "make it 10× faster" (p.14).** The performance-engineering middle. Chapter map:

| Ch. | Topic | What it teaches you to *see* |
|---|---|---|
| **4** | Threads (the atom of work) | Warps, blocks, grids; how thread IDs map to data |
| **5** | Memory hierarchy | Global / shared / constant / texture; **the single biggest perf knob** |
| **6** | Performance considerations | Occupancy, divergence, coalescing, bank conflicts |
| **7** | Floating-point | IEEE-754 sharp edges, reduction non-associativity, precision modes |

**The 10× speedup claim (p.14) is concrete.** A naïve `O(N³)` matrix-multiply kernel on a GPU runs at ~1–5% of peak FLOPS — bottlenecked entirely by global-memory bandwidth (each thread re-reads the same row/column from DRAM). Apply Phase 2's lessons one at a time:

```
Optimization                          Effect on matmul throughput
────────────────────────────────────  ─────────────────────────────
Naïve kernel (Phase 1)                  ~1×    bandwidth-bound
+ shared-memory tiling (Ch.5)           ~5–8×  reuse each tile B times
+ thread coarsening (Ch.6)              ~1.5×  fewer threads, more work each
+ proper coalescing (Ch.5+6)            ~1.5×  one cache line per warp, not 32
+ unrolled inner loop (Ch.6)            ~1.2×  ILP improvement
+ proper FP precision use (Ch.7)        ~1.0×  (correctness, not speed; FP16 if 2×)
──────────────────────────────────────────────────────────────────
                                       ≈ 10× total over naïve
```

**The Phase 2 assignments (p.14):** convolution, vector reduction, prefix scan. These aren't random picks — they are the **three canonical GPU patterns**:
- **Convolution** = stencil / map-with-neighborhood (image processing, conv layers in deep learning)
- **Vector reduction** = sum/min/max/argmax across an array — the canonical *tree-reduction* exercise that teaches you warp shuffles and shared-memory atomics
- **Prefix scan** (parallel prefix sum, Blelloch's algorithm) = the "I didn't know you could parallelize that" workhorse pattern used in sorting, sparse matrix ops, stream compaction

If you finish Phase 2 having written all three at competitive performance, **you are competent at CUDA**. Phase 3 turns competence into judgment.

**3. Phase 3 — Computational thinking + breadth (Chs.8–11) (p.14).** Different lectures, different goal: now the student must *choose* an algorithm and architecture for a real problem. Chapters:

| Ch. | Topic | What it teaches you to *decide* |
|---|---|---|
| **8** | MRI reconstruction case study | "How do I take a real-world inverse problem and decompose it for a GPU?" |
| **9** | Molecular visualization case study | Memory coalescing in *anger* — under real, irregular access patterns |
| **10** | Parallel programming + computational thinking | Problem decomposition methodology; *which* parallel algorithm fits *this* problem |
| **11** | OpenCL | The same concepts on different silicon; portability lesson |

Phase 3 also includes the *online lecture recordings* (p.14, http://courses.ece.illinois.edu/ece498/al) — Hwu's actual class videos, freely available, *which are still the best free CUDA pedagogy in 2026* if you can stomach the 2009-era recording quality. They're a primary source for this book's chapters.

**4. The final project apparatus — five interlocking artifacts (pp.14–16).** Each artifact targets a specific failure mode:

| Artifact | Targeted failure mode |
|---|---|
| **Mentor + project specification sheet** | Student picks a toy problem because they don't know the field; mentor anchors the project in real research significance |
| **Project workshop** (6 lecture slots) | Student commits to a project nobody else has thought about; peer feedback surfaces blind spots, recruits teammates, validates scope |
| **Design document** (background, objectives, design, plan, verification, schedule) | Student starts coding before thinking; design doc forces sequencing |
| **Project clinic** (1 week before symposium) | Student panic-codes the last week and ships a half-broken demo; clinic forces a debug-and-revise pass with three pre-built code versions in hand |
| **Final report + symposium presentation** | Student writes for grade-bot; presentation forces compression and individual Q&A surfaces who actually did what |

The **three code versions required at the clinic (p.16)** are the key engineering discipline buried in the apparatus:

1. **Best CPU sequential code** — SSE2 and other optimizations; the *strong serial baseline*. Without this, GPU speedup claims are meaningless (it's easy to "GPU-accelerate" a deliberately bad CPU implementation).
2. **Best CUDA parallel code** — the actual project output; the implementation the student is being graded on.
3. **CPU sequential equivalent of the GPU algorithm, in single precision** — measures the *parallel-algorithm overhead* (extra computations forced by parallel decomposition) separately from the parallel-hardware speedup.

**Speedup = (3) / (2), not (1) / (2).** Version (1)/(2) inflates the speedup by counting algorithmic improvements as if they were parallelism. This rigor is what separates published-research-grade GPU work from blog-post-grade benchmarks.

**5. The workshop's three diagnostic questions (p.15).** What instructors and TAs are listening for when students present preliminary thinking:

1. **"Is the project too big or too small for the time available?"** — Scope calibration. Most failures cluster at *too big*.
2. **"Is there existing work in the field that the project can benefit from?"** — Avoiding re-invention; identifying baseline algorithms to compete with.
3. **"Are the computations being targeted for parallel execution appropriate for the CUDA programming model?"** — The hardest question, and the one Ch.10's *Computational Thinking* trains you to answer. Some problems are **embarrassingly parallel** (map-style — perfect for GPU); some are **data-parallel with sync** (reduction, scan — fine on GPU with care); some are **fundamentally sequential** (sequential decision processes, certain graph traversals) — putting them on a GPU will at best break-even, often regress.

**The 36-presenter / 90-minute class-size constraint (p.15)** — 10 minutes per project for feedback + Q&A. This is a *systems* design choice: feedback that takes more than 10 minutes per project doesn't scale, so the apparatus is calibrated to deliver maximum signal in minimum time.

**6. The online supplements (p.17) — what you actually get if you go look.** The book's companion site at the publisher (Elsevier) provides:
- **Lab assignments** (with reference solutions, instructor-only)
- **Final-project guidelines** (the apparatus described here, generalized for adopters)
- **Sample project specifications** (real mentor-supplied problem sheets from past ECE498AL)

In 2026, the **modern equivalents** are:
- NVIDIA's **CUDA Samples** GitHub repo (`NVIDIA/cuda-samples`) — hundreds of working kernels with explanations.
- **NVIDIA Deep Learning Institute** courses (free for many tracks, paid for others) — successor to Kirk & Hwu's pedagogy for the post-AlexNet world.
- **Kayvon Fatahalian's Stanford CS149 lecture notes** — free, modern, still uses the three-phase ordering.
- **Hwu's continued YouTube uploads of ECE408 lectures** — the direct continuation of the original 2007 course, with 2020s hardware.

### If you were a student opening this book for the first time…
Apply the chunk's own pedagogy to yourself. **Today, week 1**: read Ch.3 only. Write a working but slow kernel — vector add, matmul, dot product, anything. *Run it.* If you cannot run it, you have a CUDA-installation problem, not a learning problem; fix the installation before reading anything else. **Weeks 2–6**: read one chapter per week from Chs.4–7, each time taking your toy kernel from week 1 and applying that chapter's optimization. Convolution, reduction, and scan should be the three side-projects you write *while* reading Phase 2. By the end of week 6 your matmul should be within 2× of cuBLAS, your reduction within 1.5× of CUB, your scan within 2× of Thrust. **Weeks 7+**: pick a real problem (image filter, n-body simulation, sparse linear solver, transformer attention), apply Phase 3's computational-thinking methodology, and *do not allow yourself to start coding before writing a one-page design document*. The design document is the chunk's hidden lesson: not paperwork, *forcing function* to think before doing. Skipping any phase costs you nothing in the short term but compounds into "I learned CUDA but can't ship a project" — the failure mode the apparatus exists to prevent.

### Cross-language view — what Phase-1 "write a working kernel" looks like today

```cuda
// Phase 1, vintage 2010 (the book's era). Vector add. Naïve, no optimization.
__global__ void vadd(const float *a, const float *b, float *c, int n) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i < n) c[i] = a[i] + b[i];
}

int main() {
    float *dA, *dB, *dC;
    cudaMalloc(&dA, N * sizeof(float));  cudaMalloc(&dB, N * sizeof(float));
    cudaMalloc(&dC, N * sizeof(float));
    cudaMemcpy(dA, hA, ..., cudaMemcpyHostToDevice);
    cudaMemcpy(dB, hB, ..., cudaMemcpyHostToDevice);
    vadd<<<(N+255)/256, 256>>>(dA, dB, dC, N);
    cudaMemcpy(hC, dC, ..., cudaMemcpyDeviceToHost);
    cudaFree(dA); cudaFree(dB); cudaFree(dC);
}
```

```python
# Phase 1, 2026 idiom. The book's "naïve kernel" is now a one-liner because Triton
# and CuPy hide the boilerplate.
import cupy as cp
a = cp.random.rand(N, dtype=cp.float32)
b = cp.random.rand(N, dtype=cp.float32)
c = a + b   # one line; CuPy generates and launches the kernel
```

```python
# Phase 2 (modern). Same vector add as a Triton kernel — explicit tiling, but
# Python-syntax, JIT-compiled to PTX.
import triton, triton.language as tl

@triton.jit
def vadd_kernel(a_ptr, b_ptr, c_ptr, n, BLOCK: tl.constexpr):
    pid     = tl.program_id(0)
    offsets = pid * BLOCK + tl.arange(0, BLOCK)
    mask    = offsets < n
    a = tl.load(a_ptr + offsets, mask=mask)
    b = tl.load(b_ptr + offsets, mask=mask)
    tl.store(c_ptr + offsets, a + b, mask=mask)
```

```rust
// Rust — cuRAND-style wrappers (cust crate) keep CUDA's C model + add Rust's
// safety. Same shape; the launch macro is the equivalent of <<<grid,block>>>.
let _stream = stream.launch(&vadd_module, "vadd",
    grid, block, 0, &[&dA, &dB, &dC, &n]).unwrap();
```

**What the ecosystem actually does in 2026:** for *application* code, you almost never write Phase-1 CUDA C in production — you use **PyTorch / JAX / TensorFlow** (which compile to CUDA), **Triton** (Python DSL → PTX), or **CUDA Graphs**. For *library* code (cuBLAS, cuDNN, FlashAttention, vLLM kernels), Phase-2-level handwritten CUDA C++ with template metaprogramming and inline PTX is still the norm. The pedagogy this book teaches is *most* relevant if you want to write the libraries that the framework users consume.

### Where this shows up in real systems
- **NVIDIA's GTC technical sessions** for new hardware (Hopper 2022, Blackwell 2024) all use the three-phase ordering: introduce the new feature (Phase 1, "what's new"), show optimization for the new feature (Phase 2, "how to make it fast"), then a case study (Phase 3, "look what we built with it"). Kirk & Hwu's pedagogy *is* NVIDIA's external-comms structure.
- **Modern parallel-programming courses** (CMU 15-418, Stanford CS149, MIT 6.172, Illinois ECE408) all use Phase 1's "write a working kernel today" assignment in week 1, then a Phase-2-style sequence of optimization assignments, then a Phase-3-style final project. The structure is now the field's *default*.
- **The three-code-versions discipline at the clinic (p.16)** is exactly what **MLPerf benchmarks** require: a strong CPU baseline, the accelerator implementation, and an apples-to-apples algorithmic equivalent. Without this, accelerator vendors could (and used to) inflate speedups by 10–100× by competing against deliberately-bad baselines.
- **The "have a working kernel today" Phase-1 ethos** lives on in **NVIDIA's `Hello GPU` tutorials**, **AWS Trainium's Neuron SDK quickstarts**, **Apple Metal Performance Shaders' first-kernel guides**, and **Modular's Mojo language** — all premised on "run a working kernel before reading 200 pages of theory."
- **The design-doc-before-code requirement** is everyday FAANG engineering discipline (Amazon's PRFAQ, Google's design docs, Meta's "Roadmaps" doc) — Kirk & Hwu were inculcating it in undergrads in 2007.
- **Bruner's spiral curriculum** (1960) — visit a concept, deepen, revisit — explains why the book *re-introduces* the memory hierarchy in Ch.5, then again in Ch.6 (as performance levers), then again in Ch.9 (as the bottleneck in molecular visualization), then again in Ch.12 (as the projected evolution). Each visit makes the abstraction more operational.

### Diagnostic questions
1. *"Should I read this book front-to-back or use the three phases?"* — Use the phases. Front-to-back without writing code in Phase 1 means you'll forget the model by the time you reach Phase 2. The book is not a novel; it's a curriculum.
2. *"I read Ch.3, wrote a working kernel, and it's slow. Is that bad?"* — That's *expected*. Phase 1's success criterion is *correctness*, not speed. Phase 2 will make it 10× faster. If your naïve kernel is already fast, you wrote it wrong (probably forgot to verify output) or your benchmark is too small to be bandwidth-bound.
3. *"My final-project idea is 'GPU-accelerate Algorithm X.' Is that enough?"* — No. Apply the workshop's three questions: too big/small? prior work to leverage? appropriate for CUDA programming model? Many algorithms (Dijkstra on sparse graphs, certain dynamic programs) are *bad* GPU candidates; you'll do real research finding *why* before you write a kernel.
4. *"Why does the clinic require a CPU-equivalent of the parallel algorithm?"* — Because the alternative is the dishonest speedup. If your "GPU is 100× faster than CPU" turns out to be "the GPU uses a better algorithm," the speedup belongs to the algorithm, not the hardware. Real research distinguishes the two.
5. *"The book is from 2010 — is the three-phase pedagogy still right in 2026?"* — Yes, with one update: **Phase 1 should also include a framework-level kernel** (PyTorch + Triton or CuPy), not only raw CUDA C, because that's the realistic 2026 user. The *structure* (working kernel → optimization → judgment) is unchanged.
6. *"I want to skip Phase 1 and start with Triton — is that wrong?"* — Tactically maybe okay; strategically no. Triton hides the threading and memory model that Phase 2 needs you to understand. You'll hit a performance ceiling and not know why. Spend two weeks on raw CUDA C, *then* graduate to Triton; the framework will feel sensible instead of magical.
7. *"What's the modern equivalent of the symposium?"* — Internal team demo days, conference talks (GTC, MLSys, SC), and increasingly *technical blog posts* (NVIDIA Developer Blog, AnyScale's, Modular's). The artifact has changed; the discipline (compress, present, take individual Q&A) hasn't.

### See also
- **CUDA earlier entries [2026-05-17] and [2026-05-18]** — book front matter Chs.1–8 and Chs.9–12 curriculum spine. Together with this entry, the full pedagogical scaffolding is in place; next session can finally start Chapter 1's prose.
- **CUDA Ch.3 (next session, probably 2 sessions out)** — Phase 1's only chapter. Read it with the chunk's pedagogical map in mind: this is the *one chapter* that has to leave you able to write a working kernel by the end.
- **CMU 15-418 / Stanford CS149 free notes** — modern, ECE498AL-descended curricula. Excellent supplement, in some places clearer than the 2010 book on architecture details.
- **NVIDIA `CUDA C++ Best Practices Guide`** (continuously updated) — Phase 2's modern checklist; the canonical reference for the optimization techniques Chs.5–7 introduce.
- **DDIA Ch.10–11** (Batch + Stream Processing) — same Phase-1/2/3 pedagogy at the cluster scale; "write a working MapReduce, then make it fast, then ship a real pipeline" is the macro version of the GPU micro-pedagogy.
- **CLRS Ch.27** (Multithreaded Algorithms) — the formal model (work `T₁`, span `T_∞`, parallelism `T₁/T_∞`) that Phase 3's *Computational Thinking* chapter applies informally.
- **TPP Ch.13 Estimating** (entry [2026-05-19]) — Phase 1 is *literally a tracer bullet for CUDA pedagogy*; the apparatus is risk reduction applied to learning, not to product development.
- **N2T Chs.4–6** (CPU, Assembler, VM) — the *sequential* baseline that GPU SIMT diverges from; understanding the simple Hack CPU first makes the GPU's many-thread machinery legible as "what if you had 100,000 of these in parallel?"

---

## [2026-05-18] The Back-Half Curriculum + Preface — What the Book Promises About Optimization, OpenCL, and the Hardware Roadmap · pp.10–13 · Front Matter (TOC Chs. 9–12 + Appendices + Preface intro)

### TL;DR
This chunk closes the TOC (Chapters 9–12, two appendices) and opens the Preface. Treat it as **two arguments stacked on top of each other**. (1) The **back-half curriculum** — Chs. 9 (molecular visualization case study), 10 (parallel programming and computational thinking), 11 (OpenCL), 12 (conclusion and future outlook) — tells you what success looks like: a programmer who can decompose a real problem, pick an algorithm that fits the hardware, port to a non-NVIDIA platform if needed, and reason about where the hardware is going next. (2) The **Preface's pedagogical thesis** is unusually explicit and worth taking seriously: this book was written *specifically* to make parallel programming as pervasive as calculus — its target reader is a domain scientist (mechanical, civil, chemistry, bio, physics), not a CS specialist, and the structure deliberately frontloads "write a kernel that works" before "understand why it's fast." The chunk also previews the **hardware-evolution menu** that Ch. 12 will close on — unified device memory, configurable cache/scratchpad, atomic ops, double-precision speed, control-flow efficiency — every item of which has since landed in real silicon, making the 2010-vintage Ch. 12 readable as a **15-year-old crystal ball that was almost entirely right**. This is a low-page-count, high-orientation entry; the next session enters Ch. 1 prose proper.

### History — "why does this exist?"
The first edition of Kirk & Hwu (this one, 2010) was the **first widely adopted university textbook on GPGPU programming**, written by the people who taught the first university course on it — Wen-mei Hwu's **ECE 498AL at UIUC, January 2007**, just months after CUDA 1.0 shipped (June 2007). David Kirk was NVIDIA's Chief Scientist at the time and was personally pulling GeForce 8800 GTX boards out of customer shipments to get hardware into UIUC's lab — a detail the Preface lets slip (p.12) that tells you how new and how scarce this technology was. The book exists because between 2007 and 2010 the GPGPU community's only references were (a) NVIDIA's CUDA Programming Guide (a reference manual, not a curriculum) and (b) a fast-growing pile of SIGGRAPH/Supercomputing research papers. The textbook gap was real, and Kirk/Hwu filled it. **Ch. 11 on OpenCL** exists because Apple drove the **OpenCL 1.0 standard through Khronos in December 2008**, partly to prevent NVIDIA from owning all of GPGPU — Kirk and Hwu, being NVIDIA-affiliated, nonetheless included an OpenCL chapter because the *concepts* (work-items, work-groups, kernels, memory hierarchy) are isomorphic to CUDA's and a serious textbook had to acknowledge it. The **molecular visualization case study (Ch. 9)** is from John Stone's VMD work at UIUC's Beckman Institute — the canonical "embarrassingly-parallel-but-with-memory-coalescing-traps" case used to teach the optimization loop.

### Intuition — "this is like…"
A **driving-school syllabus.** Lessons 1–8 of any drivers' course teach the *mechanics* (steering, gas, brake, mirrors, lane changes). The next lessons teach the *judgement* — when to merge, how to read the traffic ahead, what to do when the weather turns. Chs. 9–12 of Kirk & Hwu are the judgement chapters. By the time you arrive, you can already write a CUDA kernel that works; the question is now *which* kernel to write for *this* problem on *this* hardware, and how to reason about hardware that hasn't shipped yet. Ch. 9 is the worked test drive. Ch. 10 is the route-planning skill. Ch. 11 is "the same skills on a different car." Ch. 12 is "where the car industry is going."

### Mechanics

**1. The full curriculum as a dependency graph (Chs. 1–12, redrawn).** This is the map you should keep open as you read; every chapter answers a question raised by the previous one:

```
        ┌─────────────────────────────┐
   Ch.1 │ Why parallel exists         │   "the clock-speed wall in 2004"
        └────────────────┬────────────┘
                         ▼
        ┌─────────────────────────────┐
   Ch.2 │ History: graphics → GPGPU   │   "how we got general-purpose silicon
        └────────────────┬────────────┘    out of a graphics card"
                         ▼
        ┌─────────────────────────────┐
   Ch.3 │ CUDA programming model      │   "your first working kernel"
        └────────────────┬────────────┘
                         ▼
        ┌─────────────────────────────┐
   Ch.4 │ Threads (the atom of work)  │   "what a thread/block/grid IS"
        └────────────────┬────────────┘
                         ▼
        ┌─────────────────────────────┐
   Ch.5 │ Memory hierarchy            │   "where the real ceiling lives"
        └────────────────┬────────────┘
                         ▼
        ┌─────────────────────────────┐
   Ch.6 │ Performance considerations  │   "warps, occupancy, divergence"
        └────────────────┬────────────┘
                         ▼
        ┌─────────────────────────────┐
   Ch.7 │ Floating-point sharp edges  │   "IEEE-754 + parallel reduction = surprises"
        └────────────────┬────────────┘
                         ▼
   ╔══════════════════════════════════╗
   ║                                  ║
   ║   APPLICATION + JUDGEMENT TIER   ║
   ║                                  ║
   ╚════════════════╤═════════════════╝
                    ▼
        ┌─────────────────────────────┐
   Ch.8 │ MRI reconstruction          │   first end-to-end case study
        └────────────────┬────────────┘
                         ▼
        ┌─────────────────────────────┐
   Ch.9 │ Molecular visualization     │   coalescing + warps in anger
        └────────────────┬────────────┘
                         ▼
        ┌─────────────────────────────┐
   Ch.10│ Parallel thinking + algo    │   problem decomposition methodology
        └────────────────┬────────────┘
                         ▼
        ┌─────────────────────────────┐
   Ch.11│ OpenCL                      │   the concepts, on different silicon
        └────────────────┬────────────┘
                         ▼
        ┌─────────────────────────────┐
   Ch.12│ Future outlook              │   the 2010 crystal ball
        └─────────────────────────────┘
```

**The transition between Ch.7 and Ch.8 is the spine break.** Below the line you are learning *mechanics* (how a CUDA program is shaped); above the line you are learning *judgement* (when to use which mechanic). Most students drop the book at Ch.7 and miss the half that turns them into engineers.

**2. The Ch. 9 case study — what the chapter actually teaches.** The TOC entries are themselves an outline of the optimization loop:

| Section | Topic | The lesson encoded |
|---|---|---|
| 9.1 | Application background | "every kernel solves a real problem; understand the problem first" |
| 9.2 | A simple kernel implementation | "always ship a naïve version that works before tuning" |
| 9.3 | Instruction execution efficiency | "warp divergence kills throughput" |
| 9.4 | **Memory coalescing** | "the single biggest perf knob on GPU — go read this twice" |
| 9.5 | Additional performance comparisons | "you must measure, not predict" |
| 9.6 | Using multiple GPUs | "scaling out the GPU side, not just the CPU side" |

**The order matters.** Most novices try to optimize first — Kirk & Hwu force you to write naïve→correct→fast in that order, which is the only order that lets you *prove* each optimization made things better.

**3. Ch. 10 — Computational Thinking, the book's most-skipped-most-valuable chapter.** Sections **10.2 Problem Decomposition** and **10.3 Algorithm Selection** are the GPGPU equivalent of CLRS's "algorithms are technology" argument: not every problem is parallelizable, and *picking* a parallel algorithm is a different skill from *implementing* it. The four sections (Goals → Decomposition → Algorithm Selection → Computational Thinking) lay out a **methodology**, not a recipe. This is where the book moves from "how to write CUDA" to "how to think GPU-shaped."

**4. Ch. 11 — OpenCL as a portability lesson, not a competitor.** The chapter sections (Background, Data Parallelism Model, Device Architecture, Kernel Functions, Device Management & Kernel Launch, Electrostatic Potential Map in OpenCL) deliberately re-walk the same concept ladder Chs. 3–5 walked for CUDA. The pedagogical point: **the concepts transfer; only the syntax changes.** A kernel is a kernel; a work-group is a thread block; an `__local` is `__shared__`; an `enqueueNDRangeKernel` is a `<<<grid, block>>>` launch. Master CUDA's *concepts* in Chs. 3–8 and OpenCL is a rename exercise. In 2026 the heirs of this lesson are **SYCL** (Khronos), **HIP** (AMD's CUDA-syntax-compatible runtime), **Triton** (OpenAI's Python DSL that targets CUDA/AMD), and **Mojo** (Modular's heterogeneous-compute language) — all premised on the same "concepts portable, syntax variable" thesis.

**5. Ch. 12 — the 2010 crystal ball, graded in 2026.** The Preface previews the futures Ch. 12 will name. Let's grade each:

| Ch. 12 prediction (2010) | Shipped? | When / How |
|---|---|---|
| 12.2.1 Large virtual + physical address spaces | ✅ | NVIDIA Pascal (2016) unified addressing; current Hopper 64-bit virtual addrs |
| 12.2.2 **Unified Device Memory Space** | ✅ | CUDA 6 (2014) `cudaMallocManaged`; the foundational shift |
| 12.2.3 Configurable cache vs. scratch pad | ✅ | Fermi (2010, just shipping) had 16/48 KB split; refined in every gen since |
| 12.2.4 Enhanced atomic operations | ✅ | atomicCAS on doubles (CC 6.0); FP16 atomics (CC 7.0); atomic ops on `__half2` |
| 12.2.5 Enhanced global memory access | ✅ | L2 cache (Fermi+); async copy `cp.async` (Ampere); TMA (Hopper) |
| 12.3.1 Function calls within kernel | ✅ | CC 2.0 (Fermi) full function-call ABI |
| 12.3.2 Exception handling in kernels | ⚠️ | Partial — assertions yes, full C++ exceptions still no in 2026 |
| 12.3.3 Simultaneous multiple kernels | ✅ | Concurrent kernels since Fermi; CUDA streams, graph capture |
| 12.3.4 Interruptible kernels | ✅ | Preemption since Pascal (2016) |
| 12.4.1 Double-precision speed | ✅ | Tesla GPUs: FP64 = FP32 / 2 throughput; consumer cards still gimped |
| 12.4.2 Better control-flow efficiency | ✅ | Independent thread scheduling (Volta+, 2017) — solved divergent warp deadlock |

**Reading this in 2026 is humbling**: the authors saw the architectural roadmap with near-perfect clarity in 2010. The interesting *miss* is on programming environment (12.5) — they predicted toolchain maturation; what actually arrived was *higher-level abstractions* (Thrust → cuDNN → PyTorch → Triton) that hide CUDA from most users. The substrate they predicted; the audience shift they didn't.

**6. The Preface's target-audience claim (p.13) — and why it shapes the book.** Kirk & Hwu state explicitly: the target reader is a **domain scientist with basic C** — mech eng, civil eng, EE, bio-eng, physics, chemistry — not a CS specialist. This explains three otherwise puzzling editorial choices:
- **Working kernels in Ch. 3 before warps in Ch. 6.** A CS student would want the architectural reasoning first; a domain scientist wants to see results first and tolerate "magic" until later. The book chooses the second audience.
- **C, not C++.** In 2010 the target audience knew C from Numerical Recipes; C++ would have alienated them. (Modern editions have shifted; CUDA C++ is mainstream by 2020.)
- **The MRI / molecular case studies.** These are not toy examples — they are bait. A bioengineer who sees their own field's compute problem solved on a GPU is more likely to finish the book than one shown matrix multiply.

The "**40,000 programmers actively using CUDA**" figure on p.13 is from 2010. In 2026 the number is north of **5 million CUDA developers** per NVIDIA's most recent GTC keynote — a 125× growth over 15 years, driven almost entirely by deep learning, which had not yet happened when this book was written (AlexNet was 2012).

### Where this shows up in real systems
- **`cudaMallocManaged` (Unified Memory)** — Ch. 12.2.2's prediction shipped four years later in CUDA 6 (2014) and is now the default way junior CUDA programmers write code. The whole `// host pointer == device pointer` simplification of modern PyTorch's CUDA backend rests on this.
- **Hopper TMA (Tensor Memory Accelerator)** — the latest implementation of Ch. 12.2.5's "enhanced global memory access." TMA hardware can move tiles of memory autonomously, freeing threads — the natural endpoint of the "make memory coalescing easier" arc.
- **Triton (OpenAI, 2021)** is Ch. 11's OpenCL lesson taken to its logical conclusion: a Python-level DSL that compiles down to PTX *or* AMD GCN, treating GPU vendor as a target, not a paradigm.
- **HIP / ROCm** (AMD) is *the* CUDA-source-compatible-on-AMD project; Kirk & Hwu's "concepts are portable" thesis was vindicated when AMD literally adopted CUDA's syntax to lower porting friction.
- **OpenCL's slow eclipse** — Ch. 11's optimism aged poorly; OpenCL was overtaken by SYCL (still Khronos) and vendor-specific stacks. The chapter is still valuable historically and conceptually.

### Diagnostic questions
1. *"This is a 2010 book; why read it in 2026?"* — Two reasons. (a) The hardware-evolution chapter is a useful **case study in how to read a roadmap correctly** (Kirk & Hwu got nearly every architectural prediction right). (b) The pedagogical *order* (working kernel → threads → memory → warps → divergence) is still the right teaching order, even on Hopper.
2. *"Should I skip Chs. 9–10 (case studies + computational thinking) and jump to PyTorch?"* — Only if you intend to be a library *user*, not a library *author*. The case studies and decomposition methodology are exactly what distinguishes engineers who can write a new fused kernel from those who only call existing ones.
3. *"Why does the book put OpenCL in Ch. 11 instead of Ch. 3?"* — Because OpenCL's verbosity (host code is ~5× the LOC of CUDA's) would have crushed momentum for a domain-scientist reader. CUDA first, OpenCL as the portability footnote. This same logic still applies to "learn CUDA first, then SYCL" in 2026.
4. *"What's the modern equivalent of this book?"* — There isn't a single replacement. The most-cited successors are **Cheng/Grossman/McKercher, *Professional CUDA C Programming* (2014)** for depth, **Kirk & Hwu 3rd edition (2017)** for breadth, and **NVIDIA's CUDA C++ Best Practices Guide** (continuously updated) for current hardware. But for *pedagogical structure*, this 1st edition is still the cleanest.
5. *"What's Ch. 12's biggest miss?"* — The rise of **deep learning as the dominant GPU workload**. The book imagines a world where the GPU runs scientific simulations; in fact, 2026's GPU is mostly running transformer training. The same hardware features the authors predicted (atomics, unified memory, fast FP, async memory ops) are still the load-bearing ones — they just serve a workload the authors didn't anticipate.

### See also
- **CUDA earlier entry [2026-05-17]** (Front matter — Chs. 1–8 curriculum spine) — together with this entry, you now have the full 12-chapter map of the book.
- **COD (Computer Organization and Design) Ch. 6 (parallel processors)** — the CPU-side companion to Kirk & Hwu's GPU-side framing. Read both to see the same Moore's-law-stall driving two different architectural answers (multicore SMP vs SIMT GPU).
- **N2T Ch. 5 (CPU)** — provides the single-thread-of-execution baseline that GPU SIMT breaks away from.
- **DDIA Ch. 10–11** (Batch + Stream Processing) — the cluster-of-CPUs version of "compose lots of small workers"; GPUs are the same idea at silicon scale.
- **CLRS Ch. 27 (Multithreaded Algorithms)** — formalizes work-depth analysis (`T_p`, `T_∞`, `T_1/T_∞` parallelism) that gives you the language to talk about *whether* a problem is parallelizable, which is what Ch. 10's "Computational Thinking" implicitly trains you to recognize.

---

## [2026-05-17] The Curriculum Spine — Reading Kirk & Hwu's TOC as an Argument · pp.6–9 · Front Matter (Copyright + Table of Contents, Ch. 1–8)

### TL;DR
This chunk is the copyright page and the first half of the table of contents — front matter, no prose content. But the TOC is itself an argument: **Kirk & Hwu have ordered the curriculum so that each chapter teaches the *concept* and the *bottleneck it solves***, in the order a developer will hit them in practice. Ch.1 frames why parallel hardware exists; Ch.2 explains how GPUs got here historically; Ch.3 introduces CUDA's programming model; Ch.4 zooms in on the *thread* (CUDA's atom of work); Ch.5 zooms out to the *memory hierarchy* (the real performance ceiling); Ch.6 collects the **performance-tuning levers** (warps, occupancy, divergence); Ch.7 handles the **floating-point sharp edges** that catch every CUDA newcomer; Ch.8 is the first end-to-end application case study (MRI reconstruction). The sequence is **not** by feature — it's by **how the optimization loop actually plays out**: get something working (Ch.3), understand the threads (Ch.4), find the memory ceiling (Ch.5), then climb performance hill by hill (Ch.6–7). This is a day-1 roadmap entry; the technical depth begins next session.

### History — "why does this exist?"
The 2010 edition (this book) is the **first widely adopted university textbook on GPGPU programming**. Before it, the GPU compute community learned from NVIDIA's CUDA Programming Guide (a reference manual, not a curriculum) and from research papers. Kirk (NVIDIA's then-Chief Scientist) and Hwu (UIUC, who taught the original CUDA course ECE 498/598) co-authored this book based on the **2007 UIUC course** — the first university course anywhere on heterogeneous parallel programming, taught when CUDA itself was barely a year old (CUDA 1.0 shipped June 2007). The TOC's specific ordering — *programming model before architecture deep dive* — was deliberate pedagogy: it lowers the barrier for students whose background is sequential C, by letting them write working code in Ch.3 before they need to understand warps in Ch.6.

### Intuition — "this is like…"
A **driving-school syllabus.** Lesson 1: why cars exist (Ch.1 — parallel hardware exists because clock speeds stalled). Lesson 2: how cars evolved (Ch.2 — graphics → unified shaders → general compute). Lesson 3: how to start the engine and drive in a straight line (Ch.3 — write your first kernel). Lesson 4: how the steering wheel works (Ch.4 — threads + blocks). Lesson 5: where the gas station is and why running out matters (Ch.5 — memory hierarchy). Lesson 6: how to drive efficiently in traffic (Ch.6 — performance considerations). Lesson 7: edge cases you'll hit (Ch.7 — floating point). Lesson 8: a real road trip (Ch.8 — MRI reconstruction case study). The order *is* the lesson plan; skipping ahead breaks the dependency chain.

### Mechanics

**The curriculum as a dependency graph (Ch.1 → Ch.8):**

```
        ┌──────────────────────────────┐
   Ch.1 │ Why parallel? (motivation)   │   "why are we here?"
        └──────────────┬───────────────┘
                       ▼
        ┌──────────────────────────────┐
   Ch.2 │ History: graphics → GPGPU    │   "how did we get here?"
        └──────────────┬───────────────┘
                       ▼
        ┌──────────────────────────────┐
   Ch.3 │ CUDA model:                  │   ← first working code
        │   data parallelism           │
        │   kernel = SPMD function     │
        │   device memory + transfer   │
        └──────────────┬───────────────┘
                       ▼
        ┌──────────────────────────────┐
   Ch.4 │ CUDA threads:                │   ← zoom into the atom
        │   blockIdx / threadIdx       │
        │   __syncthreads()            │
        │   transparent scalability    │
        │   warps, scheduling          │
        └──────────────┬───────────────┘
                       ▼
        ┌──────────────────────────────┐
   Ch.5 │ CUDA memories:               │   ← the real ceiling
        │   global / shared / const    │
        │   "memory access efficiency" │
        │   memory as the limiter      │
        └──────────────┬───────────────┘
                       ▼
        ┌──────────────────────────────┐
   Ch.6 │ Performance considerations:  │   ← climbing the hill
        │   warp execution, divergence │
        │   global memory bandwidth    │
        │   dynamic SM partitioning    │
        │   prefetching, granularity   │
        └──────────────┬───────────────┘
                       ▼
        ┌──────────────────────────────┐
   Ch.7 │ Floating-point:              │   ← the sharp edges
        │   IEEE 754 in CUDA           │
        │   precision, rounding        │
        └──────────────┬───────────────┘
                       ▼
        ┌──────────────────────────────┐
   Ch.8 │ Case study: MRI              │   ← putting it together
        │   FHd computation            │
        │   kernel parallelism shape   │
        │   bandwidth-limited regime   │
        └──────────────────────────────┘
```

**Three TOC observations that matter for how to read the book:**

1. **Ch.5 (Memories) comes before Ch.6 (Performance) — not interleaved.** This is a deliberate split: Ch.5 teaches the *taxonomy* (global vs shared vs constant vs texture), and Ch.6 teaches the *tuning* (coalescing, bank conflicts, occupancy). The book wants you to know *what exists* before you optimize. Read Ch.5 carefully; the optimizations in Ch.6 will be incomprehensible without it.

2. **Ch.7 (Floating-Point) is a chapter, not an appendix.** This is unusual — most parallel-computing books bury IEEE 754 issues in an aside. The TOC's prominence signals that **non-associative reduction errors** (the order in which 10⁶ parallel threads sum their contributions changes the answer) are first-class CUDA failure modes. Treat Ch.7 as load-bearing, not optional.

3. **Ch.8 is the first of several application case studies** (Ch.9–11 in the rest of the book), arranged from "MRI reconstruction" (numerical) outward. The case-study structure is the book's pedagogical core — every concept from Ch.3–7 reappears in Ch.8 *under pressure of a real application's memory and bandwidth constraints*.

**Section anatomy of Ch.5 — the chapter to watch.** From the TOC (p.8):

| § | Title | What it's really teaching |
|---|---|---|
| 5.1 | Importance of Memory Access Efficiency | Setup: why this matters |
| 5.2 | CUDA Device Memory Types | Taxonomy: global / shared / const / local / register |
| 5.3 | A Strategy for Reducing Global Memory Traffic | The single most important technique: **tiling** |
| 5.4 | Memory as a Limiting Factor to Parallelism | Why "occupancy" exists |

Section 5.3 (tiling) is the **fulcrum of the entire book** — the single technique most CUDA programs use to break the bandwidth ceiling. The whole curriculum funnels toward it.

### Where this shows up in real systems
- **NVIDIA's official CUDA C Programming Guide.** Mirrors this TOC's *order* almost exactly — programming model → memory → performance — because the order reflects the optimization loop a real CUDA programmer walks through.
- **University courses (Stanford CS149, MIT 6.5940, CMU 15-418).** All teach CUDA in the same Kirk-Hwu order, sometimes citing this book as the canonical reference. The pedagogy has been validated by ~15 years of student outcomes.
- **The cuBLAS / cuDNN source organization.** NVIDIA's own production libraries are structured by the same axes — kernel implementations grouped by which memory/bandwidth regime they target (compute-bound vs bandwidth-bound vs latency-bound). Reading the TOC primes you to recognize the layers in production code.

### Diagnostic questions
1. **Q:** Why does Ch.3 (CUDA programming model) come *before* Ch.4 (threads) when threads are conceptually more fundamental?
   *Wrong-answer trap:* "Threads are an implementation detail." Closer: Kirk & Hwu want students *writing working kernels by end of Ch.3*, even if they don't yet understand the thread-execution model. Motivation precedes precision — a classic pedagogy choice.
2. **Q:** Where in the TOC would you expect "memory coalescing" to appear?
   *Wrong-answer trap:* "Ch.5 (Memories)." Wrong layer — Ch.5 names the memories; coalescing is a *bandwidth-tuning technique*, so it lives in **Ch.6 §6.2 (Global Memory Bandwidth)**. Knowing the layer mapping prevents wasted searching.
3. **Q:** A new CUDA programmer skips straight to Ch.6 because they want to optimize. What goes wrong?
   *Wrong-answer trap:* "Nothing — Ch.6 is self-contained." It isn't — Ch.6's vocabulary (warps, divergence, SMs, occupancy) is built up across Ch.4 and Ch.5. Reading Ch.6 cold is the #1 reason students bounce off the book.
4. **Q:** Why does the book treat MRI reconstruction (Ch.8) — a domain-specific case study — as the first major application?
   *Wrong-answer trap:* "Because the authors work in medical imaging." More likely: MRI's FHd kernel is **embarrassingly parallel but bandwidth-bound**, making it the perfect first case study — every Ch.5 and Ch.6 technique applies, with measurable speedups, and no domain knowledge beyond linear algebra is required.

### See also
- COD 2026-05-15 (CPU performance equation) — Kirk & Hwu's curriculum is the parallel-hardware counterpart to COD's single-core analysis; the CPU equation generalizes to per-thread on the GPU.
- CLRS 2026-05-17 (Algorithms as a technology) — the algorithmic-vs-hardware framing flips on the GPU: at very large n, throwing 10⁴ cores at an O(n²) operation can beat one core running O(n lg n). Ch.6 (Performance) is where this trade-off gets quantified.
- DDIA 2026-05-17 (Reliability) — both books open by separating *what the system does* from *what could go wrong*; DDIA's "fault vs failure" mirrors CUDA's "correctness vs performance" decoupling.

---

## [2026-05-16] Scope-Setting — Why a Book on Massively Parallel Processors? · pp.2–5 · Front Matter (Praise Blurbs + Title Page)

### TL;DR
The chunk is pre-chapter front matter (praise blurbs, title page, intentionally-blank page) — but the blurbs themselves do real framing work. They state Kirk & Hwu's two-axis thesis: **parallel programming is about performance** (otherwise sequential is simpler and clearer), and **GPUs are the most accessible parallel hardware** in mainstream computers (hundreds of cores, sitting in millions of laptops). The book teaches **CUDA** (NVIDIA's C-like data-parallel language) over **Tesla** (the GPU architecture line at the time of writing, ~2010), with two extended case studies reportedly showing **10–15× speedups for naïve CUDA code** and **45–105× speedups for expertly tuned versions** over CPU-only C. The blurbs also flag the book's closing material: a forward look at **OpenCL** (the cross-vendor data-parallel standard, Khronos 2009) and **Fermi** (NVIDIA's next-generation architecture after Tesla). This is a **scope-setting entry**, not a deep technical one — the substantive ideas (kernels, threads, blocks, the memory hierarchy, divergence, occupancy) begin in the actual chapters that follow.

### History — "why does this exist?"
GPU programming did not start as a general-compute story. **From 1996 (3dfx Voodoo) through ~2003 (NVIDIA GeForce FX, ATI Radeon 9700)**, GPUs were fixed-function pipelines — texture, shade, rasterize, ship pixels. The shift happened in two pulses. **Pulse 1 (2001–2006): shader programmability.** Vertex and fragment shaders (DirectX 8 / OpenGL ARB) let researchers smuggle non-graphics computation into a GPU by encoding it as fake rendering. **Mark Harris's GPGPU.org (2002–2008)** catalogued the community; **Brook (Stanford, 2004, Buck et al.)** was the first credible high-level language. **Pulse 2 (2006–2008): CUDA and unified shader architecture.** **NVIDIA's G80 (GeForce 8800, Nov 2006)** unified vertex and fragment shaders into a single pool of general-purpose cores ("**streaming multiprocessors**"), and **NVIDIA released CUDA 1.0 in June 2007** — the first commercially supported, C-with-extensions language for general GPU compute. Kirk & Hwu's first edition (Jan 2010) appeared in this window. **David Kirk** was NVIDIA's Chief Scientist (1997–2009) and a Turing Award-eligible architect of CUDA; **Wen-mei Hwu** ran the parallel computing lab at UIUC and co-taught the first university course built around CUDA. The book is *the* canonical practitioner text — the parallel-computing equivalent of K&R for C. **OpenCL (2009)** is the open-standard reaction to CUDA's NVIDIA-exclusivity, championed by Apple and Khronos; **Fermi (2010)** added IEEE-754 compliant double precision, ECC memory, and a unified address space — the moment GPUs became scientific-computing peers, not just gaming chips. Reading the blurbs against this timeline makes them legible as a **historically positioned 2010 manifesto**: GPU compute had just become legitimate, and this book was the textbook that consolidated the field.

### Intuition — "this is like…"
The CPU vs GPU difference is the **commercial-airliner-vs-school-bus-fleet** analogy. A modern CPU is a 777 — four to sixteen extremely capable cores, deep pipelines, huge caches, aggressive branch prediction, optimized for taking a single passenger from A to B as fast as possible (low single-thread latency). A GPU is a fleet of school buses — thousands of simple cores, shallow pipelines, no fancy speculation, optimized for taking *millions* of passengers from A to B per hour (high aggregate throughput, indifferent to any one passenger's wait). For a sequential workload (a long phone call, a chain of `if` branches with unpredictable outcomes), the 777 crushes the bus fleet. For a data-parallel workload (sort a billion numbers, multiply two matrices, train a neural network), the bus fleet crushes the 777 — even though each bus is slower per passenger. The whole CUDA book is teaching you when your problem is a phone call vs. a stadium evacuation, and how to load the buses correctly.

### Mechanics

There is no real mechanics content in pp.2–5 — it's praise blurbs and the title page. Instead, this entry **inventories the vocabulary the blurbs hint at**, so future chapters land in a primed mind.

**Terms the blurbs name, with one-line previews:**

| Term | What it is (preview only) | When it shows up |
|---|---|---|
| **CUDA** | NVIDIA's C-with-extensions for writing code that runs on the GPU. The host launches *kernels*; the device runs them on thousands of threads. | Chapter 2–3 onward |
| **Kernel** | A function annotated `__global__` that executes once per GPU thread when launched. The unit of GPU work. | Ch. 3 |
| **Thread / Block / Grid** | Threads execute the kernel; threads are grouped into blocks (typically 32–1024 threads); blocks form a grid. This hierarchy determines what can synchronize with what. | Ch. 3–4 |
| **Tesla** (architecture, not the company's branded products) | The GPU architecture family this edition targets: G80, GT200, etc. Pre-Fermi. | Throughout |
| **Streaming Multiprocessor (SM)** | The hardware unit that runs a block. Contains arithmetic units, register file, shared memory, instruction scheduler. | Ch. 4 |
| **Warp** | 32 threads executed in lock-step on an SM. The unit of *actual* hardware parallelism, distinct from the logical thread. **Warp divergence** is the single most common CUDA performance pitfall. | Ch. 6 |
| **Shared memory** | A small (~48 KB per SM in this era), low-latency scratchpad. The most important resource the programmer manages by hand. | Ch. 5 |
| **OpenCL** | Khronos's cross-vendor parallel standard. Same conceptual model as CUDA, broader hardware reach, more verbose. | Final chapters |
| **Fermi** | The post-Tesla architecture (NVIDIA, 2010). First with IEEE-compliant double precision and ECC. | Forward-looking final chapter |

**The 10–15× / 45–105× speedup claim — context for the credulous reader.**

The blurb's framing is honest if read carefully. **10–15× for naïve CUDA over CPU-only C** is the *typical* result of porting a data-parallel kernel directly — the threads-and-blocks abstraction does most of the work for you. **45–105× for expertly tuned** is what you get after applying every technique the book teaches: coalesced global-memory access, shared-memory tiling, avoiding warp divergence, occupancy tuning, asynchronous transfers. Two cautions:
1. The CPU baseline in 2010 was a single-threaded C program; on a 2026 multicore CPU with AVX-512 and good cache use, the gap is real but smaller (~3–20× for the same workloads).
2. The book's case studies are *embarrassingly parallel* (matrix multiply, MRI reconstruction). Workloads with heavy branching, pointer chasing, or sparse access patterns do **not** see those speedups — GPUs punish irregularity.

### Where this shows up in real systems
- **Deep learning, basically all of it.** PyTorch, TensorFlow, JAX — every modern ML framework is fundamentally a CUDA-kernel-launcher with a nicer API. The Tesla→Fermi→Kepler→Pascal→Volta→Ampere→Hopper→Blackwell lineage this book starts is the same lineage that produced the H100 GPUs training every frontier LLM today.
- **CUDA's vendor lock-in is the strategic story of the 2020s.** OpenCL never matched CUDA's tooling; AMD's ROCm and Intel's oneAPI are still chasing CUDA-completeness. NVIDIA's ~80% datacenter-GPU market share rests almost entirely on the moat this book describes.
- **Scientific computing renaissance.** ECC + double-precision on Fermi (2010) was the moment HPC moved from CPU-only clusters (Top500 dominated by x86) to GPU-accelerated clusters (Top500 increasingly dominated by NVIDIA + AMD GPUs from 2012 onward).
- **The "GPU programming is hard" myth and reality.** The blurbs say "easy-to-comprehend concepts." That's true for the *programming model* (threads/blocks/kernels). Performance tuning is genuinely hard — warp divergence, memory coalescing, occupancy, register pressure all interact non-locally. This is exactly what the book's case studies will demonstrate.

### Diagnostic questions
1. **Q:** A workload contains 99% sequential code and 1% data-parallel code. Worth porting to CUDA?
   *Wrong-answer trap:* "Yes — 1% on a GPU is fast." Amdahl's law: even infinite GPU speedup on the parallel 1% leaves 99% sequential. Max overall speedup is ~1.01×. The 1% needs to be **inside** an inner loop that dominates the runtime profile, not the runtime's total.
2. **Q:** Why are GPUs better for matrix multiply than CPUs, given a modern CPU also has SIMD?
   *Wrong-answer trap:* "More cores." Partially — but the deeper answer is **memory bandwidth + thousands of threads hiding memory latency**. A GPU has 5–20× the memory bandwidth of a CPU and the thread count to keep that bandwidth saturated; a CPU has more compute per core but cannot feed it as fast for huge data.
3. **Q:** Why didn't OpenCL win, despite being open?
   *Wrong-answer trap:* "Tooling." Partly — but the bigger reason is NVIDIA invested ten years into CUDA's *libraries* (cuBLAS, cuDNN, cuSPARSE, NCCL) before OpenCL had equivalents. The lock-in is at the library level, not the language level.
4. **Q:** A CUDA kernel runs 100× faster than the CPU version. Should you celebrate?
   *Wrong-answer trap:* "Yes." Verify two things first: (a) the CPU baseline was actually optimized (multi-threaded, SIMD-vectorized, cache-blocked) — if it was a single-threaded reference, the 100× includes free wins the CPU also leaves on the table; (b) the GPU result is *correct* to the precision you need (mixed precision and non-deterministic atomics can silently change answers).

### See also
- COD §1.5 (already noted, 2026-05-15): wafer yield economics — why GPUs are massive *and* commercially viable despite enormous die sizes (it's a relentless yield-optimization story).
- DDIA Preface (today's other entry): "data-intensive vs compute-intensive" — CUDA lives on the *compute-intensive* side; DDIA explicitly excludes it from its scope.
- CLRS Ch.1 (today's other entry): "algorithms as technology" — Strassen, FFT, etc. — many of the algorithms whose constant factors GPUs blow up most dramatically.
- N2T Ch.5–6: the same hardware-software contract (ISA) idea, but here the "ISA" is the CUDA virtual machine (PTX), not x86 or ARM.

---
