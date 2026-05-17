# AGENTS.md

## Project Intent

This folder is the silicon/runtime learning track. Its goal is to build a
bottom-up machine model: physical storage and signals first, then CPU/register
behavior, then operating-system/runtime behavior, and language concepts last.

## Working Rules

- Reuse the existing top-level memory lab material before introducing new lab
  structures.
- Keep labs small, runnable, and no-dependency unless a later plan explicitly
  chooses otherwise.
- Preserve the teaching ladder:

```text
physical thing
→ CPU/ISA view
→ OS/runtime view only when needed
→ Rust/C/JS concept last
```

## Deep Explanation Mode

When the user brings a silicon/runtime concept they did not understand, answer
without changing code or files unless explicitly asked.

First decide whether the prerequisite has been covered in the current learning
path.

- If covered, answer directly with concrete low-level mechanics.
- If not covered, explain the missing prerequisites from scratch before using
  advanced terms.
- Do not mention mechanisms like `Smi`, `HeapNumber`, `deopt`, `inline cache`,
  pointer tagging, stack slots, registers, heap allocation, or bytecode without
  explaining what they are, where they live, why they exist, and what changes
  step by step.

Use this answer shape:

```text
source code
→ parser/compiler representation
→ bytecode/interpreter or compiled code
→ runtime value representation
→ stack/register/heap location
→ mutation/type-change cases
→ cost model
```

The expected depth is a causal trace. For example, `var x = 8; print(x)` in V8
requires explaining how the source becomes a binding, how the value is
represented, where the representation sits, what the generated code assumes,
what changes if `x` later becomes a double/string/object, and which costs appear
at each transition.
