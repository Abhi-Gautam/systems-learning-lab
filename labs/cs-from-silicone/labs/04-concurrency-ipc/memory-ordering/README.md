# memory-ordering

Learning why a correct lock still breaks without acquire/release ordering.

1. The compiler and CPU may reorder independent loads/stores for speed.
2. x86 is TSO: only StoreLoad reorders; ARM64 can reorder almost anything.
3. A mutex embeds acquire (on lock) and release (on unlock) to fence the block.
4. Without ordering, another thread may see writes inside the critical section early/late.
5. `memory_order_relaxed` is a bare atomic with no synchronization at all.
6. `memory_order_acquire`/`release` form a happens-before edge between threads.
7. `std::atomic` + seq_cst is the default full fence; costs more on weak memory models.
8. `sfence`/`dmb` are the hardware fences; `mfence` on x86 is heavy.
9. Litmus tests (StoreLoad, MsgPassing) prove reordering empirically.
10. Experiments: reorder demo on x86 vs ARM, acquire/release vs relaxed race.
