---
name: study-today
description: Use when the user invokes /study-today in the 2026 Reading List repo to produce the day's batch of study notes
---

# /study-today

Orchestrates the daily 4-slot study session. This is the daily-batch mode, complementary to the interactive `Note ALIAS p.N: …` flow described in `CLAUDE.md`.

## Workflow — execute in order

1. **Plan.** Run `python3 tools/scheduler.py today`. Note the 4 slot rows it prints: each gives `{book, planned page range, chapter context, day on slot}`. If a slot prints `— (tier exhausted)`, skip it in step 2.

2. **For each non-exhausted slot in order T1, T2, T3, T4:**

   The `today` output gives each slot a planned page range `pp.X–Y`. The chunk size `Y - X + 1` is the slot's current **rate** (per-book pages/session, stored in `Schedule/state.json` under `rates`).

   a. **Locate chapter context.** `python3 tools/book.py locate <ALIAS> --page <start>` — confirms which top-level chapter the chunk sits in.

   b. **Fetch verbatim source.** `python3 tools/book.py read <ALIAS> --pages <start>-<planned_end>`. The text is *input only* — never copied into the note. If the command fails or returns empty text, stop and surface the error to the user rather than fabricating content.

   c. **Pick topic.** Open `Glossaries/<alias>.json` and find the TOC entries whose `page` falls inside the chunk. Choose one focused topic — usually the most substantive section in the range, or a single worked example/algorithm.

   d. **Verify boundary (LLM judgment).** If the planned end cuts mid-sentence, mid-example, or just before a clean section break, adjust the actual end page so that `|actual_end − planned_end| ≤ ⌈rate × 0.25⌉` pages. Larger drift means the rate itself is wrong — surface to the user and let them recalibrate with `scheduler.py rate <BOOK> <pp>`; do not force the adjustment here.

   e. **Compose** one entry per `Notes/TEMPLATE.md`. Header line: `## [YYYY-MM-DD] {Topic} · pp.X–Y · {Ch.N § N.M}`. Mandatory sections: TL;DR, History, Intuition, Mechanics, Where this shows up in real systems, Diagnostic questions, See also. Conditional: If-you-were-the-X, Cross-language view, Worked example. Match the depth and voice of `Notes/cod-notes.md`'s 2026-05-11 pipelining entry — it is the canonical exemplar.

   f. **Insert** the entry at the top of `Notes/<alias_lower>-notes.md`, immediately after the file header (after the italicized `_Entries follow…_` line and the first `---` separator). Older entries scroll downward unchanged. Never edit prior entries.

3. **Commit reality.** Run `python3 tools/scheduler.py tick` with one `--tN <actual_end_page>` flag *only* for each slot whose actual end differed from the planned end. Slots whose actual end matched the planned end take no flag (the rate applies). Example: if only T1 was adjusted to end at p.20, run `... tick --t1 20`. This advances `Schedule/state.json` and appends to `Schedule/log.md`.

4. **Verify done.** Confirm: (a) `Schedule/log.md` has a new dated line, (b) each non-exhausted slot's notes file has a new entry at the bottom. Report the result back to the user with a one-line per slot summary.

## What NOT to do

- **Do not `tick` before writing notes.** State and notes must advance together. If notes fail, state stays put.
- **Do not touch `Notes/archive/` or `Notes/*-doubts.md`.** Archive was a one-time reset. Doubts files only get entries when a real question surfaces mid-reading — see `CLAUDE.md` for that flow.
- **Do not regenerate `Glossaries/`** — they are stable inputs.
- **Do not reproduce verbatim PDF text** in any note. Notes are *your synthesized version* of the source, not transcription.
- **Do not produce fewer than 4 notes** unless a slot is genuinely exhausted (tier ran out of books).
- **Do not invent rate adjustments inside `tick`** beyond ±25%. If reading really diverges, surface it to the user; they decide whether to recalibrate with `scheduler.py rate <BOOK> <pp>`.

## Reference files

- `Notes/TEMPLATE.md` — entry template (mandatory + conditional sections + conditionals table)
- `Notes/cod-notes.md` (2026-05-11 entry on Pipelining) — canonical exemplar; match its depth and voice
- `docs/superpowers/specs/2026-05-11-notes-template-design.md` — full design spec for the note contract
- `CLAUDE.md` — book aliases, repo conventions

## Success criterion

Exactly N new entries appended (where N = non-exhausted slots that day, typically 4), one `tick` recorded in `Schedule/log.md`, no edits to prior entries or doubts files.
