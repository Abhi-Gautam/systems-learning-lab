# AGENTS.md

## Project Intent

This folder is the silicon/runtime learning track. Its goal is to build a
bottom-up machine model: physical storage and signals first, then CPU/register
behavior, then operating-system/runtime behavior, and language concepts last.

## Repository Shape

- `core/` holds the canonical no-dependency Rust lab runner.
- `docs/` holds curriculum specs, day guides, and milestone notes.
- `labs/` holds isolated experiments and alternate designs.
- `references/` holds notes from books or upstream systems used for study.
- `external/` holds local read-only reference material, if needed later.
- `bench/` holds optional benchmark notes and timing captures.
- `tools/` holds helper scripts, if needed later.

## Current Learning Path

- Day 1 starts in `docs/day-01-bits-bytes-words.md`.
- The full pilot spec is `docs/curriculum.md`.
- The runnable artifact is `core/src/main.rs`.
- Days 1-5 are the active pilot block.
- Days 6-30 are runnable experiments, but their curriculum order is pending
  rewrite after the first block.

## Working Rules

- Reuse `docs/curriculum.md` and `core/src/main.rs` before introducing new lab
  structures.
- Keep labs small, runnable, and no-dependency unless a later plan explicitly
  chooses otherwise.
- Keep the canonical learning path small and readable.
- Put experimental or alternative implementations behind clear boundaries in
  `labs/`.
- Preserve the teaching ladder:

```text
physical thing
→ CPU/ISA view
→ OS/runtime view only when needed
→ Rust/C/JS concept last
```

## Runnable Lab Contract

- Each lab's source is the lesson: explain mechanism, expected observations,
  and interpretation in concise comments immediately beside the relevant code.
  A learner must be able to read, run, edit, and learn from the artifact without
  relying on chat.
- Keep the README's existing numbered coverage list intact. It is the lab's
  curriculum contract; add a run command if needed, but never replace the list
  with prose or a summary.
- In chat, report only the artifact path, exact command, and the next concrete
  edit/observation. Do not restate the inline lesson as a wall of text.

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
