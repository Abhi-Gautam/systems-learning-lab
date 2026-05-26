# N2T Notes

_Entries follow the template at `Notes/TEMPLATE.md`. Append-only. **Newest entry at top**, immediately after this header._

---

## [2026-05-27] VM program control: flow & calls · pp.155–166 · Ch.7 §7.3.3–§7.5 (Project) → Ch.8 §8.1–§8.1.2

- VM translator architecture: Parser + CodeWriter two-module design
- `label` / `goto` / `if-goto` — the three opcodes that encode all control flow
- Subroutine call protocol — how a stack machine saves and restores caller state

### History — "why does this exist?"

Program flow via labels and jumps is as old as computing itself — **Konrad Zuse's Z3 (1941)** had conditional branch, and **EDSAC (1949)** stored programs with jump instructions at fixed addresses. But the idea of encoding *all* high-level control flow (`if`, `while`, `for`, `switch`) into just two primitives — **unconditional goto** and **conditional goto** — was formalized by **Böhm and Jacopini (1966)**, who proved any flowchart can be expressed with sequence, selection, and iteration. The subroutine-call protocol is a separate lineage: **David Wheeler's EDSAC subroutine library (1951)** was the first implementation of "jump to a piece of code and come back," and **McCarthy's LISP (1960)** first demonstrated recursive calls backed by a stack discipline. The stack-based calling convention that N2T implements is essentially the same protocol the **JVM (1995)** and **CLR (2002)** use, minus the type verification.

### Intuition — "this is like…"

Think of the VM translator as a **Kubernetes controller that turns declarative YAML into imperative shell commands**. The `.vm` file says `call Sys.init 0` — a declarative intent ("run this function"). The translator emits 20+ lines of Hack assembly that save registers, repoint the stack, jump, and eventually restore everything. The caller never sees the assembly, just as a pod spec never mentions `cgroup` syscalls. The `label`/`goto`/`if-goto` trio is even simpler: it's the assembly-level `jmp`/`jnz` you already know, but scoped to a function — like Kubernetes labels that are namespace-scoped rather than cluster-global.

### Mechanics

#### The VM translator's two-module architecture (§7.3.3)

```
   Xxx.vm  ──►  ┌─────────┐    ┌────────────┐  ──►  Xxx.asm
                 │  Parser  │───►│ CodeWriter  │
                 └─────────┘    └────────────┘

   Directory mode:
   *.vm  ──►  N × Parser  ──►  1 × CodeWriter  ──►  Dir.asm
```

| Module | Responsibilities | Key routines |
|---|---|---|
| **Parser** | Tokenize one `.vm` file; classify commands into 9 types; extract `arg1`, `arg2` | `advance()`, `commandType()`, `arg1()`, `arg2()` |
| **CodeWriter** | Emit Hack assembly for each command; manage label uniqueness across files | `writeArithmetic()`, `writePushPop()`, `setFileName()` |
| **Main** | Wire Parser → CodeWriter; handle file-vs-directory input | single loop: `while hasMoreCommands: advance → dispatch` |

The design mirrors the **Unix pipe philosophy**: Parser is a lexer with no knowledge of assembly; CodeWriter is an emitter with no knowledge of `.vm` syntax. The Main loop is glue. In directory mode, each `.vm` file gets its own Parser instance but all share one CodeWriter — which is why `setFileName()` exists: the CodeWriter needs the current filename to generate unique `static` labels (`@Foo.3` vs `@Bar.3`).

#### Program flow commands (Ch.8 §8.1.1)

Three commands encode **all** control flow:

```
label LABEL        ← declare a jump target (emits no code, just an assembly label)
goto LABEL         ← unconditional jump (PC = address of LABEL)
if-goto LABEL      ← pop top of stack; if ≠ 0, jump to LABEL; else continue
```

**How `if` and `while` reduce to these three:**

```
High-level            VM translation

if (cond)             [compute ~cond, push result]
  s1                  if-goto L1
else                  [code for s1]
  s2                  goto L2
                      label L1
                      [code for s2]
                      label L2

while (cond)          label LOOP
  body                [compute ~cond, push result]
                      if-goto END
                      [code for body]
                      goto LOOP
                      label END
```

The subtle point: `if-goto` tests the **negation** of the condition — when cond is true, fall through to s1; when false, jump to L1 (the else branch). This is the **branch-on-false** convention, the same one GCC and LLVM use in their internal IR. It minimizes jumps on the common (true) path.

**Translation to Hack assembly** is mechanical:

| VM command | Hack assembly |
|---|---|
| `label LOOP` | `(functionName$LOOP)` |
| `goto LOOP` | `@functionName$LOOP` / `0;JMP` |
| `if-goto LOOP` | `@SP` / `AM=M-1` / `D=M` / `@functionName$LOOP` / `D;JNE` |

Labels are scoped by prepending the current function name — `functionName$LOOP` — so two functions can both use `LOOP` without collision. This is the same namespace-scoping trick C linkers use with static-linkage symbols.

#### Subroutine calling protocol (§8.1.2)

The eight things a stack machine must do per call:

```
┌─ CALLER side ─────────────────────────────────────────────┐
│  1. Push arguments onto the stack                         │
│  2. call functionName nArgs                               │
│     └─► push return-address                               │
│         push LCL, ARG, THIS, THAT  (save caller's frame)  │
│         ARG = SP - nArgs - 5       (reposition ARG)        │
│         LCL = SP                   (reposition LCL)        │
│         goto functionName                                  │
└───────────────────────────────────────────────────────────┘

┌─ CALLEE side ─────────────────────────────────────────────┐
│  3. function functionName nLocals                         │
│     └─► push 0 × nLocals  (initialize local segment)      │
│  4. Execute body                                          │
│  5. push return value                                     │
│  6. return                                                │
│     └─► endFrame = LCL     (save for unwinding)            │
│         retAddr = *(endFrame - 5)                          │
│         *ARG = pop()       (put return value for caller)   │
│         SP = ARG + 1       (discard callee's frame)        │
│         THAT = *(endFrame-1); THIS = *(endFrame-2)         │
│         ARG  = *(endFrame-3); LCL  = *(endFrame-4)         │
│         goto retAddr                                       │
└───────────────────────────────────────────────────────────┘
```

**The stack frame layout at the moment the callee starts executing:**

```
          ┌────────────────────────┐
   ARG ──►│ argument 0             │
          │ argument 1             │
          │ ...                    │
          │ argument nArgs-1       │
          ├────────────────────────┤
          │ return address         │  ← saved by call
          │ saved LCL             │
          │ saved ARG             │
          │ saved THIS            │
          │ saved THAT            │
          ├────────────────────────┤
   LCL ──►│ local 0 (= 0)        │  ← pushed by function
          │ local 1 (= 0)        │
          │ ...                    │
    SP ──►│ (empty — work stack)  │
          └────────────────────────┘
```

The five saved values between ARG's last argument and LCL's first local form the **frame record** — the production term is **activation record** or **stack frame**. This is structurally identical to what `gcc -fno-omit-frame-pointer` emits on x86-64, where `rbp` points at the saved frame pointer and return address sits just above it.

**Why the return value goes to `*ARG`:** after `return`, the caller expects the result at the top of *its* stack — which is the slot where `argument 0` used to be. By writing the return value there and then setting `SP = ARG + 1`, the callee's entire frame vanishes and the return value is exactly where the caller would find it after a `push`.

#### The two-tier compilation model (§7.4 Perspective)

```
                    Frontend tier              Backend tier
                   (Ch.10–11)                 (Ch.7–8)
   Jack source ──► Jack Compiler ──► .vm ──► VM Translator ──► .asm ──► Assembler ──► .hack
                                      │
                                      └─ portable: runs on any VM backend
```

The `.vm` file is the **portable artifact** — same role as JVM `.class` files, CLR `.dll` files, or WASM `.wasm` modules. The frontend (compiler) doesn't know about Hack; the backend (VM translator) doesn't know about Jack. The decoupling means you could target the same `.vm` to a different platform by writing a new backend, or compile a different language to `.vm` by writing a new frontend.

Real-world parallels: **JVM** serves Java/Kotlin/Scala/Clojure; **CLR** serves C#/F#/VB; **LLVM IR** serves C/C++/Rust/Swift/Zig. The N+M argument from the previous entry is the economic motivation; this chapter shows the *implementation* — the VM translator *is* the M-side backend.

### If you were the VM translator…

You're implementing `call Sys.init 0`. The tricky part: you need to push a return address, but the return address is the *next instruction after your emitted call sequence*. You don't know that address at emit time because you haven't finished emitting yet. How do you solve it?

You **generate a unique label** — say `Sys.init$ret.0` — and emit `@Sys.init$ret.0` as the return address to push. At the end of the call sequence, you emit `(Sys.init$ret.0)`. The assembler resolves the forward reference in its two-pass algorithm (exactly the pattern from Ch.6). Each `call` gets a unique label by appending an incrementing counter. This is the same technique JIT compilers use: emit a label placeholder, fix up later. The VM translator generates assembly *with unresolved labels* and delegates resolution to the assembler — a clean separation of concerns.

### Cross-language view

Every language runtime implements the same call protocol; only the details differ:

| | Hack VM | JVM | x86-64 (System V ABI) | Go |
|---|---|---|---|---|
| Save return addr | push as data word | `jsr` pushes to operand stack | `call` pushes to hardware stack | `BL` stores in `LR`; prologue saves to stack |
| Save caller state | push LCL/ARG/THIS/THAT | JVM manages internally per frame | `push rbp; mov rbp,rsp` | goroutine stack; `NOSPLIT` for leaf |
| Allocate locals | `push 0` × nLocals | verifier knows local count from `.class` | `sub rsp, N` | compiler allocates frame at compile time |
| Return value | write to `*ARG` | push to caller's operand stack | `rax` register | multiple returns via stack slots |
| Frame cleanup | restore from saved values | JVM pops frame automatically | `leave; ret` | `RET` pops saved `LR` |

The Hack VM makes the protocol *explicit* — every push and restore is a visible VM command. Production runtimes hide it behind a single hardware instruction (`call`/`ret`) plus ABI conventions, but the logical steps are identical.

### Where this shows up in real systems

- **GDB's `backtrace` command** walks the chain of saved frame pointers (the `saved LCL → saved ARG → ...` chain in Hack terms) to reconstruct the call stack. When you see `#0 main() ... #1 libc_start_main()`, GDB is reading exactly the linked list of activation records that Hack's `return` command unwinds.
- **JavaScript's call stack limit** (`RangeError: Maximum call stack size exceeded`) exists because each `call` allocates a frame. V8's default stack is ~1 MB; each frame is ~100–500 bytes depending on local count. The Hack VM has the same limit — the stack region is RAM 256–2047, giving ~1792 words of frame space, enough for ~50 nested calls with 30 locals each.
- **Tail call optimization** (TCO) in functional languages (Scheme, Erlang, some Kotlin/Swift) works by *reusing the current frame* instead of pushing a new one. In Hack terms: instead of the full `call` protocol, a tail call would overwrite the current frame's arguments, reset SP, and `goto` — skipping the 5-word frame save entirely. This is why recursive Erlang processes don't blow the stack.

### Diagnostic questions

1. **Q:** Why does `if-goto` test for ≠ 0 rather than = 0?
   *Wrong-answer trap:* "Convention." The real reason: the VM's boolean `true` is `0xFFFF` (all ones, = −1 in two's complement), which is ≠ 0. Testing ≠ 0 means `if-goto` fires on `true`, which matches the programmer's intent for `if (condition) goto target`. Testing = 0 would invert every conditional.

2. **Q:** What happens if a function `return`s without pushing a return value?
   *Wrong-answer trap:* "Runtime error." There's no runtime check — `*ARG = pop()` will pop whatever's on top of the stack (possibly a saved register or garbage). The caller will silently use a corrupt value. This is the VM equivalent of undefined behavior.

3. **Q:** Why is the return address saved *before* LCL/ARG/THIS/THAT?
   *Wrong-answer trap:* "Arbitrary convention." It's positioned so that `*(endFrame - 5)` reaches it after the callee sets `endFrame = LCL`. The five saved values occupy exactly the slots between the last argument and the first local, so the frame record is a contiguous block that `return` can unwind with fixed offsets.

4. **Q:** Two VM functions both use `label LOOP`. Does the translator emit duplicate assembly labels?
   *Wrong-answer trap:* "Yes, causing an assembler error." No — the translator prepends the function name: `(Foo$LOOP)` vs `(Bar$LOOP)`. Labels are function-scoped by construction.

5. **Q:** The VM translator emits ~20 assembly instructions for a single `call`. Is that expensive?
   *Wrong-answer trap:* "Yes, calls are slow." In Hack, yes — there's no hardware call instruction. But the same logical work happens in one or two cycles on x86 (`call` + `push rbp`) because the hardware implements the frame save in microcode. The 20-instruction expansion is the cost of building a CPU from NANDs with no call hardware.

---

## [2026-05-26] VM segments → Hack RAM · pp.143–154 · Ch.7 §7.2.3 (Segments cont'd) → §7.3.1 (Standard Mapping, Part I)

- Eight virtual segments + heap & stack, all backed by Hack's flat 16-bit RAM
- Predefined pointer registers (SP, LCL, ARG, THIS, THAT) — one indirection per segment access
- Worked: array via `pointer 1` / `that`; object via `pointer 0` / `this`

### History — "why does this exist?"
Stack-machine VMs trace to Niklaus Wirth's **P-code machine (1972)** for Pascal — the first widely-used "compile once, interpret everywhere" target, which freed Pascal compilers from being one-per-platform monoliths. UCSD's p-System extended it into a portable OS in 1978. The **JVM (1995)** is the same idea industrialised: classes compile to a stack-bytecode that any JVM on any chip can execute. The Hack VM, like its inspirations Smalltalk-80 and JVM, picks the **stack machine** over a register machine because stack ISAs are easier to compile *to* (no register allocator needed) — at the cost of being slower to execute without a JIT. By 2017, WebAssembly returned to this exact lineage with a wasm-style stack VM as the browser's universal target.

### Intuition — "this is like…"
Think of each VM function as a **Postgres prepared statement**: when you call it, the VM stages a tiny private environment — initialised local slots, argument values copied in, an empty work-stack — exactly the way Postgres stages a snapshot, plan, and parameter bindings for each `EXECUTE`. From inside the prepared statement you don't worry where the data physically lives; you reference `$1`, `$2`, locals by name. The VM gives each function the same illusion: `argument 2` always means "third argument of *this* invocation", regardless of where on the physical RAM stack those bytes actually sit today. The five pointer registers (`SP`, `LCL`, `ARG`, `THIS`, `THAT`) are the mapping table that keeps that illusion coherent across calls.

### Mechanics

**The eight segments, recapped:** `argument`, `local`, `static`, `constant`, `this`, `that`, `pointer`, `temp`. Plus the two **implicit** structures the VM commands never name directly but every push/pop touches: the **stack** and the **heap**.

**Hack RAM layout — the entire 32K is partitioned by convention, not hardware:**

```
RAM addr     Usage                                       Symbol(s)
─────────    ──────────────────────────────────────────  ─────────────
   0–15      Sixteen virtual registers (R0..R15)         R0..R15
                                                         (R0=SP, R1=LCL,
                                                          R2=ARG, R3=THIS,
                                                          R4=THAT)
                                                         (R5..R12 = temp)
                                                         (R13..R15 = scratch)
  16–255     Static variables of all VM functions        Xxx.j symbols
 256–2047    Stack                                       (SP points here)
2048–16383   Heap (objects + arrays)                     (THIS/THAT point in)
16384–24575  Memory-mapped I/O (screen, keyboard)        SCREEN, KBD
```

The first 16 RAM words *are* the registers — Hack has no separate register file. R0–R4 carry the predefined names `SP`/`LCL`/`ARG`/`THIS`/`THAT` so assembly can be readable; R5–R12 *are* the temp segment (no extra indirection); R13–R15 are scratch the translator can clobber.

**Segment access — translation rules side by side:**

| Segment | Translation of `push <seg> i` | Notes |
|---|---|---|
| `local`, `argument`, `this`, `that` | `addr = *(BASE_REG) + i`; `push RAM[addr]` | base in LCL / ARG / THIS / THAT |
| `pointer 0` | `push RAM[THIS]` (i.e. value of THIS reg) | rebinds the `this` segment when popped |
| `pointer 1` | `push RAM[THAT]` | rebinds the `that` segment when popped |
| `temp i` | `push RAM[5 + i]` | fixed base = 5; no indirection |
| `constant i` | `push i` | no RAM access — purely virtual |
| `static i` | `push RAM[@Xxx.i]` | assembler allocates a slot starting from RAM 16 |

The crucial insight: **`pointer 0` and `pointer 1` are how the user changes the base of THIS/THAT.** Writing to `pointer 0` rebinds the `this` segment; writing to `pointer 1` rebinds the `that` segment. That's the entire mechanism by which a method "becomes a method on object X" — `pop pointer 0` sets THIS to X's base address.

**Worked example 1: `bar[2] = 19` where `bar` is `local 0`.**

```vm
push local 0       // push bar's base address (call it B)
push constant 2    // push 2
add                // top of stack = B + 2
pop pointer 1      // THAT = B + 2  ← rebind 'that' segment
push constant 19
pop that 0         // RAM[THAT + 0] = 19 ← writes RAM[B+2]
```

```
Before:                          After 'pop that 0':
 local[0] = 4315                  RAM[4317] = 19
 pointer/THAT = ?                 pointer/THAT = 4317
 RAM[4317] = (old value)
```

The `bar` array was allocated at RAM 4315; element 2 lives at RAM 4317; rebinding THAT to 4317 makes `that 0` *be* that element.

**Worked example 2: `b.radius = r` where `b` is `argument 0`, `r` is `argument 1`, radius is the object's 3rd field.**

```vm
push argument 0    // push b's base address
pop pointer 0      // THIS = b's base ← rebind 'this' segment
push argument 1    // push r
pop this 2         // RAM[THIS + 2] = r ← b.radius = r
```

The compiler picked `this 2` because at code-generation time it had a symbol table mapping `radius` → field index 2 (third field, zero-indexed). All field accesses on `b` are now `this <field_index>` for the lifetime of this method.

**Why this design works.** Every VM-level access is **one indirection** through a base register — no per-instruction page-table walk, no MMU, no segment descriptors. The "virtual" in "virtual memory segment" means *logical view*, not the OS-level virtual memory you know from OSTEP. Each function reading `argument 0` always touches `RAM[ARG + 0]`, regardless of where the caller's stack frame happens to be — the VM translator's job (chapter 8) is just to keep `ARG` correctly pointing into the live stack frame.

**Trade-off you should see:**

```
┌─────────────────────────────────────────────────────────────┐
│  Stack machine (Hack VM, JVM, CPython, wasm)                │
│    + No register allocator in compiler — trivial to emit    │
│    + Compact bytecode (no register fields)                  │
│    – Every op pushes/pops — high memory traffic without JIT │
│                                                             │
│  Register machine (LLVM IR, Dalvik, real CPUs)              │
│    + Faster naive execution (values stay in registers)      │
│    – Compiler must do register allocation (graph colouring) │
│    – Bytecode is fatter (each op names its registers)       │
└─────────────────────────────────────────────────────────────┘
```

Hack picks stack because the focus of chapters 7–8 is *building the translator*, not *running fast*. JVMs and v8 ship JIT specifically to claw back the lost performance.

### If you were the VM translator…

You hit a `push static 3` inside the file `Game.vm`. You have no symbol table — Hack assembly has no notion of "scope" beyond labels. How do you assign `static 3` to a unique RAM slot that won't collide with `static 3` in another `.vm` file?

You **emit the symbol `@Game.3`** and let the Hack assembler do the work. The assembler allocates each new symbol a fresh RAM slot starting at address 16. Because `Game.3` is distinct from `Board.3`, the assembler gives them different slots automatically. You get per-file static-segment isolation for free — without ever writing allocation logic in the VM translator. This is the same trick C uses with `static`-linkage names mangled by file/scope; the assembler is your allocator.

### Cross-language view

Stack-machine VMs all share Hack's general layout. Names and exact register counts differ:

| | Hack VM | JVM | CPython | wasm |
|---|---|---|---|---|
| Stack | implicit, SP-tracked | per-frame operand stack | per-frame value stack | per-function operand stack |
| Locals | `LCL`-based segment | indexed `local[i]` slots | `co_varnames[i]` cells | indexed `local.get i` |
| Args | `ARG`-based segment | first N entries of local | same as locals | merged with locals |
| Globals/static | per-file `Xxx.j` | per-class `static` field | module-level `globals()` dict | `global.get i` |
| Heap base | `THIS`/`THAT` registers | implicit, via object refs | implicit, via PyObject* | `memory[0]` linear memory |

In all four, the compiler's job is the same: turn a high-level field access into "rebind a base pointer, then index off it." Hack just exposes those base pointers (THIS, THAT) as user-visible registers; the JVM hides them behind the bytecode verifier.

### Where this shows up in real systems

- **JVM bytecode verifier** does the same kind of mapping Hack's translator does — but eagerly, at class-load time. It walks each method's bytecode, tracks the inferred stack shape at each instruction, and rejects code that would put a non-object on `THIS`-equivalent operations (`invokevirtual`, `getfield`). This is exactly the discipline a Hack programmer maintains by hand: `pop pointer 0` must be preceded by something that produces a valid object base address.
- **PostgreSQL's executor** uses an analogue: per-tuple `ExprContext` carries per-row argument/local slots that expressions reference by index. `ExecEvalScalarVar` walks the slot array exactly like Hack walks `argument i`.
- **Linux `clone()` + thread-local storage.** Each thread has its own LCL/ARG equivalent — the `%fs`/`%gs` segment registers on x86-64 point to a per-thread TLS area. `errno` is accessed as `*((tls_base + offset_of_errno))` — the same "register-relative indirection" Hack uses. The whole TLS design is one indirection away from the Hack VM's `local i` semantics.
- **WebAssembly's `local.get i`** is structurally identical to Hack's `push local i` — wasm modernised the idea with type-checking and linear memory, but the indirection-via-base-pointer model is unchanged 50 years after Pascal P-code.

### Diagnostic questions

1. **Q:** What does `pop pointer 0` actually do, in one sentence about RAM?
   *Wrong-answer trap:* "It pops a value off the stack." That's necessary but not the point — it overwrites `RAM[3]` (the THIS register), thereby rebinding the entire `this` segment to a new base address.

2. **Q:** Why does the VM translator emit `@Xxx.3` for `push static 3` in `Xxx.vm`?
   *Wrong-answer trap:* "Because static variables live at fixed addresses." They don't — the Hack assembler allocates them on first encounter starting at RAM 16. The mangled name *causes* per-file isolation; nothing else does.

3. **Q:** The temp segment has no dedicated base register. Why not?
   *Wrong-answer trap:* "Because temp is small." Size is a consequence, not the reason. Temp lives at fixed addresses 5–12 — there is no base to track because there is no per-function variation; temp is shared scratch across all functions of a single VM program, and the translator just emits `@5+i` directly.

4. **Q:** A VM function calls another. What is the bare minimum the translator must save and restore around the call?
   *Wrong-answer trap:* "All the segment registers." Wrong: `static`, `pointer`, `temp` are shared and need no save. `THIS`/`THAT` are the *caller's* responsibility to preserve if it cares; the callee gets fresh `LCL`/`ARG` it didn't have to ask for. Saving SP, LCL, ARG, return address suffices (chapter 8 spells this out).

5. **Q:** Why isn't `constant` mapped to any RAM region?
   *Wrong-answer trap:* "Because constants are small." Wrong — `push constant 19` emits `@19 / D=A / push D`; the 19 lives inline in the instruction stream, not in data RAM. Mapping it to RAM would *cost* a slot per constant in the program.

---

## [2026-05-25] VM paradigm & stack execution · pp.131–142 · Ch.6 §6.4 (Assembler test) → Ch.7 §7.1–§7.2.2 (Background → Stack Machine Model → Arithmetic Commands)

- Two-tier compilation — why a VM layer exists at all
- Stack machine model — push/pop/sp and the LIFO discipline
- Stack arithmetic — evaluating any expression as a sequence of pushes and ops

### History — "why does this exist?"
The abstract idea — one machine simulating another — is **Alan Turing's universal machine (1936)**, decades before silicon existed to run it. The first widely shipped *practical* form was **Pascal p-code (Niklaus Wirth & UCSD Pascal, ~1973)**: one Pascal compiler emitted "p-code" (portable code), and dozens of universities wrote tiny p-code interpreters for their local mainframes. The model went dormant for ~20 years, then exploded into mainstream practice with **Sun's JVM (1995)** — "write once, run anywhere" was a Pascal idea with marketing. **Microsoft's CLR (2002, CIL bytecode)** followed, then in the 2010s the model crossed into systems territory with **eBPF (Linux, 2014, register-based VM running untrusted kernel code)** and **WebAssembly (W3C, 2017, stack-based VM for browsers and edge runtimes like Cloudflare Workers and Fastly)**. The **stack-arithmetic** part of the picture has a separate lineage: **Forth (Charles Moore, 1970)** put RPN evaluation into hardware-adjacent programming, and HP's calculators (HP-35, 1972) trained a generation of engineers in stack-based math before they ever saw bytecode.

### Intuition — "this is like…"
A VM is **Docker for code, fifty years earlier**. The compiler emits a portable artifact (bytecode / a container image) against a stable abstract platform (the VM / the Linux kernel ABI), and a runtime on each target machine knows how to execute that artifact. Without the layer, every (language × hardware) pair needs its own compiler — **N × M** compilers; with the layer, you need N frontends + M backends — **N + M** effort. That's the entire pitch.

Stack-based execution is the **HP-12C** of computation: type `2`, type `3`, hit `+`, get `5` — no register names, no addressing modes, just topology. `add` doesn't take operands; it knows where to look (the top of the stack). The simplicity is exactly what makes the VM portable: a stack is a `Vec<i32>` plus an integer index; a register file is a hardware-shaped contract.

### Mechanics

#### Two-tier compilation: the N+M vs N×M argument

```
Without VM (N×M compilers)             With VM (N+M = frontend + backend)

   Java ─→ Java→x86 compiler          Java       ─┐
   Java ─→ Java→ARM compiler          Kotlin     ─┤   N frontends emit
   Java ─→ Java→RISC-V compiler       Scala      ─┘   the SAME bytecode
                                                       │
   Kotlin ─→ Kotlin→x86 ...                            ▼
   Kotlin ─→ Kotlin→ARM ...                       ┌─────────┐
   Kotlin ─→ Kotlin→RISC-V ...                    │ Bytecode│  ← portable artifact
                                                  └────┬────┘
   Scala ─→ … ad nauseam                               │
                                          ┌────────────┼────────────┐
                                          ▼            ▼            ▼
                                       VM on x86   VM on ARM   VM on RISC-V
                                          │            │            │
                                          ▼            ▼            ▼
                                       native       native       native
```

N=3 languages, M=3 ISAs → 9 compilers (without) vs 6 components (with). The arithmetic gets dramatic at real scale: JVM serves Java/Kotlin/Scala/Clojure/Groovy/JRuby/Jython × x86/ARM/RISC-V/POWER/zArch, but only Sun (then Oracle) had to write the M backends — the language teams write only the N frontends.

#### Stack machine state — what `add` actually does

```
Before:                        After (add executes):
sp ─→ [empty]                  sp ─→ [empty]
      │  17 │ ← top                  │  23 │ ← top (6 + 17)
      │   6 │
      │ 108 │                        │ 108 │
      │   . │                        │   . │
      └─────┘                        └─────┘

In code: sp -= 1; stack[sp-1] = stack[sp-1] + stack[sp];
```

`add` always: pops two, pushes one → net `sp -= 1`. No operands in the instruction. Compare to x86: `add eax, ebx` names both source registers explicitly — that's the **register-based** alternative.

| Dimension | Stack-based VM (JVM, WASM, Hack) | Register-based VM (eBPF, Dalvik, Lua) |
|---|---|---|
| Instruction width | Tiny (1-2 bytes, no operands for arith) | Wider (must encode register numbers) |
| Code density | Higher | Lower |
| Per-op overhead | More (push/pop bookkeeping) | Less (direct reg access) |
| Interpreter speed | Slower (sp updates per op) | Faster (fewer mem accesses) |
| Verifier complexity | Lower (stack typing is local) | Higher (must track register types) |
| Common in | JVM (1995), WASM (2017), Hack | Dalvik (Android, 2008), eBPF (2014), LuaJIT |

The trade is essentially: **stack-based optimizes for code size and verification simplicity**; register-based optimizes for interpreter speed. JIT compilers erase most of the runtime difference, which is why both flavors coexist.

#### Worked example — evaluating `d = (2 - x) * (y + 5)` with `x=5, y=9`

The high-level expression compiles to 8 VM commands:

```
push 2          push x          sub             push y
sp ─→            sp ─→           sp ─→           sp ─→
  │ 2  │           │ 5  │          │-3  │          │ 9  │
  └────┘           │ 2  │          └────┘          │-3  │
                   └────┘                          └────┘

push 5          add             mult            pop d
sp ─→            sp ─→           sp ─→           sp ─→ (empty)
  │ 5  │           │14  │          │-42 │
  │ 9  │           │-3  │          └────┘            d in memory = -42
  │-3  │           └────┘
  └────┘
```

Eight commands, no temporaries named, no register allocation. The high-level compiler's job is to **linearize the expression tree into postfix order** (`2 x - y 5 + *`) — which is exactly the depth-first traversal of the AST. That's why stack VMs are easier to target: the compiler back-end is a tree walker, not a register allocator.

#### The full VM command taxonomy (Hack VM, §7.2)

```
Arithmetic/Logical (9 commands, no operands)
  add  sub  neg               ← 2's complement integer math
  eq   gt   lt                ← comparison (push 0xFFFF for true, 0x0000 for false)
  and  or   not               ← bit-wise

Memory access (2 commands, segment + index)
  push segment index          ← read segment[index], push onto stack
  pop  segment index          ← pop top, write into segment[index]

  Segments: argument, local, static, constant, this, that, pointer, temp

Program flow (next chapter)
  label  goto  if-goto

Function calling (next chapter)
  function  call  return
```

Nine ALU commands cover all of `+ - * / < > = & | !` for both ints and bools. The "missing" multiply/divide are handled by routines in the OS layer (§9), not as ALU ops — a deliberate simplification that mirrors **early RISC philosophy** (MIPS didn't have hardware divide either; you called a software routine).

#### What the bytecode actually looks like on disk — JVM analogue

The Hack VM keeps its IR as text (.vm files) because pedagogy. Real production VMs encode bytecode as bytes for density. Example: the JVM compilation of `int x = a + b;`:

```
0x1a    iload_0      ; push local var 0 (a)            ← 1 byte, no operand
0x1b    iload_1      ; push local var 1 (b)            ← 1 byte
0x60    iadd         ; pop two, push sum               ← 1 byte
0x3c    istore_1     ; pop, store to local var 1 (x)   ← 1 byte
```

Four bytes, four instructions. `iload_0` is a separate opcode from `iload_1` precisely *because* eliminating the operand byte saves space — the most common stack reads got their own opcodes. WebAssembly took the lesson further: variable-length LEB128-encoded indices instead of fixed opcodes for common-case compression.

### If you were the VM designer…
The decision that drives everything else is **how do operands reach an operation?** Stack-based: operands live on a single growing pile, operations grab from the top. Register-based: operands live in named slots, operations name them. The Hack VM picks stack-based for the same reason JVM did: **the compiler is dumber**. A stack-VM backend just emits postfix; a register-VM backend has to do live-range analysis and assign physical registers — that's classic register allocation, which is the hardest single phase in a real compiler. By moving complexity from compiler-writer to interpreter-writer (the interpreter has to track sp), the stack VM lets thousands of language teams target one runtime that ten people maintain.

The textbook's answer is more humble: stack VMs are easier to *specify*, easier to *teach*, and Hack is a teaching machine. But the production lineage tells you the deeper reason — JVM picked stack-based in 1995 *knowing* about register-based alternatives, because Java needed to be implementable in many places by many teams.

### Where this shows up in real systems
- **JVM (`javac` → `.class` files)** runs every Kotlin/Scala/Clojure/Groovy program on the planet. Bytecode verification is a stack-typing problem: the JVM verifier walks each method's bytecode and proves that the stack's type and depth are consistent at every basic-block boundary, before any code runs. That static check is what made Java "safe to run untrusted applets" in 1995.
- **WebAssembly** is a 2017 redesign of the same idea for the web. Stack-based, statically-typed, structured control flow (no arbitrary `goto`), AOT- or JIT-compiled at load time. Cloudflare Workers, Fastly Compute@Edge, and Shopify's Functions all run WASM modules in V8 isolates because the stack-VM model is small enough to embed and verify in milliseconds per cold start.
- **eBPF** (Linux kernel, 2014) deliberately chose **register-based** (10 registers, 64-bit) instead of stack-based — because eBPF runs in kernel mode and the **verifier needs to bound execution exactly**. Register-based bytecode is easier to verify for termination and memory safety than stack-based, even though it's denser code.
- **Python's `dis` module** shows you Python's stack VM live: `dis.dis(lambda x: x + 1)` prints `LOAD_FAST 0; LOAD_CONST 1; BINARY_ADD; RETURN_VALUE`. CPython is a stack VM at heart, which is why the GIL exists in part to keep the stack consistent under threading.

### Diagnostic questions
1. **After 7 `push`es and 3 `add`s, what is sp's net change?** *Wrong:* "depends." *Right:* sp moves by **+4** (7 pushes add 7, 3 adds remove 3). Each `add` is net −1 because it pops 2 pushes 1.
2. **Why did WebAssembly choose stack-based when eBPF chose register-based, given both run untrusted code?** *Wrong:* "stack is faster." *Right:* WASM runs in user-space inside a JIT (V8, Wasmtime) where speed comes from JIT-compilation; the stack-VM gives smaller modules over the wire. eBPF runs **in the kernel** where the verifier must prove safety statically without JIT — register-VM is easier to verify because each register is a typed location, vs a stack whose contents shift per op.
3. **A Hack VM program compiles `if (x < 7) or (y = 8) then …`. How many items are on the stack between the `lt` and the `eq` commands?** *Wrong:* "two." *Right:* **one** — the boolean result of `x < 7` is sitting on top; `eq` then pushes `y` and `8`, computes, leaves a second boolean. After `or`, one boolean remains.
4. **Why is there no `mul` in early JVM bytecode for fixed-point or bignum math?** *Wrong:* "JVM is dumb." *Right:* JVM's `imul` is integer; floating-point uses separate opcodes (`fmul`, `dmul`). Bignum and decimal math live in `java.math.BigInteger`/`BigDecimal` as ordinary classes, called via `invokevirtual`. The VM keeps the opcode set small and pushes complexity to libraries — exactly what Hack does with multiplication.
5. **If you swapped Hack's VM for a register-based VM with the same expressiveness, what would get harder?** *Wrong:* "everything." *Right:* the *VM translator* (next chapter's main project) would get harder — it would need register allocation to map an arbitrary-depth stack-style expression onto a finite register file. The bytecode would shrink; the translator would grow.

---

## [2026-05-24] The Hack Instruction Encoding — Reading a 16-bit Word Like a CPU Does (and the Four-Module Assembler That Emits It) · pp.119–130 · Ch.6 Assembler §6.2 (Specification) → §6.3 (Implementation: Parser / Code / SymbolTable / Main)

### TL;DR
The Hack assembler's job, stripped of symbol bookkeeping, is to turn each line of `.asm` text into a single 16-bit word and append it to a `.hack` file. The encoding is famously regular: the **most significant bit** is the **instruction type** (`0` = A-instruction = load a 15-bit constant into A; `1` = C-instruction = compute), and for C-instructions the remaining 15 bits split into five named fields — `11 a cccccc ddd jjj` — where the **a-bit** picks the ALU's second input (A-register vs. memory), the **6 c-bits** select one of 28 ALU operations from a fixed table, the **3 d-bits** are a bitmap of *destinations* (A, M, D — independently settable), and the **3 j-bits** are a 3-way comparison-against-zero jump predicate. The assembler's implementation mirrors the encoding's structure: a **Parser** that tokenizes lines into (type, dest, comp, jump, symbol) tuples, a **Code** module that is essentially three lookup tables, a **SymbolTable** (a hashmap), and a **Main** driver that wires them together. The whole thing is ~300 lines in any language because **the encoding was deliberately designed to be easy to assemble** — a software/hardware co-design choice the chapter never names but everywhere demonstrates.

### Intuition — "this is like…"
Reading a Hack instruction is like reading a **license plate from a country where the first character tells you which alphabet to use for the rest**. See a `0` first → the rest is a 15-bit unsigned integer, nothing more to parse. See a `1` first → fixed-position bitfields, decoded by table lookup. The d-bits being a *bitmap* (not an enum) is the key elegance: `D=A+1`, `M=A+1`, `AM=A+1`, `AMD=A+1` are *not four opcodes* — they're one opcode (`A+1` in the comp field) with four different `ddd` bit patterns. Independent destinations × ALU operations gives the multiplicative explosion you see in the mnemonic table without needing N×M opcodes — the same architectural trick ARM uses with its conditional execution suffixes and predicated registers.

### Mechanics

#### What the chunk actually covers
- §6.2.1 binary file format (one 16-bit ASCII-binary line per instruction)
- §6.2.2 the **two instruction types** and their bit layouts (this entry's core)
- §6.2.3 the three symbol kinds (predefined / labels / variables) — *previously summarized in yesterday's entry; not re-derived here*
- §6.2.4 a worked example (sum 1..100, 12 instructions, side-by-side asm → binary)
- §6.3.1–6.3.5 the **four-module implementation**: Parser / Code / SymbolTable / Main, with a two-stage build plan (symbol-less first, symbol-aware second)
- §6.4 Perspective (macro assemblers, why standalone assemblers are rare in practice)
- §6.5 Project (test programs: Add, Max, Rect, Pong)

#### The 16-bit word, fully decoded

```
 bit:   15  14  13  12  11  10   9   8   7   6   5   4   3   2   1   0
       ┌───┬───────┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐
A-ins  │ 0 │  v   v   v   v   v   v   v   v   v   v   v   v   v   v  v│
       └───┴────────────────────────────────────────────────────────────┘
         │ └───────────────── 15-bit unsigned constant ────────────────┘
         └─ type bit (0 = A)

       ┌───┬───┬───┬───┬───────────────────────┬───────────┬───────────┐
C-ins  │ 1 │ 1 │ 1 │ a │ c1  c2  c3  c4  c5  c6│ d1  d2  d3│ j1  j2  j3│
       └───┴───┴───┴───┴───────────────────────┴───────────┴───────────┘
         │ ←─ unused, must be 1 ─→ │     comp (7 bits incl. a)    dest  jump
         └─ type bit (1 = C)
```

Five fields, three of which are independently settable bitmaps:

| Field | Bits | Role |
|---|---|---|
| **type** | 15 | `0` → A-instruction; `1` → C-instruction |
| **a** | 12 | Selects ALU's second operand: `0` → A-register, `1` → M (RAM[A]) |
| **c1..c6** | 11..6 | One of 28 valid combinations selecting an ALU operation |
| **d1, d2, d3** | 5, 4, 3 | Destination bitmap: `(A, D, M) ← comp` — any subset |
| **j1, j2, j3** | 2, 1, 0 | Jump predicate: `(<0, =0, >0)` bitmap; PC jumps if (ALU sign) matches |

Bits 13–14 are "must be 1" filler — they're the price of having one clean type-bit at position 15 while leaving the c-bits in a convenient 6-bit window.

#### The comp field — one table, two columns, swapped by the a-bit

The `a`-bit is *outside* the 6 c-bits but is functionally a 7th comp bit. The book's table has two columns of identical shape:

| Mnemonic (a=0) | c1 c2 c3 c4 c5 c6 | Mnemonic (a=1) |
|---|---|---|
| `0` | 1 0 1 0 1 0 | `0` |
| `1` | 1 1 1 1 1 1 | `1` |
| `-1` | 1 1 1 0 1 0 | `-1` |
| `D` | 0 0 1 1 0 0 | `D` |
| `A` / `M` | 1 1 0 0 0 0 | `M` |
| `!D` | 0 0 1 1 0 1 | `!D` |
| `!A` / `!M` | 1 1 0 0 0 1 | `!M` |
| `-D` | 0 0 1 1 1 1 | `-D` |
| `-A` / `-M` | 1 1 0 0 1 1 | `-M` |
| `D+1` | 0 1 1 1 1 1 | `D+1` |
| `A+1` / `M+1` | 1 1 0 1 1 1 | `M+1` |
| `D-1` | 0 0 1 1 1 0 | `D-1` |
| `A-1` / `M-1` | 1 1 0 0 1 0 | `M-1` |
| `D+A` / `D+M` | 0 0 0 0 1 0 | `D+M` |
| `D-A` / `D-M` | 0 1 0 0 1 1 | `D-M` |
| `A-D` / `M-D` | 0 0 0 1 1 1 | `M-D` |
| `D&A` / `D&M` | 0 0 0 0 0 0 | `D&M` |
| `D|A` / `D|M` | 0 1 0 1 0 1 | `D|M` |

The deep insight: **the a-bit doesn't change the ALU's wiring** — it changes what's *fed into* one input port. The same 6 c-bits compute `D+A` and `D+M`; the multiplexer in front of the ALU picks A or M based on the a-bit. This is why the Hack ALU has only 6 control bits despite supporting both A- and M-flavored operations: the 7th degree of freedom lives outside the ALU itself.

#### The dest field — a bitmap, not an enum

```
d1 d2 d3   dest mnemonic   meaning
─────────  ──────────────  ─────────────────────────────
 0  0  0   (null)          discard the ALU output
 0  0  1   M               M ← comp                     (just write to RAM[A])
 0  1  0   D               D ← comp                     (just write to D register)
 0  1  1   MD              M ← comp; D ← comp           (write to both)
 1  0  0   A               A ← comp
 1  0  1   AM              A ← comp; M ← comp
 1  1  0   AD              A ← comp; D ← comp
 1  1  1   AMD             A ← comp; D ← comp; M ← comp  (write to all three)
```

The bits are flags for **(A-register, D-register, M-memory)** — any subset is legal. The hardware is wired so each register's load-enable pin is one of d1/d2/d3 directly. No decoder, no special case. This is why `AMD=A+1` exists and is **one cycle** — three writes happen on the same clock edge because three load-enable wires are asserted simultaneously.

#### The jump field — a 3-way bitmap against ALU sign

```
j1 j2 j3   mnemonic   jumps if ALU output is …
─────────  ─────────  ────────────────────────
 0  0  0   (null)     never (PC just increments)
 0  0  1   JGT        > 0
 0  1  0   JEQ        = 0
 0  1  1   JGE        ≥ 0  (= JGT | JEQ)
 1  0  0   JLT        < 0
 1  0  1   JNE        ≠ 0  (= JGT | JLT)
 1  1  0   JLE        ≤ 0
 1  1  1   JMP        unconditional (always)
```

Again a bitmap: `(negative?, zero?, positive?)`. The ALU emits two status flags (`zr` = "is zero?" and `ng` = "is negative?"). The jump-control logic is a tiny combinational circuit: `should_jump = (j1 & ng) | (j2 & zr) | (j3 & !ng & !zr)`. When `should_jump = 1`, the PC takes A's value (the jump target, set by a preceding `@LABEL`); otherwise PC increments. This is precisely the `PC ← A if jump, else PC+1` logic from §5.3.1.

#### Worked example — line 5 of the sum-1..100 program

Source line: `D;JGT` (after `@END`, meaning "if D > 0, jump to END")

Decoding:
- Type bit = `1` (C-instruction)
- Filler `1 1`
- a = `0` (operate on A-register, not M)
- comp = `D`  → c1..c6 = `0 0 1 1 0 0`
- dest = (none) → d1 d2 d3 = `0 0 0` (discard the computed value)
- jump = `JGT` → j1 j2 j3 = `0 0 1`

Concatenated: `1 1 1 0 001100 000 001` = `1110 0011 0000 0001`. The book's listing confirms this exact bit pattern on line 5 of `Prog.hack`.

The non-obvious step: `D;JGT` *computes D*, throws the result away (`dest = null`), but **uses the result's sign** to drive the jump. This is the Hack idiom for "if D > 0 then goto" — you must compute the expression you want to test in the comp field, with no destination, just for its ALU sign-flag side effect. There is no separate "compare and branch" instruction in Hack; comparison is what you get for free with any C-instruction.

#### The four-module implementation

```
              ┌───────────────┐
   Prog.asm → │    Parser     │ — strips whitespace/comments
              │ - advance()   │ — splits each line into:
              │ - cmdType()   │     A_COMMAND / C_COMMAND / L_COMMAND
              │ - symbol()    │
              │ - dest()      │
              │ - comp()      │
              │ - jump()      │
              └───────┬───────┘
                      │
                      ▼
              ┌───────────────┐         ┌──────────────────┐
              │   Code        │         │  SymbolTable      │
              │ (3 lookup     │         │ - addEntry()      │
              │  tables)      │         │ - contains()      │
              │ - dest(m)     │         │ - getAddress()    │
              │ - comp(m)     │         │ - hashmap inside  │
              │ - jump(m)     │         └────────┬──────────┘
              └───────┬───────┘                  │
                      │                          │
                      └──────────┬───────────────┘
                                 ▼
                         ┌───────────────┐
              Prog.hack ← │     Main      │ — two-pass driver
                         └───────────────┘
```

Each module is small (<100 lines in Python/Java). The Code module is **stateless**: it's a function from mnemonic strings to bit strings, implemented as three dicts. The SymbolTable is **stateful** but uses the language's built-in hashmap. The Parser is the only module with non-trivial control flow (whitespace, comments, type-discrimination of lines starting with `@`, `(`, or anything else).

The two-stage build plan is pedagogical scaffolding: stage 1 = symbol-less assembler (≈30 lines of Main once Parser/Code exist) that handles only `@5` and C-instructions; stage 2 = add SymbolTable and the two-pass loop. Stage 1 verifies your encoding tables are right; stage 2 verifies your symbol-resolution logic is right. They fail in different ways and shouldn't be debugged together.

### If you were the assembler's Main driver…
You'd open `Prog.asm`, instantiate a Parser over it, walk every line once (Pass 1), advancing a `rom_counter` for every A/C-instruction and adding `(label_name, rom_counter)` to the SymbolTable on every `L_COMMAND`. You'd then *re-seek the parser to the beginning* (or build a Parser-2 over the same file) and walk again (Pass 2): for each A_COMMAND, get the symbol; if it's a number, encode it; if it's in the symbol table, encode its address; if it's a new variable, assign it the next RAM slot starting at 16, register it, encode that. For each C_COMMAND, ask Code for the three bit strings, concatenate `111 + a-bit + ccccccc + ddd + jjj`, and write the line. **The total source-state you carry between iterations is one integer (ram_counter), one int (rom_counter), and one hashmap.** No backtracking, no AST, no IR — assembly is the IR.

### Cross-language view
The Code module is the universal "lookup table" idiom and looks essentially the same in every language:

```python
# Python
COMP = {'0':'0101010', '1':'0111111', 'D':'0001100', 'A':'0110000', 'M':'1110000', ...}
def comp(m): return COMP[m]
```

```rust
// Rust
fn comp(m: &str) -> &'static str {
    match m {
        "0" => "0101010", "1" => "0111111", "D" => "0001100",
        "A" => "0110000", "M" => "1110000", /* ... */
        _ => panic!("unknown comp mnemonic"),
    }
}
```

```go
// Go
var comp = map[string]string{
    "0":"0101010", "1":"0111111", "D":"0001100",
    "A":"0110000", "M":"1110000", /* ... */
}
```

The shape is the same because **the problem is the same**: a finite enumeration mapped to a fixed-width bit string. There's no algorithmic content — only the table. In a production assembler (GNU `as`, NASM), the equivalent "table" is the **opcode encoding tables** auto-generated from instruction-set XML descriptions (`x86-64` has thousands of these; ARM uses ASL specs).

### Where this shows up in real systems
- **RISC-V's regularity** is the same design philosophy taken further: fixed 32-bit instruction width, opcode bits always in positions 0–6, register fields always in fixed positions across instruction formats (R/I/S/B/U/J). The RISC-V spec explicitly cites pedagogical regularity as a goal. Hack is the simpler precursor — same instinct.
- **x86's irregularity** is the opposite endpoint: variable-length instructions (1–15 bytes), opcode position varies, prefix bytes change semantics retroactively. The x86 decoder is the most complex sequential logic block in any modern CPU. The "regularity tax" Hack pays in unused filler bits (the `11` at positions 13–14) is what x86 traded away — and modern x86 cores spend transistors *re-decoding into a clean micro-op format* to undo that trade.
- **JIT compilers** emit instructions using exactly this lookup-table pattern at runtime — V8's TurboFan, JVM HotSpot's C2, LLVM's MC layer. The "Code" module from Hack is conceptually identical to `MachineCodeEmitter` classes in production JITs; only the table is bigger.
- **eBPF** in the Linux kernel uses a Hack-shaped fixed-width regular ISA (8-byte instructions, fixed fields) for exactly Hack's reason: the in-kernel verifier and JIT both need to decode trivially. eBPF is what Hack would look like if it survived to production.

### Diagnostic questions
1. **Why are bits 13 and 14 hard-wired to 1 in every C-instruction?** *They're unused — but had to be assigned a value to keep the type-bit at position 15 cleanly distinguishable from any A-instruction. The Hack designers chose `1` (rather than `0`) so a partially-decoded word that includes those bits still parses unambiguously as a C-instruction at any decoding stage.*
2. **What happens if a programmer writes `M=D+M` with `a=0` instead of `a=1`?** *They'd actually be writing `M = D+A` (since `a=0` selects the A-register for the second ALU operand). The assembler enforces this by mapping the mnemonic `D+M` to the `a=1` table column — the programmer never picks a-bit directly. The mnemonic *is* the (a-bit + c-bits) selector.*
3. **Why is `dest` a bitmap rather than an enum?** *Because Hack supports writing to A, D, and M simultaneously in one cycle — 8 combinations would need 3 enum bits anyway, but as an enum they'd require a decoder. As a bitmap they wire **directly** to the three register/RAM load-enable lines. Same bit count, less hardware.*
4. **What's the cost in bit-density of Hack's regularity?** *2 bits per C-instruction are wasted (bits 13–14), so ~6% of instruction-memory is overhead. The benefit is decode logic = ~10 gates instead of ~200. For a 32K-instruction ROM at 1971-era costs, this would be a no-brainer; today the bits are free but the decode-simplicity still matters in tight critical paths (V/U-pipe scheduling).*
5. **Could the assembler be single-pass without back-patching?** *Only if the language forbade forward label references — and Hack assembly explicitly allows them ("a label can be used anywhere... even before the line in which it is deﬁned"). Therefore the two-pass design isn't optional; it's a direct consequence of the language spec.*

### See also
- N2T 2026-05-23 entry — the **symbol resolution** layer that populates the `vvvvvvvvvvvvvvv` field of A-instructions. Today's entry is its bit-encoding complement.
- N2T 2026-05-22 entry — the **CPU hardware** that decodes these exact bit-fields at run time. The a-bit driving the ALU input MUX, the d-bits driving the load-enables, and the j-bits driving the PC's load signal are all visible in figure 5.9.
- COD §2.5 — RISC instruction formats (R/I/J types) — the production-grade version of the same regularity discipline.
- DDIA Ch.4 (Encoding and Evolution) — wire-format encoding has the same "type byte + payload" shape; lessons transfer.
- DBI 2026-05-24 entry — cell layout uses the same trick of putting fixed-size fields *before* variable-size payload so offsets are computable in O(1).

---

## [2026-05-23] The Symbol Table — How an Assembler Resolves Names to Addresses (and Why It Needs Two Passes) · pp.107–118 · Ch.5 §5.3–5.5 (CPU jump logic + Perspective) → Ch.6 Assembler §6.1 Background, Symbols, Symbol Resolution

### TL;DR
The bridge from hardware to software is the **assembler**, and the assembler's hard problem is not opcode translation — that's a table lookup — it's **symbol resolution**: turning programmer-written names like `loop`, `end`, `sum`, `i` into the integer memory addresses the hardware actually understands. The standard solution is a **two-pass design** built around a **symbol table** (a hashmap from name → address). Pass 1 walks the source code without emitting any output and learns where every label lives; Pass 2 walks it again and emits machine code, using the symbol table to resolve every reference. This is the first non-trivial translator pattern in the software hierarchy, and the exact same two-pass-with-symbol-table shape will return in the linker, the compiler's name resolver, and even in `dlsym()`.

### Intuition — "this is like…"
Imagine you're translating a mystery novel and the manuscript says *"see what Inspector Lestrade said in the previous chapter."* If you translate top-to-bottom in one pass, you'd have to either pause and flip ahead every time a forward reference appears, or guess. **Pass 1** is the translator first reading the whole book and writing down "Lestrade speech = page 84, paragraph 3" in a notebook (the symbol table). **Pass 2** is the translator now translating in order, glancing at the notebook every time a name appears. The reason a single pass doesn't work isn't laziness — it's that **forward references are undecidable until you've seen the whole program**.

### Mechanics

#### What the chunk actually covers
The chunk crosses a chapter boundary. The first half is the tail of Chapter 5 (already covered in depth on 2026-05-22): the **CPU jump logic** (`PC ← A if jump, else PC ← PC+1`), the **Memory chip** packaging (RAM16K + Screen + Keyboard into one address space), and the §5.4 **Perspective** comparing Hack to general-purpose CISC/RISC machines. The new conceptual territory begins at p.114 with Chapter 6, the **Assembler**.

#### The translator's input/output contract

```
   Assembly source                 Binary machine code
   ────────────────                ───────────────────
   @sum                            0000000000010000
   M=0              ───►  Hack ──► 1110101010001000
   (LOOP)             Assembler    (no output — label)
   @i                              0000000000010001
   D=M                             1111110000010000
   @LOOP                           0000000000000010   ← resolved to instruction #2
   D;JGT                           1110001100000001
```

Two things the assembler must do:
1. **Mnemonic → binary** for opcodes, dest fields, jump fields — pure table lookup, trivial.
2. **Symbol → address** for `@sum`, `@LOOP`, `(LOOP)` — requires the symbol table.

#### Two kinds of symbols (and why they're allocated differently)

| Symbol kind | Source form | Resolved to | Allocated when |
|---|---|---|---|
| **Label** (code anchor) | `(LOOP)` definition; `@LOOP` reference | **ROM address** = instruction number where the label was declared | Pass 1, the moment the `(LOOP)` is encountered |
| **Variable** (data slot) | `@sum`, `@i` (no `(…)` definition) | **RAM address**, starting at `0x0010` (= 16) and incrementing | Pass 2, the first time the variable is referenced |

The asymmetry is the core insight: **labels must be allocated in Pass 1 because they can be referenced before they're declared** (forward jumps). Variables have no forward-reference problem — the *first* mention is also the *declaration* — so they can be assigned lazily in Pass 2.

#### The two-pass algorithm in pseudo-code

```python
# Pass 1: build the symbol table, no output emitted
symbol_table = {predefined symbols: R0..R15, SCREEN=16384, KBD=24576, SP=0, ...}
rom_address  = 0
for line in source:
    if line is "(LABEL)":
        symbol_table[LABEL] = rom_address    # label points at the *next* instruction
    elif line is A- or C-instruction:
        rom_address += 1
    # blank lines & comments: skip, don't advance rom_address

# Pass 2: emit binary, resolving symbols on the fly
ram_address = 16                              # variables start above predefined R0..R15
for line in source:
    if line.startswith('@'):
        sym = line[1:]
        if sym.isdigit():
            addr = int(sym)
        elif sym in symbol_table:
            addr = symbol_table[sym]
        else:                                 # new variable
            symbol_table[sym] = ram_address
            addr = ram_address
            ram_address += 1
        emit A_instruction(addr)
    elif line is C-instruction:
        emit C_instruction(parse_dest, parse_comp, parse_jump)
    # labels emit nothing
```

The **`rom_address` invariant**: labels point at the address of the *instruction that follows them*. They do not occupy ROM space themselves — they are pure source-level annotations. That's why `rom_address` doesn't increment when a label is parsed.

#### Why this matters: forward references are the whole reason

```
  // Source                  // Pass 1 sees this  // Pass 2 emits this
  @LOOP        ← reference   rom=0  add nothing   emit @0000000000000010
  D;JGT                       rom=1  +1            emit C-instr
  ...                                              ...
  (LOOP)       ← declaration  rom=2  symbol[LOOP]=2  (nothing)
  @i                          rom=2  +1            emit @0000000000010000
```

When Pass 2 reaches `@LOOP` on line 1, it needs the value `2` — which wasn't known until line 5 was scanned. Without Pass 1, you'd have to **back-patch**: emit a placeholder, remember its location, fix it up later. That's a valid alternative (used by some single-pass assemblers and JITs), but it's strictly more complex than two-pass.

#### Predefined symbols — the "OS" hiding in the symbol table

Hack ships with ~23 built-in symbols pre-loaded into the symbol table before Pass 1 starts:
- `R0`..`R15` → addresses 0..15 (the general-purpose RAM scratch space)
- `SP=0`, `LCL=1`, `ARG=2`, `THIS=3`, `THAT=4` (VM stack pointers, used in Chapter 7)
- `SCREEN=16384` (`0x4000`), `KBD=24576` (`0x6000`)

These are the **memory-mapped I/O addresses from Chapter 5 §5.2**, now named. The connection: the hardware decided these regions exist; the assembler decides they have human-readable names. The OS chapter (Ch.12) will rest on the same names.

### If you were the assembler…
You're scanning a 1,000-line source file and you hit `@FOO` on line 12, but `(FOO)` isn't defined until line 800. What do you do? The naïve answer is "scan forward to find it" — but that's O(N²) per `@`-reference and the source might have many. The two-pass answer is to **separate symbol discovery from code emission**: pay one O(N) sweep to learn where every label lives, then a second O(N) sweep to emit, looking up each symbol in O(1). Total O(N), no back-patching, no rescans. The cost is reading the source twice and holding the symbol table in memory — a trade modern systems consider trivially worthwhile.

The deeper lesson: any time you have a **dependency between code that hasn't been seen yet and code that's being emitted now**, you have a choice of (a) two-pass, (b) back-patching, or (c) demanding declarations come before uses (the Pascal/C-89 prototype rule). Modern languages mostly pick (a) or (b) because forcing declaration-before-use is hostile to humans.

### Cross-language view
The symbol-table pattern is a near-universal:

| Tool | Symbol kind | Where the table lives |
|---|---|---|
| GNU `as` (assembler) | labels, externs | `symtab` section of the resulting `.o` ELF |
| `ld` (linker) | function & global names | merged across `.o`s; unresolved → "undefined reference" |
| `gcc` / `rustc` front-end | local & module names | scope-stacked symbol table during name resolution |
| Python's `compile()` | name → cell/free/global | `co_names`, `co_varnames`, `co_freevars` tuples on the code object |
| `dlsym(handle, "name")` | dynamic symbol → address | `.dynsym` section + GOT/PLT at load time |

The **same algorithm** scales from a 200-line Hack assembler to a multi-megabyte linker resolving millions of symbols across thousands of object files. The data structure may be a hashmap, a hashmap-per-scope, or a B-tree, but the conceptual shape — *name → address, populated in one pass, queried in another* — does not change.

### Where this shows up in real systems
- **ELF relocation entries** (`.rela.text`) are the production-grade version of back-patching: the compiler emits `0x00000000` placeholders for any symbol it can't resolve and a side-table of "fix this offset to point at symbol X." The linker walks the relocation table at link time and patches the bytes — exactly the same problem the Hack assembler dodges by having the whole program in one file.
- **JIT compilers** (V8, JVM Hotspot, LLVM ORC) cannot do two passes because they emit code as they go. They use back-patching: emit a `jmp 0`, remember the address, fill it in when the target is generated. This is why JITs have *trampolines* — they're forward-reference placeholders that get rewritten on first call.
- **The dynamic linker** (`ld.so`) does symbol resolution *at process start time*, walking each shared library's `.dynsym` and binding names to runtime addresses via the GOT/PLT. The same conceptual two-pass — discovery, then resolution — is now spread across compile-time, link-time, and load-time.

### Diagnostic questions
1. **Why can variables be allocated lazily in Pass 2, but labels cannot?** *Wrong answer: "Variables don't have forward references." → They do — `@i` can appear before `i` is "used" anywhere meaningful. The right reason is that **all variable mentions are equivalent** (no declaration vs. use distinction), so the first encounter is as good as any.*
2. **A user writes `@5` instead of `@variable`. Does the symbol table grow?** *No — numeric A-instructions bypass the symbol table entirely. This is why `@16` and `@SCREEN` produce identical binary even though only one uses the symbol table.*
3. **What happens if a label is defined twice?** *The Hack spec is silent, but conventional assemblers either reject (`gas`) or silently use the last definition. Real assemblers add a third "duplicate-detection" pass or use `set vs equ` semantics.*
4. **Why does the variable region start at RAM address 16, not 0?** *Addresses 0–15 are pre-claimed by `R0..R15` (the VM virtual registers). The assembler must not collide with them, so the watermark starts above.*
5. **Could you write a single-pass Hack assembler?** *Yes, via back-patching: emit each `@LABEL` as a 16-bit placeholder, remember the file offset, fix it when `(LABEL)` is later seen. The cost is mutable output and a fix-up list — strictly more complex than two passes.*

### See also
- N2T 2026-05-22 entry — the CPU implementation this assembler emits code for (the jump-logic mechanism `PC ← A if jump, else PC+1` is exactly what `D;JGT` triggers).
- N2T Ch.5 §5.4 (Perspective) — von Neumann vs. Harvard, CISC vs. RISC; Hack is Harvard + RISC-ish, which is *why* the assembler stays this simple (no instruction-length variance to track in `rom_address`).
- Forward link: N2T Ch.7 (VM Translator) and Ch.10–11 (Compiler) will both reuse the symbol-table pattern at higher levels (VM symbols → assembly, Jack identifiers → VM).
- DDIA Ch.3 *SSTables and LSM-Trees* — a different kind of "symbol table" (key → file offset), but the same lookup-after-build shape.

---

## [2026-05-22] The Hack Computer — von Neumann Anatomy, Registers, Memory-Mapped I/O, and the Fetch-Execute Loop · pp.95–106 · Ch.5 Computer Architecture (§5.1 The von Neumann Architecture → §5.2 The Hack Hardware Platform Spec → §5.3.1 CPU Implementation Sketch)

### TL;DR
Chapter 5 closes the loop on the bottom-up build: the ALU (Ch. 2), the registers + RAM (Ch. 3), and the machine language (Ch. 4) are now wired into a single **von Neumann machine** with one address space, three CPU registers (D, A, PC), and two memory-mapped peripherals (Screen, Keyboard). Each clock cycle runs the canonical **execute-then-fetch** sequence: ALU output and control bits flow from the *current* instruction simultaneously, while the PC is updated for the *next* one — both in the same tick. The chapter's pedagogical punchline is that **memory-mapped I/O** lets the CPU stay completely ignorant of peripherals: every keypress, every pixel, is a `M[addr]` read or write — the same instruction that touches RAM.

### Intuition — "this is like…"
A CPU is a tiny **assembly line whose conveyor belt is the clock**. Each tick, the current item (instruction) is acted upon by every station at once — ALU, registers, multiplexers — and at the *end* of the tick a single sequential element (the PC) advances to the next item. The peripherals (screen, keyboard) aren't special stations; they're more conveyor belts that happen to be wired so writing to their addresses *lights up pixels*, and the operator pressing a key *writes a value into a slot*. The CPU never knows the difference.

### Mechanics

#### The von Neumann triangle
```
        +---------------------+
        |   Instruction ROM   |  (Hack: 32K × 16-bit, read-only)
        +----------+----------+
                   |  ROM[PC]
                   v
        +---------------------+
        |        CPU          |     <-- D, A, PC registers + ALU + control
        +----+-----+-----+----+
       inM   |  outM,addrM,writeM
             |     |
             v     v
        +---------------------+
        |    Data Memory      |  (Hack: RAM + Screen map + Keyboard map)
        +---------------------+
```
The **single bus** to data memory carries both regular RAM traffic *and* I/O traffic. The CPU only knows it issued an address — the memory chip itself routes the request:

| Address range (hex) | Size | Routed to |
|---|---|---|
| `0x0000–0x3FFF` | 16K | General RAM |
| `0x4000–0x5FFF` | 8K | **Screen** memory map |
| `0x6000`        | 1   | **Keyboard** memory map |
| `> 0x6000`      | —   | Invalid |

#### The three Hack registers
| Reg | Width | Role | Notes |
|---|---|---|---|
| **D** | 16 | Pure data scratchpad | Cannot be used as an address |
| **A** | 16 | Triple-purpose: data **or** ROM address **or** RAM address | Context-dependent — the *instruction* decides |
| **PC** | 15 | Program counter | Always points into ROM; outputs feed ROM's address pin |

The **A register's overloading** is the trick that keeps the instruction format at 16 bits. Most ISAs need extra bits to say "this is a memory address, not data"; Hack instead says "if you want to touch memory, *first* put the address in A with an A-instruction, *then* refer to it as `M`." Two instructions to do what x86 does in one, but the encoding stays tiny.

#### One clock cycle (the execute-then-fetch dance)
```
clock tick t:
  PC -> ROM -> instruction[16]         (combinational, no delay)
  instruction routed in parallel to:
     - ALU control bits (a, cccccc)    -> ALU computes
     - destination bits (ddd)          -> A, D, M load enables
     - jump bits (jjj) + ALU flags     -> jump decision
  edge of tick t (clocked elements latch):
     - D, A latch new values if ddd says so
     - RAM[A] latches outM if writeM=1
     - PC latches:  A   if jump triggered
                    PC+1 otherwise
                    0   if reset=1
```
Combinational signals settle *during* the tick; sequential elements (D, A, PC, RAM) commit *at* the tick edge. Reading this as "execute then fetch" is shorthand — physically they overlap, and that overlap is the whole reason single-cycle CPUs are fast.

#### Memory-mapped I/O — the screen, dissected
- Screen RAM: 8K words = `8192 × 16 bits = 131,072` pixels.
- Physical screen: 256 rows × 512 cols = **131,072 pixels**. (Match — by design.)
- Pixel at `(row r, col c)` lives at bit `c % 16` of word `Screen[r*32 + c/16]`.

So drawing a horizontal line is 32 word writes (one per 16-pixel chunk); a vertical line is 256 word *read-modify-writes* (you must preserve the other 15 bits of each word). This asymmetry — **rows are cheap, columns are expensive** — is a real cost any Hack OS-level routine has to internalize.

#### Why the A register is overloaded
| A used as… | Triggering instruction | Effect |
|---|---|---|
| **15-bit constant** | `@n` (A-instruction `0vvvvvvvvvvvvvvv`) | A ← n |
| **Data value** | C-instr like `D=A` | Treats A's bits as the operand |
| **RAM address** | Any C-instr referencing `M` (`M=...`, `D=M`, `D=D+M`) | `M` resolves to `RAM[A]` |
| **ROM address (jump target)** | C-instr with jump bits ≠ 000 | If jump fires, PC ← A |

This single register, four meanings: the densest encoding trick in the book.

### If you were the CPU…
You wake up on tick *t*. The instruction memory hands you 16 bits — you don't get to "fetch" anything; the ROM is already shouting `ROM[PC]` into your face combinationally. You look at bit 15: if it's `0`, you're in the easy case — latch bits 0–14 into A at the tick edge and bump PC by 1. If it's `1`, you fan out the rest of the bits as control signals: bit 12 (`a`) tells the ALU whether to read its second operand from A or from M; bits 11–6 (`cccccc`) select one of the ALU's functions; bits 5–3 (`ddd`) flip A/D/writeM load-enables; bits 2–0 (`jjj`) — combined with the ALU's `zr`/`ng` flags — decide whether the next tick's PC will be `A` or `PC+1`. You never deliberate; every wire settles in parallel during the tick, and the clock edge picks a winner.

### Where this shows up in real systems
- **Memory-mapped I/O is universal**: PCIe devices expose BARs (Base Address Registers) that the OS maps into the physical address space; a `mov` to that range becomes a transaction on the PCIe bus. ARM Cortex-M MCUs do peripheral control entirely this way (no separate `IN`/`OUT` instructions like x86). The Linux `/dev/mem` interface lets userland do exactly what Hack does — store a word, change the hardware state.
- **Harvard vs. von Neumann**: Hack is technically **modified Harvard** (separate ROM and RAM buses) but presents a unified programming model. Real CPUs split L1I/L1D caches (Harvard inside the chip) over a unified main memory (von Neumann outside) — same trick, larger scale.
- **The "A register" pattern is the `lea`/RIP-relative addressing of x86 and the `adr`/`adrp` of ARM**: load an address into a register first, then operate. RISC-V is even more explicit — there's no instruction that takes both an address *and* a complex operand in one go.

### Diagnostic questions
1. **Why can't you write `D = RAM[100]` in a single Hack instruction?**
   *Wrong answer interpretation*: "Because the ISA is RISC-y" — partially true but misses the point. The real reason is the **16-bit instruction budget**: encoding a 15-bit address *and* a destination *and* an ALU op in 16 bits is impossible, so addresses are factored out into the A register via a preceding A-instruction.
2. **If `Screen[r*32 + c/16]` holds 16 pixels, why is drawing a *vertical* line slower than a *horizontal* line?**
   *Wrong answer interpretation*: "Because the screen is wider than tall" — no, it's because vertical lines touch a *different word per row* (256 read-modify-writes), while horizontal lines fit in 32 full-word writes.
3. **The PC is 15 bits, but D and A are 16. Why the mismatch?**
   *Wrong answer interpretation*: "Saving silicon" — really because the ROM has 32K = 2¹⁵ addresses; one more bit would address ROM cells that don't exist. The data registers must be 16 bits to hold full instruction operands and ALU outputs.
4. **What happens if a program writes to address `0x7000`?**
   *Wrong answer interpretation*: "It writes to RAM" — no, the Memory chip's contract says addresses > `0x6000` are **invalid** (undefined behavior in the spec). A real chip might wrap, alias, or fault.
5. **Why does the spec say "execute then fetch" when in fact both happen in the same cycle?**
   *Wrong answer interpretation*: "Because the textbook is sloppy" — it's a teaching convenience. Logically, the *current* instruction's effect must be visible *before* the next PC is chosen, even though physically the combinational logic for both runs in parallel within one clock tick.

### See also
- [[n2t-2026-05-21-hack-machine-language]] — the Hack ISA laid out in Ch. 4; this chapter now hands those instructions to actual silicon.
- [[cod-2026-05-11-pipelining]] — COD's 5-stage pipeline accelerates this single-cycle baseline by overlapping fetch/decode/execute across instructions.
- When OSTEP introduces `mmap` and `/dev/*` device files, the underlying mechanism is the same memory-map trick generalized to a virtual address space.

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

