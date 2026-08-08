# AGENTS.md

## Project Intent

This repository is a long-horizon database learning project.

The goal is not to ship a feature-complete database quickly. The goal is to
build deep intuition about database internals through small, usable,
well-bounded implementations that can evolve over time.

## Working Rules

- Always try to reuse as much code and structure as possible before introducing
  new abstractions.
- Before implementing anything, inspect what already exists and explain whether
  it should be reused, adapted, or replaced.
- Prefer clean educational architecture over feature breadth.
- Keep the canonical learning path small and readable.
- Put experimental or alternative implementations behind clear boundaries.
- PostgreSQL and DuckDB are reference codebases for study. They are not copied
  into the core implementation.

## Repository Shape

- `core/` holds the canonical educational engine.
- `labs/` holds experiments and alternate implementations.
- `references/` holds notes about upstream systems and design learnings.
- `docs/` holds specs, plans, and milestone notes.
- `external/` holds local read-only upstream clones for reference.

## Collaboration Notes

- Come back with recommendations and tradeoffs before significant new
  implementation work.
- When styling work exists in this repo, always inspect `global.css` or
  `globals.css` before making styling changes.

## Deep Explanation Mode

The user has two database learning modes:

1. **Day-by-day lab mode**: follow the planned database internals curriculum.
2. **Concept/doubt mode**: the user brings a database concept from reading that
   did not make sense. In this mode, answer as a teacher and do not change code,
   files, notes, or lab artifacts unless explicitly asked.

For concept/doubt mode, first decide whether the prerequisite has been covered
in the current learning path.

- If covered, answer directly but still use concrete storage mechanics.
- If not covered, explain the missing prerequisites from scratch before using
  the term.
- Do not mention mechanisms like `overflow page`, `slot directory`, `WAL`,
  `buffer pool`, `page cache`, `LSM`, `compaction`, or `MVCC` without unpacking
  what they are, why they exist, where they live, and what changes step by step.
- Prefer causal traces over summaries.

Use this answer shape:

```text
logical operation
→ encoded bytes
→ record/cell layout
→ page layout
→ page read/write path
→ overflow/chunk behavior if relevant
→ buffer/cache/WAL implications
→ cost model
```

A weak answer says "Large documents may use overflow pages." A good answer
explains why a normal page cannot hold the document, what remains in the
original page, how overflow/chunk pages are linked or addressed, which pages
must be read for a tiny field access, what gets rewritten on update, and why
locality helps seeks but not bytes read, copied, parsed, or rewritten.
