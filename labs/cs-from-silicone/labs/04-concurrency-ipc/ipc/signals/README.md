# ipc-signals

Learning async one-bit notification between processes and threads.

1. A signal is a kernel-set pending bit delivered to a process or thread.
2. `kill(pid, sig)` sends to a process; the handler runs in that process's context.
3. Threads share one set of signal dispositions (sa_handler) per process.
4. A signal sent to a process is delivered to one arbitrary thread.
5. `sigaction` installs a handler; `SIG_DFL`/`SIG_IGN` are the defaults.
6. Handlers must be async-signal-safe: no malloc, printf, or locks inside.
7. `SIGKILL`/`SIGSTOP` cannot be caught or blocked — hard kernel boundaries.
8. Signals carry no payload beyond the number; use them for events, not data.
9. `pause()`/`sigsuspend` let a thread sleep until a signal arrives.
10. Experiments: kill between processes, per-thread handler, async-safe violation.
