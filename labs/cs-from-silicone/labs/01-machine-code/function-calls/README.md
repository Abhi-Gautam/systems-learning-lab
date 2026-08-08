# Function Calls — from the silicon up

Learning what actually happens when one function calls another, at the ISA level.

1. A call moves the argument values into registers (w0, w1 on ARM64; rdi, rsi on x86).
2. `bl _add` (branch with link) jumps to the callee and stores the return address in lr.
3. The callee builds a stack frame: `sub sp, sp, #16` reserves locals; `stp x29,x30` saves fp/lr.
4. `ret` jumps back to the saved lr; the caller's frame is restored by `ldp x29,x30`.
5. `-O0` spills every local to the stack (str/ldr pairs); the frame is explicit and large.
6. `-O2` drops the frame entirely: `add w0, w1, w0; ret` — the call is nearly free.
7. `call.o2.s` shows the optimizer inlining `twice_add` and `add` into one instruction.
8. Register conventions (caller-saved vs callee-saved) are the contract between functions.
9. A deep call chain grows the stack downward; overflow is the classic stack-smash bug.
10. Experiments: compile call.c at -O0/-O2, diff the .s, watch the frame appear and vanish.
