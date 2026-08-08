# ipc-pipe

Learning the simplest kernel channel between two processes.

1. `pipe()` creates a kernel ring buffer with a read fd and a write fd.
2. `write(fd)` copies userspace buffer into the kernel buffer (~1-5k cycles).
3. `read(fd)` copies it back out into the reader's userspace buffer.
4. The data crosses the user-kernel boundary twice — unlike shared memory.
5. Reader blocks (sleeps) until the writer writes; built-in synchronization.
6. Writer blocks when the kernel buffer is full (bounded capacity).
7. A pipe is a byte stream: no message boundaries, partial reads happen.
8. After `fork()`, parent and child share the fds and can talk both ways.
9. Threads never use pipes: they already share memory, it would be a detour.
10. Experiments: parent->child pipe, measure two copies, observe blocking.
