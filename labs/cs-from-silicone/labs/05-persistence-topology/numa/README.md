# numa

Learning physical topology cost when threads span sockets.

1. A multi-socket machine has separate DRAM per socket; access is not uniform.
2. Local DRAM access ~100ns; remote (cross UPI/QPI) ~200ns+ on modern servers.
3. A thread's memory should live on its socket: `numactl --membind` controls it.
4. `libnuma` (`numa_alloc_onnode`) allocates on a specific node explicitly.
5. Cross-socket futex contention hits a remote bucket lock: extra latency.
6. First-touch policy: memory is allocated on the node that first writes it.
7. Wrong placement silently halves bandwidth and doubles latency (NUMA bounce).
8. `numactl --hardware` shows nodes, cpus, and distances on your machine.
9. Cache coherence across sockets traverses the interconnect, not just L1-L3.
10. Experiments: local vs remote allocation latency, cross-node futex cost.
