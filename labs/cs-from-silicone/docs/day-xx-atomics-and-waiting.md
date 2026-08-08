# Day X — Atomics (Silicon) and the Four Waiting Primitives

## Concept Ladder

```text
variable = just RAM bytes; silicon guarantees nothing about it
→ plain counter++ = load, add, store (3 instructions, NOT atomic)
→ MESI keeps cache lines coherent but does NOT make RMW atomic → lost updates
→ atomic instruction (LOCK XADD / ldaxr+stlxr) holds cache line exclusive for the
  whole read-modify-write → indivisible
→ coherence (MESI) ≠ atomicity: coherence stops stale reads, atomicity stops
  interleaved RMWs
→ atomic scope = ONE memory location only; cannot protect a multi-step block
→ spinlock = CAS 0→1 in a while loop; burns 100% CPU spinning
→ futex = userspace atomic CAS fast-path + kernel sleep/wake slow-path
→ mutex = futex word, binary (0/1), OWNED (only locker unlocks)
→ semaphore = futex + counter (N permits), ownerless; caps concurrency or signals
```

## Atomics (Silicon Level)

A variable is RAM. The CPU makes no promise about it. Safety comes from the
*instruction*, not the variable.

```text
plain counter++:   ldr w0,[x] → add w0,#1 → str w0,[x]     (3 steps, can interleave)
atomic add:         lock xadd [x], reg   (x86)              (1 step, line held M)
                   ldaxr/ stlxr retry     (ARM64)           (exclusive monitor)
```

MESI juggles one cache line at a time. Coherence prevents a core from reading a
*stale write*; it does NOT prevent two cores from both loading before either
stores. Atomicity (the LOCK/exclusive instruction) is what closes that gap.

## The Four Waiting Primitives — Algorithm

| | ENTER | EXIT | When busy |
|---|---|---|---|
| Spinlock | CAS 0→1, loop if fail | set 0 | spin (burn CPU) |
| Mutex | CAS 0→tid, sleep if fail | set 0 + wake | sleep (0% CPU) |
| Semaphore | if count>0: count--; else sleep | count++ + wake | sleep (0% CPU) |

- **Spinlock**: CAS makes the check+flip one uninterruptible step so exactly one
  thread wins 0→1. Waiter busy-loops, burning a full core.
- **Futex**: CAS fast-path grabs it with no syscall; on failure the thread asks
  the kernel to SLEEP on the object's wait queue (0% CPU). Lost-wakeup prevented
  because "is it still 0?" and "enqueue me" are one uninterruptible kernel op.
- **Mutex**: futex + ownership. Only the locker may unlock → catches bugs.
- **Semaphore**: futex + a counter of N permits; ownerless (any thread can post).

## Stack

```text
atomic (silicon: LOCK / ldaxr-stlxr)
  → spinlock (CAS loop, spins)
    → futex (sleep)
      → mutex (exclusion) / semaphore (N-limit)
```

## Cross-Boundary Notes

- Cross-process mutex must live in `mmap`'d shared mem + `PTHREAD_PROCESS_SHARED`
  (or named sem). Same futex algorithm, different setup.
- Green threads: MESI only bites if they land on different OS threads/cores. Same
  core = no coherence juggling, but scheduler interleaving still races.
- macOS uses Mach semaphores (semaphore_wait/signal via psynch) instead of Linux
  futex — same concept, different syscall name.

## Stop When You Can Say

```text
The variable is not atomic; the instruction is.
MESI coherence is necessary but not sufficient for correctness.
Atomic protects one location; a lock protects a block; futex makes the wait cheap.
Spin burns CPU; mutex/semaphore sleep; semaphore counts, mutex owns.
```
