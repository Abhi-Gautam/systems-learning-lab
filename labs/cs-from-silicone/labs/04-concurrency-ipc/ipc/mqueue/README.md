# ipc-mqueue

Learning kernel-backed message queues with priority and boundaries.

1. POSIX mqueues (`mq_open`) live in the kernel, like pipes but message-oriented.
2. Each `mq_send` is one message; `mq_receive` gets whole messages, not bytes.
3. Messages carry a priority; the queue dequeues highest priority first.
4. Unlike pipes, there is no partial read and no byte-stream merging.
5. Capacity is bounded by `mq_maxmsg` x `mq_msgsize`; sender blocks when full.
6. `mq_notify` can request a signal or thread notification on message arrival.
7. SysV queues (`msgget`) are older, similar, but less portable than POSIX.
8. More structured than pipes, lighter than sockets for local message passing.
9. Still copies userspace->kernel->userspace, so not zero-copy like shm.
10. Experiments: priority send/receive order, blocking, mq_notify signal.
