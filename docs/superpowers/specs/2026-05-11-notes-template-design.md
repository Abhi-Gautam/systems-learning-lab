# Daily Notes Template — Design

**Date:** 2026-05-11
**Context:** Reading scheduler produces a daily chunk (book + page range + chapter) for each of 4 processors. Each chunk yields one notes entry. This spec defines the contract for that entry.

## Goal

Notes that (a) lose no important detail from the source chapter, (b) anchor intuition before mechanics, (c) bridge textbook content to modern systems engineering, and (d) actively test internalization. Optimized for re-scan months later, not for first-write efficiency.

## Scope

- **One file per book**: `Notes/{alias}-notes.md`.
- **One entry per daily session**, inserted at the **top** of the file (immediately after the file header). Newest entry is always the first thing visible when you open the file.
- Prior entries are append-only-in-spirit: they scroll downward as new ones land, but are never edited.
- If a session covers an already-in-progress concept, write a new dated entry — do not edit prior entries.
- Existing notes archived to `Notes/archive/2026-05-11/` before reset.

## Entry template

```markdown
---

## [YYYY-MM-DD] {Topic Title} · pp.X–Y · {Ch.N § N.M}

### TL;DR
3 sentences. Written last, placed first.

### History — "why does this exist?"
3–4 lines. Problem-before, breakthrough (year + system/person), what it displaced.

### Intuition — "this is like…"
One short analogy paragraph.

### Mechanics
ASCII diagrams, formulas in code blocks, **bolded keywords**,
multi-framing (prose + table + flowchart), trade-off boxes,
worked numerical example if quantitative.

### If you were the {agent}…              [conditional]
Active-reasoning prompt + textbook's answer. Two paragraphs max.

### Cross-language view                    [conditional]
Side-by-side Rust / Go / Python (and C when systems-relevant).
"What the stdlib actually does" note.

### Where this shows up in real systems    [mandatory]
2–3 bullets connecting textbook → production code.

### Diagnostic questions                   [mandatory]
3–5 Qs with one-line "wrong-answer interpretation" for each.

### See also
Cross-references to other books / earlier entries.

---
```

## Section conditionals

| Section | Always present? | Skip rule |
|---|---|---|
| TL;DR | Yes | — |
| History | Yes | Skip with `*(n/a)*` one-liner only when truly unhistorical |
| Intuition | Yes | — |
| Mechanics | Yes | — |
| If-you-were-the-X | Conditional | Skip when topic has no agentive subject |
| Cross-language view | Conditional | Insert `*(n/a — hardware/conceptual)*` one-liner if absent |
| Where this shows up | Yes | — (load-bearing for systems-engineering goal) |
| Diagnostic questions | Yes | — |
| See also | Yes | At least one cross-reference required |

## Style rules carried forward from existing notes

- ASCII / box-drawing diagrams preferred over Mermaid (renders everywhere, including PDF export).
- Formulas in fenced code blocks, not LaTeX.
- **Bold** for first introduction of a term.
- Trade-off framings get their own bordered box.
- Worked numerical examples included whenever the concept is quantitative.

## What a daily session does

1. **Plan (deterministic).** Read `Schedule/state.json` (or `scheduler.py today`) → which book, what pages-per-rate, which chapter context.
2. **Fetch source.** `book.py read <ALIAS> --pages X-Y` → verbatim text for the planned chunk.
3. **Verify boundary (LLM judgment).** Determinism is king for *which book / which tier / when yields happen*, but the rate-based end page is an estimate. Before composing, check:
    - Does the chunk end mid-sentence, mid-worked-example, or just before a subsection break? **Extend** by 1–3 pages to align.
    - Does the chunk overshoot a chapter end the textbook flags as a natural break ("we'll see this in §4.6")? **Pull back** to that break.
    - Adjustments must stay small (±25% of the rate). Bigger drift means the rate is wrong — fix it with `scheduler.py rate <BOOK> <pp>` instead of forcing it via `tick`.
4. **Compose** one entry per the template. Source text is **input only** — never reproduced verbatim in the note.
5. **Insert at top** of `Notes/{alias}-notes.md` (immediately after the file header).
6. **Commit reality to state.** `scheduler.py tick --t1 N --t2 N --t3 N --t4 N` with the *actual* end pages (omit any slot that matched the rate). State.json records the truth; tomorrow's chunk starts from there.
7. If a doubt surfaces, log it to `Notes/{alias}-doubts.md` with the same date header.

## Out of scope (deferred)

- "Move to notes" workflow for doubts → keep existing CLAUDE.md rule.
- Note linking / Obsidian graph auto-generation.
- Per-book style overrides (e.g., LGO might want more code, less prose). Revisit after 2 weeks of real use.

## Validation artifact

`Notes/cod-notes.md` contains a complete sample entry for **"Hardware Pipelining — The Big Idea" (Ch.4 §4.5)** demonstrating every section. Treat that entry as the canonical reference for what an entry should look and feel like.
