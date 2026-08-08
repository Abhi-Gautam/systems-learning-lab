# green-threads

Learning M:N scheduling and when cache coherence actually bites.

1. Green threads are user-space; a runtime maps M of them onto N OS threads.
2. Two green threads on one OS thread share one core: no MESI juggling at all.
3. The danger there is scheduler interleaving mid-RMW, not cache coherence.
4. Spread across OS threads on different cores, MESI juggling returns exactly.
5. A minimal scheduler round-robins green threads on a single core (cooperative).
6. Preemption yields the core; a ring buffer holds the ready queue of contexts.
7. Pinning the runtime to one core means green threads never pay RFO/TLB cost.
8. Under load the runtime spreads them; then atomics/mutex become mandatory.
9. Go/Tokio hide this; the lab makes the M:N mapping explicit and observable.
10. Experiments: 1-core ring scheduler, then spread across cores, observe races.
11. Kernel stacks are 16-32 KB fixed per OS thread; green threads use 2 KB growable user stacks — 1000x density.
12. Context switch: OS thread = ~1-3 μs (kernel, TLB, scheduler); green thread = ~20-100 ns (user-space register swap).
