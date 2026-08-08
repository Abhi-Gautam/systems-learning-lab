# shared-mem

Learning how two processes share one physical page via the page tables.

1. `mmap(MAP_SHARED)` maps a physical frame into two processes' address spaces.
2. Same physical page, different virtual addresses — kernel inserts one frame twice.
3. `shm_open` + `mmap` is the portable named shared-memory path.
4. `memfd_create` gives an anonymous fd-backed region, inheritable across fork.
5. `fork()` uses copy-on-write: pages shared until one process writes.
6. Prove sharing with /proc/<pid>/pagemap: same PFN, different virtual address.
7. `SM=SHM` in vmmap marks the region as shared physical memory.
8. A mutex in shm must be `PTHREAD_PROCESS_SHARED` or it is useless across processes.
9. After setup, cross-process access costs ~1-50 cycles (no copy, TLB-dependent).
10. Experiments: writer/reader demo, COW fork, pagemap PFN inspection.
