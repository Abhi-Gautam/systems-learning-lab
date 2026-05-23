# Problems/

Interview problem catalog for `/design-today`. Two files:

- `hld.json` — 45 high-level design problems (15 seeded, 30 queued)
- `lld.json` — 45 low-level design problems (15 seeded, 30 queued)

## Schema (per problem row)

```jsonc
{
  "id": "H17",                   // immutable; H## for HLD, L## for LLD
  "title": "DynamoDB-style KV store",
  "ask": "...",                  // verbatim interviewer opening
  "difficulty": "easy|medium|hard",
  "asked_by": ["amazon", "google", "anthropic"],
  "tags": ["consistent-hashing", "vector-clocks", ...],
  "sources": [
    { "type": "paper|book|blog|github|leetcode",
      "ref": "Dynamo (DeCandia et al. 2007)",
      "uses_for": ["sloppy quorum §4.5", "vector clocks §4.4"] }
  ],
  "deep_dive_targets": ["mechanism A", "mechanism B"],
  "follow_ups": [
    "What if X happens?",
    "How do you handle Y?",
    "Why not Z?"
  ],
  "trap_for_juniors": "...",     // common mistake to call out
  "status": "seeded|queued|deprecated",

  // LLD-only fields
  "patterns": ["Strategy", "State"],
  "concurrency_required": true,
  "language": "go"
}
```

## Status field

- `seeded` — fully populated; in active queue rotation.
- `queued` — stubbed (title + asked_by + tags only); fills in when promoted from wave 2 to active.
- `deprecated` — no longer surfaced; kept for ID stability.

## Adding a new problem

1. Append a row to `hld.json` or `lld.json` with at least the seeded fields (if you want it active immediately) or just the queued stub.
2. Pick the next available ID (don't reuse — IDs are immutable across the catalog's lifetime).
3. If `status: seeded`, append the ID to the appropriate queue in `Schedule/design_state.json`.

## Promoting from queued to seeded

When the active queue drops to ≤3 unsolved seeded problems, the scheduler will surface a prompt. Manually:

1. Pick a queued problem (typically based on upcoming interview loop or thematic gap).
2. Fill in the missing fields: `ask`, `sources`, `deep_dive_targets`, `follow_ups`, `trap_for_juniors`.
3. Change `status` to `seeded`.
4. Append the ID to the appropriate queue in `Schedule/design_state.json`.

## ID assignments

- HLD: H01-H45 (current allocation; H46+ available for new problems)
- LLD: L01-L45 (current allocation; L46+ available for new problems)

Reserved gaps (intentionally skipped during initial design — feel free to use): none currently.
