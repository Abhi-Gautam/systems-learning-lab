# scheduler

Learning how the kernel parks and wakes threads around a futex.

1. A blocked (futex) thread is descheduled: core picks the next runnable task.
2. The thread's register state is saved; its task leaves the run queue.
3. Wake (FUTEX_WAKE) re-enqueues it; scheduler may run it on another core.
4. A context switch then costs the CR3/TLB/register tax from processes-threads.
5. `SCHED_FIFO`/`SCHED_RR` change preemption; `sched_yield` volunteers the core.
6. `TASK_INTERRUPTIBLE` (R in ps) vs `TASK_UNINTERRUPTIBLE` (D) visibility.
7. `taskset`/`numactl` pin threads to show core migration on wake.
8. Priority inversion: low thread holds lock, high thread blocks (needs priority boost).
9. `sched_getaffinity` shows which CPUs a thread may run on.
10. Experiments: block a thread, watch State in /proc/<pid>/status, core migration.
11. Context switch cost: ~1,500-8,000 cycles (0.5-3 μs) — kernel saves ALL regs, FPU, switches page tables, TLB flush.
12. Green thread yield: ~150-200 cycles (50-80 ns) — runtime saves ONLY callee-saved regs (rbp, rbx, r12-r15, rsp, rip), no kernel entry.
13. M:N scheduler: runtime pins N OS threads to N cores, multiplexes M green threads; work-stealing + timer preemption (Go) or budget (Tokio).
