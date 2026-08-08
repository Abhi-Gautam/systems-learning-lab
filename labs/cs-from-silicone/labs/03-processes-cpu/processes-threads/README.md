# processes-threads

Learning the silicon/OS distinction between a process and a thread.

1. A process owns an isolated virtual address space; a thread shares it.
2. Page tables (CR3 x86 / TTBR0 ARM64) are per-process; threads share one.
3. `getpid` is identical for all threads; `gettid` is unique per thread.
4. Thread switch = register swap only; process switch = CR3 swap + TLB flush.
5. Process switch costs ~2-3k cycles; thread switch ~1-1.5k (TLB stays hot).
6. File descriptors, signal handlers, and `mm_struct` are shared per process.
7. Two threads see the same virtual address as the same physical page.
8. Two processes see the same virtual address as different physical pages.
9. Pin threads to cores with `sched_setaffinity` to observe coherence directly.
10. Experiments: read /proc/self/maps, getpid/gettid, measure switch cost.
11. Each OS thread has a 16-32 KB kernel stack (pinned, non-pageable) — 100k threads = 1.6-3.2 GB kernel memory.
