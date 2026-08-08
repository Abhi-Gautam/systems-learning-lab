# futex

Learning the kernel slow-path that makes a contended lock cheap.

1. Fast path: atomic CAS in userspace grabs the lock, no syscall (~15ns).
2. Slow path: on contention, the thread calls `futex(FUTEX_WAIT)` and sleeps.
3. The kernel hashes (mm, addr) into a bucket to find the right wait queue.
4. Lost-wakeup is prevented: check `*addr==expected` and enqueue are one op.
5. `FUTEX_WAKE` dequeues one waiter and marks it runnable (O(1) by hash).
6. `TASK_INTERRUPTIBLE` waiters can be killed by a signal (returns EINTR).
7. `TASK_UNINTERRUPTIBLE` waiters (disk I/O) cannot be interrupted.
8. Futex syscall costs ~500ns-2us; far cheaper than spinning a core forever.
9. `strace`/`dtrace` shows the futex() syscall firing only under contention.
10. Experiments: contended mutex under strace, atomic-vs-syscall latency bench.
11. Go/Tokio don't use futex directly — they use netpoll (epoll/kqueue/io_uring) + timers + self-parking queues; futex only for std::sync::Mutex/Once.
