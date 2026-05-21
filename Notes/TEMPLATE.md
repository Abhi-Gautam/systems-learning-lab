# Notes Entry Template

Every entry in a book's notes file follows this structure. **New entries are inserted at the top of the file, immediately after the file header** — the newest entry is always the first thing you see when you open the file. Prior entries are append-only (never edited), they simply scroll downward as new ones land.

Sections marked **conditional** may be skipped when not applicable (insert a one-line "(n/a — reason)" instead of dropping the heading silently).

---

## [YYYY-MM-DD] {Short Topic} · pp.X–Y · {Ch.N § N.M}

- {sub-topic 1 — short phrase}
- {sub-topic 2 — short phrase}
- {sub-topic 3 — short phrase, optional}

### History — "why does this exist?"
3–4 lines. Problem-before, breakthrough (year + system/person), what it displaced.

### Intuition — "this is like…"
One short analogy paragraph. Anchor before mechanics. **Use real-world software analogies wherever possible** (a CDN edge cache, a Postgres autovacuum, a Slack message queue, a Git rebase) — they stick harder than abstract ones because the reader has touched them. Avoid generic "library / restaurant / highway" analogies if a software analogy fits.

### Mechanics
The juiced-out core. ASCII diagrams, formulas in code blocks, **bolded keywords**, multi-framing (prose + table + flowchart), trade-off boxes when there's a real tension, worked numerical example if quantitative.

Mechanics is where the saved-from-TL;DR-and-See-also budget gets reinvested: **more worked examples, more code, more concrete numbers**. Prose is connective tissue between visuals and examples — not the main medium. If a paragraph could be a table, it should be. If a comparison spans three systems, show all three side-by-side.

### If you were the {CPU / scheduler / process / replica}… *(conditional)*
Active-reasoning prompt with the textbook's answer. Two paragraphs max.

### Cross-language view *(conditional)*
Side-by-side Rust / Go / Python (and C when it matters for systems). Includes a "what the stdlib actually does" note.

### Where this shows up in real systems
2–3 bullets connecting textbook → production. Mandatory. This is the systems-engineering through-line. Name **specific** systems and what they do (e.g., "Postgres uses MVCC snapshots stored in pg_xact" — not "many databases use this").

### Diagnostic questions
3–5 questions with one-line "wrong-answer interpretation" for each.

---

## Title format

The H2 header has three parts separated by `·`:

```
## [DATE] {Short Topic} · pp.X–Y · {Ch ref}
```

- **Short Topic**: 2–6 words. A handle, not a sentence. Bad: `Yield, Performance, and the CPU Time Equation`. Good: `Yield & CPU performance equation`.
- **Page range**: `pp.50–60` (en-dash, not hyphen).
- **Chapter ref**: `Ch.1 §1.5 → §1.6` or `§1.5–§1.6` — whatever's tightest.

If the entry covers multiple distinct sub-topics, follow the H2 header with a **3-bullet preview list** (short phrases, no prose). The preview is for re-scan navigation: glance at the bullets, know if this entry has what you need without reading the prose.

## Section conditionals

| Section | Always present? |
|---|---|
| Preview bullets · History · Intuition · Mechanics · Where this shows up · Diagnostic Qs | **Yes** |
| If-you-were-the-X | When topic has an agentive subject |
| Cross-language view | When concept has a code form |
| Worked example (inside Mechanics) | When quantitative |

## Removed sections (do not reintroduce)

- **~~TL;DR~~** — was meant for 30-second re-scan, but the preview-bullet list under the title does the same job in 5 seconds. User never reads TL;DR in practice.
- **~~See also~~** — cross-references were never followed during re-scans. Cross-book links, when truly load-bearing, go inline in prose where the connection actually matters (e.g., "this is the same idea as OSTEP's `wait()` semantics — the caller blocks until child state stabilizes").

## Visual + low-level depth contract (carried forward)

- Tables and diagrams over prose wherever a comparison/structure/flow is being explained.
- When a low-level concept appears (memory layout, cache lines, syscall path, bytes-on-wire, instruction encoding, etc.), **stop and teach it in detail**. That depth is the whole point of the reading list.
- Mechanics sections should look like: short framing → diagram → short interpretation → table → worked example → short synthesis. Walls of prose in Mechanics are a regression.
