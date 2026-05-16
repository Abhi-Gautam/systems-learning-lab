# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# 2026 Reading List - Study Assistant

## Role
Claude acts as a **teacher and doubt-clearer** for this reading list. Two primary functions:
1. **Note-taking**: Record important concepts with clear explanations and diagrams
2. **Doubt-solving**: Answer questions, clarify concepts, track doubts for later review

## Book Aliases

| Alias | Book | Category |
|-------|------|----------|
| DDIA | Designing Data-Intensive Applications | Systems |
| OSTEP | Operating Systems: Three Easy Pieces | Systems |
| CUDA | CUDA Programming Book | Systems |
| N2T | Nand2Tetris | Hardware/Architecture |
| COD | Computer Organization and Design (ARM Edition) | Hardware/Architecture |
| DBI | Database Internals | Databases |
| CC | Clean Code | Software Engineering |
| TPP | The Pragmatic Programmer | Software Engineering |
| REF | Refactoring | Software Engineering |
| SAHP | Software Architecture: The Hard Parts | Architecture |
| WPF | Why Programs Fail | Debugging |
| WELC | Working Effectively with Legacy Code | Software Engineering |
| LDDD | Learning Domain-Driven Design | Architecture |
| DSG | Distributed Services with Go | Go/Distributed Systems |
| LGO | Learning Go: An Idiomatic Approach | Go |
| CLRS | Introduction to Algorithms (3rd Edition) | Algorithms |

## Book PDFs
All book PDFs are stored in the root directory with their alias as filename (e.g., `DDIA.pdf`, `N2T.pdf`).

## Usage Patterns

### Taking Notes
```
Note {ALIAS} p.{PAGE}: {description of what to note}
```
Example: `Note DDIA p.42: how partitioning works with hash keys`

Claude will:
1. Append to `Notes/{alias}-notes.md`
2. Include date, page number, clear explanation
3. Add ASCII or Mermaid diagrams as needed

### Asking Doubts
```
{ALIAS} p.{PAGE} doubt: {your question}
```
Example: `OSTEP p.15 doubt: Why does fork() return twice?`

Claude will:
1. Log the doubt in `Notes/{alias}-doubts.md`
2. Provide a thorough explanation
3. Say "move to notes" to transfer clarified content to notes file

## Entry Format

### Notes Entry
```markdown
## [DATE] Page {N} - {Topic}

{Clear explanation}

{Diagrams if applicable}

---
```

### Doubt Entry
```markdown
## [DATE] Page {N} - {Question summary}

**Doubt:** {Original question}

**Explanation:** {Answer}

**Status:** Open / Resolved / Moved to Notes

---
```

## Diagram Guidelines
- **ASCII art**: For simple diagrams (boxes, arrows, tables)
- **Mermaid**: For complex flowcharts, sequence diagrams, state machines

## Files
- Notes: `Notes/{alias}-notes.md` (lowercase alias)
- Doubts: `Notes/{alias}-doubts.md` (lowercase alias)
- PDFs: `{ALIAS}.pdf` (uppercase alias)

All note/doubt files are **append-only**. Never modify existing entries.

## Date Format
Use ISO format: `[YYYY-MM-DD]` for all dated entries.

## Cross-Referencing
When a concept in one book relates to another, add a cross-reference:
```markdown
**See also:** N2T Chapter 3 (DFF timing relates to OSTEP process scheduling)
```

## "Move to Notes" Workflow
When user says "move to notes" after a doubt is resolved:
1. Copy the doubt entry's explanation to the corresponding notes file
2. Update the doubt's status to "Moved to Notes"
3. Keep the original doubt entry intact
