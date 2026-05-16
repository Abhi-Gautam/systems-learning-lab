---
name: test-today
description: Use when the user invokes /test-today in the 2026 Reading List repo to generate, grade, and carry forward a cumulative diagnostic quiz from the reading notes and doubts.
---

# /test-today

Runs a single end-to-end diagnostic loop for the reading-list repo. The goal is to test recall, mechanism understanding, systems judgment, and speaking clarity, then turn misses into tomorrow's drill list.

## Workflow

1. **Build the scope.**
   - First read the covered notes so the quiz stays grounded in what the user has already studied.
   - Read the newest relevant `Notes/*-notes.md` entries first.
   - Pull in recent `Notes/*-doubts.md` items to target unresolved concepts.
   - Use older notes for cumulative retention checks.
   - Do not ask on material outside the covered notes unless the user explicitly asked for a stretch drill.
   - Favor topics with real reasoning depth: OS, memory, databases, indexing, performance, refactoring, programming languages, architecture, and debugging.
   - If `Schedule/state.json` and `tools/scheduler.py today` help anchor the day, use them. Otherwise sample across the backlog.

2. **Draft the quiz.**
   - Produce one markdown test sheet that the user can answer in one chunk.
   - Keep it small by default: 5-7 questions.
   - Mix question types:
     - recall
     - mechanics
     - why it exists
     - apply it to a real system
     - compare trade-offs
     - failure modes and edge cases
     - design or pseudo-code when useful
   - Ask one idea per question. Prefer blunt, senior-engineer phrasing.
   - The first message should be the quiz only.

3. **Collect answers.**
   - Ask the user to answer without checking notes.
   - Treat the full response as the submission.
   - Do not interrupt mid-answer unless the user asks to split the run.

4. **Grade honestly.**
   - Score each answer on a 0-10 scale.
   - Judge:
     - correctness
     - mechanism precision
     - trade-off awareness
     - systems transfer
     - communication clarity
   - Distinguish:
     - right idea, vague explanation
     - correct but incomplete
     - wrong mental model
     - unanswered

5. **Teach back the answer.**
   - For each question, return:
     - score
     - what was right
     - what was missing
     - model answer
     - how to think about it
     - whiteboard sketch or pseudo-code when it helps
     - likely interviewer follow-ups
     - one drill for next time
   - Be direct. Do not soften weak answers.

6. **Carry forward weak spots.**
   - Rank the user's shortcomings.
   - Carry those topics into the next session.
   - Increase difficulty in the same weak areas until the user improves.

## Quiz format

Use this shape when writing a saved markdown test file:

```md
# Test Day N

## Scope
- books/topics covered
- cumulative carry-over topics

## Questions
1. ...
2. ...

## Answers
[user's full response]

## Grading
[scores and feedback]

## Weak Spots
[ranked shortcomings]

## Next Drill
[what to grill tomorrow]
```

## Source selection rules

- Use `Notes/*-notes.md` as the primary source of covered material.
- Use `Notes/*-doubts.md` to target weak concepts.
- Reuse the existing note language and rigor from `Notes/TEMPLATE.md` and `Notes/cod-notes.md`.
- Keep the test cumulative. Revisit older material, not just the latest reading.
- Build questions only from the studied surface area unless the user explicitly asks for a stretch question.

## Feedback style

- Honest, specific, and practical.
- Explain exactly why an answer is weak or strong.
- When a concept benefits from it, include:
  - a compact diagram
  - pseudo-code
  - a tiny code snippet
- Focus on helping the user answer like a senior engineer in an interview or whiteboard setting.

## Output contract

- First response: quiz only.
- Second response after answers: grading, model answers, and drill plan.
- If the user wants a persisted artifact, create or update a daily markdown file in the repo so the quiz and feedback live in Obsidian.
