# ipc-socket

Learning process-to-process messaging with message boundaries.

1. Unix domain sockets are local (no network stack) but cross process boundary.
2. Unlike pipes, they are datagram or connection-oriented with message edges.
3. `SOCK_DGRAM` preserves message boundaries; `SOCK_STREAM` is byte-oriented.
4. TCP localhost reuses the same code path but adds protocol + checksum cost.
5. `socketpair()` gives two connected fds, often used parent-child like a pipe.
6. Sockets are slower than shared memory (copy + syscall) but isolated/safe.
7. Cross-host communication is identical API; only the address family changes.
8. `sendmsg`/`recvmsg` can pass file descriptors over Unix sockets (SCM_RIGHTS).
9. Compared to pipe: bidirectional, connection-oriented, no forced byte-stream.
10. Experiments: Unix socket echo server, latency vs pipe, fd passing demo.
