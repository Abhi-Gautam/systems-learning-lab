# Notes Entry Template

Every entry in a book's notes file follows this structure. **New entries are inserted at the top of the file, immediately after the file header** — the newest entry is always the first thing you see when you open the file. Prior entries are append-only (never edited), they simply scroll downward as new ones land.

Sections marked **conditional** may be skipped when not applicable (insert a one-line "(n/a — reason)" instead of dropping the heading silently).

---

## [YYYY-MM-DD] {Topic Title} · pp.X–Y · {Ch.N § N.M}

### TL;DR
3 sentences. Written last, placed first. Re-scan budget: 30 seconds.

### History — "why does this exist?"
3–4 lines. The problem the world had before this idea, the breakthrough that introduced it (year + system/person where possible), and one line on what it displaced. Skip with `*(n/a — no notable origin story)*` only when truly unhistorical.

### Intuition — "this is like…"
One short analogy paragraph. Anchor before mechanics.

### Mechanics
The juiced-out core. ASCII diagrams, formulas in code blocks, **bolded keywords**, multi-framing (prose + table + flowchart), trade-off boxes when there's a real tension, worked numerical example if quantitative.

### If you were the {CPU / scheduler / process / replica}… *(conditional)*
Active-reasoning prompt with the textbook's answer. Two paragraphs max.

### Cross-language view *(conditional)*
Side-by-side Rust / Go / Python (and C when it matters for systems). Includes a "what the stdlib actually does" note.

### Where this shows up in real systems
2–3 bullets connecting textbook → production. Mandatory. This is the systems-engineering through-line.

### Diagnostic questions
3–5 questions with one-line "wrong-answer interpretation" for each.

### See also
Cross-references to other books / earlier entries.

---

## Section conditionals

| Section | Always present? |
|---|---|
| TL;DR · History · Intuition · Mechanics · Where this shows up · Diagnostic Qs · See also | **Yes** |
| If-you-were-the-X | When topic has an agentive subject |
| Cross-language view | When concept has a code form |
| Worked example (inside Mechanics) | When quantitative |
