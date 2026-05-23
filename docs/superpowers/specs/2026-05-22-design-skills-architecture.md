# Design Skills Architecture — `/design-today`

**Date:** 2026-05-22
**Status:** Approved (brainstorming complete; implementation plan to follow via writing-plans skill)
**Owner:** Abhishek
**Companion to:** `/study-today` (reading-list daily ritual)
**Related memory:** [[feedback_deep_dive_contract]] · [[feedback_run_scope]] · [[feedback_writing_style]]

---

## 1. Context & motivation

The 2026 Reading List repo already has `/study-today` — a daily 4-slot ritual that produces deep, visual-heavy notes from 13 systems books. That builds **systems-thinking depth**.

Interview prep is the parallel track. To land at Anthropic, Google, Databricks, Datadog (and equivalent), it's not enough to *understand* DDIA — you must be able to **walk into a 45-minute whiteboard session and design Spanner**, with a worked back-of-envelope, a derived consensus argument, and named production parallels.

`/design-today` is the second daily ritual. It produces one interview-grade design problem per day, alternating between High-Level Design (HLD) and Low-Level Design (LLD), with spaced-repetition revisits on Sunday. Output mirrors `/study-today`'s visual + depth contract — but the source material is interview problems and the section schema is design-specific.

## 2. Goals & non-goals

### Goals

- **One problem per day**, alternating HLD (Mon/Wed/Fri) and LLD (Tue/Thu/Sat). Sunday is revisit day.
- **Depth ≥ /study-today**: ~350-line notes with 2 mandatory deep dives per HLD problem (~120 lines each, per the depth contract).
- **Re-derivation, not pattern-matching**: every Deep-dive section must derive the mechanism from first principles, not cite a paper. (See §8.)
- **Company-tagged catalog**: every problem is tagged with which target companies ask it; supports company-specific cram weeks later.
- **Source-grounded notes**: papers + engineering blogs are pinned per problem; the composer cites named systems with named numbers in production sections.
- **Spaced repetition**: confidence-rated revisits at 7d / 21d / 60d intervals; Sunday surfaces what's due.
- **90-problem catalog over 6 months**: 30 seeded, 60 queued in wave 2.

### Non-goals

- Not a replacement for `/study-today`. Both rituals run daily.
- ~~Not a code-execution playground~~ **Amended 2026-05-23**: labs ARE now a first-class deliverable. Every fresh problem produces **three artifacts** — primer (concepts), lab (runnable code), note (interview synthesis). See §7.3, §7.4, §18.
- Not a flashcard system. Output is full-depth design notes; spaced repetition is re-derivation, not flashcard recall.
- Not a problem-volume sprint. Quality and depth are explicit priorities over count.
- Not behavioral or coding-round prep. HLD/LLD only.

## 3. Architecture overview

### File tree (additions to the repo)

```
.claude/skills/design-today/
└── SKILL.md                            # the daily ritual workflow

tools/
├── design_scheduler.py                  # mirrors scheduler.py for problems
└── design_book.py                       # mirrors book.py for source pointers (optional v2)

Problems/
├── hld.json                             # 45 HLD problems (15 seeded, 30 queued)
├── lld.json                             # 45 LLD problems (15 seeded, 30 queued)
└── README.md                            # how to add/edit problems

Schedule/
└── design_state.json                    # tracks solved, due, confidence, queues

Notes/
├── HLD-TEMPLATE.md                      # HLD entry template (narrative-first, see §7.1)
├── LLD-TEMPLATE.md                      # LLD entry template
├── hld-notes.md                         # append-only HLD entries (newest on top)
├── lld-notes.md                         # append-only LLD entries (newest on top)
├── hld-primers/                         # one primer per problem — concept prereqs (§7.3)
│   ├── PRIMER-TEMPLATE.md
│   └── H01-primer.md                    # e.g. base62, sharding, Pareto, CDN-edge, p99
└── lld-primers/                         # same shape for LLD
    └── PRIMER-TEMPLATE.md

labs/                                    # runnable code per problem (§7.4)
├── README.md                            # how labs are structured + run
├── H01-url-shortener/
│   ├── cmd/shortener/main.go            # the minimal working version
│   ├── internal/                        # toggleable variants (counter modes, hash mode)
│   ├── bench/                           # wrk/vegeta scripts; expected outputs
│   ├── EXPERIMENTS.md                   # the 5 things to break and measure
│   └── README.md                        # how to run + what each flag changes
└── L02-elevator-bank/                   # LLD labs follow the same shape
    └── ...

Sources/
├── papers.md                            # canonical papers, one-line summaries
├── blogs.md                             # engineering blogs, URLs by topic
└── README.md                            # source conventions
```

### Data flow (daily)

```
user invokes /design-today
   │
   ▼
design_scheduler.py today    ──►  picks HLD or LLD by weekday
   │                              picks next-due problem from queue
   ▼
Problems/{hld,lld}.json      ──►  read problem definition, sources
   │
   ▼
compose note per template    ──►  HLD-TEMPLATE.md or LLD-TEMPLATE.md
   │                              apply depth contract (§8)
   │                              cite sources from Sources/
   ▼
prepend entry to Notes/{hld,lld}-notes.md
   │
   ▼
design_scheduler.py tick     ──►  update design_state.json
                                  set last_seen, revisit_due (7d), confidence prompt
```

## 4. Daily workflow — `/design-today`

The skill executes in order:

### Step 1 — Pick

Run `python3 tools/design_scheduler.py today`. The scheduler:

- Reads `Schedule/design_state.json`.
- Branches on weekday: Mon/Wed/Fri → HLD queue; Tue/Thu/Sat → LLD queue; Sun → revisit queue (problems whose `revisit_due` ≤ today).
- Returns: `{problem_id, problem_definition, mode: "fresh" | "revisit"}`.

### Step 2 — Compose

For **fresh mode**, produce **three artifacts** in order (this replaces the single-note output of the original spec — see §18 amendment):

1. **Primer** (`Notes/{hld,lld}-primers/{ID}-primer.md`, ~80 lines).
   Five prerequisite concepts the user needs *before* the note will make sense. Plain language. One concrete worked example per concept. No system design yet. Format in §7.3.
2. **Lab** (`labs/{ID}-{slug}/`, runnable code).
   A minimal, single-process working version in Go. Real handlers, real data structures, real bench targets. Toggles that let the user *break* the naive version and feel the failure mode that motivates the production design. Structure in §7.4.
3. **Note** (`Notes/{hld,lld}-notes.md` entry, ~350 lines).
   The interview-grade synthesis, composed per `Notes/{HLD,LLD}-TEMPLATE.md` and the depth contract (§8). **Rewritten format (2026-05-23)**: narrative-first — start from the naive design, show what breaks, derive the fix, then jump to the deep dives. The depth contract still applies to deep dives; only the surrounding structure becomes a learning path instead of a finished cheat-sheet.

Order matters: primer → lab → note. The user is expected to read the primer, run/break the lab, *then* read the note. The note assumes lab familiarity and will reference lab files (`labs/H01-url-shortener/internal/counter.go`) where helpful.

For **revisit mode**: show the user only the problem `ask` field (the verbatim opening). The user re-derives the architecture from memory. The skill then grades against the canonical note — surfacing what's missing, what's drifted, what's wrong. Lab and primer are *not* re-shown in revisit mode — they remain on disk for reference.

### Step 3 — Insert

Prepend the entry to `Notes/{hld,lld}-notes.md`, immediately after the file header (same pattern as `/study-today`). Never edit prior entries.

### Step 4 — Tick

Run `python3 tools/design_scheduler.py tick --problem H26 --confidence 3`. The scheduler:

- Sets `last_seen[H26] = today`.
- Computes `revisit_due[H26]` based on confidence:
  - confidence 0-1: revisit in 3 days
  - confidence 2-3: revisit in 7 days
  - confidence 4: revisit in 21 days
  - confidence 5: revisit in 60 days
- Pops the problem from the active queue; appends to the revisit queue.
- Appends a line to `Schedule/log.md` mirroring `/study-today`'s log format.

### Step 5 — Report

Single-line summary: `2026-05-22 · HLD H26 LLM Inference Serving · written · ~350 lines · revisit due 2026-05-29`.

## 5. Routing logic

```python
weekday = today.weekday()  # Monday = 0, Sunday = 6

if weekday in {0, 2, 4}:       # Mon, Wed, Fri
    branch = "HLD"
elif weekday in {1, 3, 5}:     # Tue, Thu, Sat
    branch = "LLD"
else:                          # Sunday
    branch = "revisit"
```

**Override path**: `python3 tools/design_scheduler.py today --force HLD` or `--problem H26` bypasses the routing for ad-hoc study sessions.

**Override use cases**:
- Imminent interview loop tagged for a specific company: `--company anthropic` filters the queue to anthropic-tagged problems.
- Re-do a problem you bombed: `--problem L02`.
- Skip a day: don't run the command — `design_state.json.day` doesn't advance.

## 6. Data models

### 6.1 `Problems/hld.json` row shape

```json
{
  "id": "H17",
  "title": "DynamoDB-style KV store",
  "ask": "Walk me through how you'd design a Dynamo-style distributed key-value store that needs to be always-writable, even during a network partition. Single-region, 100k writes/sec, 1KB values, 99.99% availability.",
  "difficulty": "hard",
  "asked_by": ["amazon", "google", "anthropic"],
  "tags": ["consistent-hashing", "vector-clocks", "sloppy-quorum", "anti-entropy"],
  "sources": [
    {
      "type": "paper",
      "ref": "Dynamo (DeCandia et al. 2007)",
      "uses_for": ["sloppy quorum §4.5", "vector clocks §4.4", "Merkle anti-entropy §4.7"]
    },
    {
      "type": "book",
      "ref": "DDIA Ch.5+6 (your notes)",
      "uses_for": ["leaderless replication", "partitioning strategies"]
    },
    {
      "type": "blog",
      "ref": "Discord — Cassandra → ScyllaDB migration",
      "uses_for": ["real ops numbers", "tail-latency mitigations"]
    },
    {
      "type": "blog",
      "ref": "Werner Vogels — 10 lessons from DynamoDB",
      "uses_for": ["multi-tenant design", "adaptive capacity"]
    }
  ],
  "deep_dive_targets": ["sloppy quorum + hinted handoff", "vector clocks + conflict resolution"],
  "follow_ups": [
    "What if the hint queue grows unbounded?",
    "How do you handle concurrent writes during partition?",
    "Why not Paxos here?"
  ],
  "trap_for_juniors": "Picking strict quorum and missing the AP requirement. Or citing 'eventually consistent' without explaining what that means mechanically.",
  "status": "seeded"
}
```

### 6.2 `Problems/lld.json` row shape

Same shape with LLD-specific fields:

```json
{
  "id": "L02",
  "title": "Elevator bank scheduler",
  "ask": "Design the system that controls a bank of 4 elevators in a 20-floor building. Riders press call buttons; elevators pick up and drop off. Write the classes, the scheduling algorithm, and make it thread-safe.",
  "difficulty": "medium",
  "asked_by": ["amazon", "microsoft-idc", "atlassian"],
  "tags": ["state-machine", "strategy-pattern", "concurrency", "scheduling"],
  "patterns": ["Strategy", "State", "Observer"],
  "concurrency_required": true,
  "language": "go",
  "sources": [
    { "type": "github", "ref": "ashishps1/awesome-low-level-design — elevator", "uses_for": ["UML reference", "Java solution comparison"] },
    { "type": "book", "ref": "LDDD Ch.4 aggregates (your notes)", "uses_for": ["ElevatorBank as aggregate root"] }
  ],
  "deep_dive_targets": ["scheduling algorithm (LOOK vs SCAN vs nearest-car)", "concurrency model (channels vs mutex+condvar)"],
  "follow_ups": [
    "Add emergency stop priority.",
    "VIP rider — guarantee under 30s wait.",
    "How do you test this deterministically?"
  ],
  "trap_for_juniors": "Treating each elevator independently. The win is the scheduler sees the bank as one queue.",
  "status": "seeded"
}
```

### 6.3 `Schedule/design_state.json` shape

```json
{
  "start_date": "2026-05-22",
  "day": 0,
  "queues": {
    "hld": ["H01", "H10", "H02", "H09", "H03", "H14", "H16", "H17",
            "H20", "H23", "H26", "H27", "H31", "H32", "H39"],
    "lld": ["L01", "L14", "L20", "L13", "L04", "L05", "L02", "L18",
            "L21", "L17", "L26", "L10", "L12", "L28", "L35"]
  },
  "last_seen": {},
  "confidence": {},
  "revisit_due": {},
  "completed": {
    "hld_count": 0,
    "lld_count": 0,
    "revisits_count": 0
  }
}
```

### 6.4 Spaced-repetition intervals

| Confidence | Next revisit after |
|---|---|
| 0 — couldn't derive | 3 days |
| 1 — derived with hints | 3 days |
| 2 — derived but stumbled on follow-ups | 7 days |
| 3 — derived with one major gap | 7 days |
| 4 — derived cleanly | 21 days |
| 5 — could lecture on it | 60 days |

After a revisit, confidence is re-rated and the next interval is computed from the new value. Problems never "graduate" — confidence 5 still triggers a 60-day revisit. The bar is interview-readiness, not flashcard mastery.

## 7. Note templates

### 7.1 `Notes/HLD-TEMPLATE.md` — mandatory sections (narrative-first, amended 2026-05-23)

```markdown
## [YYYY-MM-DD] {ID} · {Short Title} · {company-tags}

- {deep-dive 1 topic — short phrase}
- {deep-dive 2 topic — short phrase}
- {bottleneck or trade-off headline}

### Prereqs (run these first)
- Primer: `Notes/hld-primers/{ID}-primer.md` — concepts you need before this note
- Lab:    `labs/{ID}-{slug}/` — runnable code; do EXPERIMENTS.md before reading on

### Problem (as asked)
Verbatim `ask`. Below it, the 5 clarifying questions to ask back.

### Back-of-envelope
QPS · payload · storage · cost. Show arithmetic.

### Naive design (start here — this is what a junior would draw)
ONE diagram of the obvious wrong-but-simple design. Single DB, no cache,
single counter. Identifies WHY it's wrong by reference to the BOE numbers
("at 300k QPS this row sees row-lock contention; latency p99 explodes").
This section makes the rest of the note feel inevitable instead of magical.

### What breaks, in order
A numbered list. Each item: (a) the constraint that's violated, (b) the
specific failure mode, (c) the lever applied to fix it. The fixes
accumulate into the final architecture — the reader watches it being
built, not handed.

### Final architecture
ONE diagram (mermaid or ASCII), ≤8 boxes. Every arrow labeled.
Annotated with which "what breaks" item motivated each box.

### Functional / Non-functional requirements
Two-column table. NFRs are the trade-off levers.

### API contract
HTTP/gRPC sketch. Idempotency. Streaming concerns.

### Data model
Persists vs ephemeral. Schema sketch.

### Deep dive — 2 mechanisms, ~120 lines each
For each, follow the 6-subsection contract (§8). Pick mechanisms the lab
exercised — the deep dive can then reference lab files directly:
"see labs/H01-url-shortener/internal/counter.go for the pre-allocation."

### Bottlenecks + scaling levers
Where the system breaks at 10x, 100x; the lever for each.

### Trade-off matrix
3×3+ table. Option × criteria (latency, cost, complexity, consistency).

### Common follow-ups
The 5 "what if" questions to pre-empt.

### What top-tier looks for
"Mid says X; staff says Y because Z."

### Diagnostic questions (self-quiz)
5 questions with wrong-answer interpretations.
```

Total target: ~380 lines per HLD entry (slightly larger to absorb the naive-design + what-breaks scaffold). The trade-off matrix and follow-ups stay; only the *order* changed — naive → break → fix → final → deep, instead of finished → deep.

### 7.2 `Notes/LLD-TEMPLATE.md` — mandatory sections

```markdown
## [YYYY-MM-DD] {ID} · {Short Title} · {company-tags}

- {pattern 1}
- {pattern 2}
- {concurrency model summary}

### Problem (as asked)
Verbatim `ask`. Below it, the 5 clarifying questions.

### Requirements teardown
Functional bullets + NFRs (concurrency, extensibility, SLA).

### Actor & entity inventory
One line per class. ≤8 entries.

### Class diagram
ASCII or mermaid. Inheritance vs composition explicit. Aggregation
arrows labeled.

### Patterns applied + alternatives rejected
Strategy / State / Observer / etc. "Why not X" for each common alt.

### Code skeleton (Go)
50–80 lines. Real signatures, not bodies. Picks Go because DSG + LGO
are in the reading list — single-language deepening.

### Deep dive — 2 mechanisms, ~120 lines each
Same 6-subsection contract as HLD. For LLD, the mechanisms are
typically: the scheduling algorithm, the concurrency primitive, the
pattern instantiation. Example for L02 elevator:
  Deep dive 1: scheduling algorithm (LOOK vs SCAN vs nearest-car)
  Deep dive 2: concurrency model (channels vs mutex+condvar)

### Concurrency model
Goroutines + channels OR mutex + condvar; deadlock-freedom argument.

### Extension points
"Add priority floors / VIP / emergency" — show each landing point.

### SOLID self-audit
5-row table: S/O/L/I/D × did we obey, why/why not.

### Where this shows up in production
LDDD aggregate parallels; real schedulers (CPU sched, k8s sched).

### Common follow-ups
5 typical "now add..." prompts.

### Diagnostic questions
5 questions with wrong-answer interpretations.
```

Total target: ~350 lines per LLD entry.

### 7.3 `Notes/{hld,lld}-primers/PRIMER-TEMPLATE.md` — added 2026-05-23

The primer is **the reading you do BEFORE the note will make sense**. Five concepts max. ~80 lines. Plain language. The goal is to remove every "wait what is X" moment from the note.

```markdown
# {ID} Primer — {Short Title}

_Read this before `notes/{hld,lld}-notes.md` entry {ID}. Run the lab next._

## What this problem assumes you already know

A 3-row table: concept · why it shows up here · ~1-line definition.
This is the *triage* — if all 5 are familiar, skip to the lab.

## Concept 1: {name}
~12 lines. ONE concrete example with numbers. Tiny code snippet if the
concept is mechanical (e.g., base62 encoding). NO system-design framing
yet — pure prerequisite.

## Concept 2..5: same shape

## Checkpoint: can you answer these without the note?
3-4 simple questions. If yes → open the lab. If no → re-read the concept
you stumbled on.
```

**Primer ≠ note.** No deep dives, no trade-off matrices, no production parallels. It's the textbook chapter you wish you'd read in college.

### 7.4 `labs/{ID}-{slug}/` — runnable lab structure (added 2026-05-23)

Every fresh problem gets a runnable Go (or Python for LLD where idiomatic) project under `labs/`. The lab is **the experiment surface**: the user runs it, breaks it, measures it. The note's claims become observations the user has personally seen.

**Required files:**

```
labs/{ID}-{slug}/
├── README.md            # what this lab is, how to run, what flags do
├── EXPERIMENTS.md       # 3-5 numbered experiments with expected outcomes
├── go.mod
├── cmd/{name}/main.go   # the entrypoint — ≤80 LOC; orchestrates internal/
├── internal/            # the variants under test
│   ├── {primary}.go     # production-ish version
│   └── {naive}.go       # the broken-on-purpose version (e.g. single counter)
└── bench/
    ├── run.sh           # wrk or vegeta invocations
    └── expected.md      # numbers the user should see on their laptop
```

**Required behavior:**

- **Two variants minimum**: a `naive` mode (the "junior" design from §7.1's naive section) and a `prod` mode (the design the note advocates). A CLI flag toggles them.
- **One observable failure**: at least one experiment in `EXPERIMENTS.md` makes `naive` mode visibly fail (latency spike, collision, lost data) where `prod` mode does not.
- **Self-contained**: `go run ./cmd/...` works without external services where possible. If Redis/Postgres is needed, provide a `docker-compose.yml` and gate it behind a flag.
- **Tiny**: aim for ≤500 LOC total. The lab is a teaching tool, not a product.

**Required `EXPERIMENTS.md` shape:**

```markdown
# {ID} Lab — Experiments

## Experiment 1: feel the bottleneck the production design fixes
**Run**: `./bench/run.sh --mode=naive --qps=5000`
**Expected**: p99 > 200ms after ~30s as counter row locks contend.
**Then**: `./bench/run.sh --mode=prod --qps=5000`
**Expected**: p99 stays < 20ms.
**Lesson**: this is *why* §"Deep dive 1: ID generation" exists.

## Experiment 2..N: each maps to a deep dive or trade-off in the note.
```

The note's deep dives MUST reference at least one lab experiment by name. That's the link that makes the note feel grounded instead of recited.

## 8. The depth contract — every Deep dive follows this

**This is the centerpiece of the spec.** Both HLD and LLD notes have a "Deep dive" section. The contract for every Deep dive is non-negotiable.

### Six mandatory subsections

1. **Why does this mechanism exist?** Problem before, alternative rejected, the constraint that forced the design.
2. **Walk it concretely.** ASCII diagram or numbered steps with a specific scenario (named nodes, values, timestamps). No abstract descriptions.
3. **The cost you're accepting.** A trade-off table. What does this mechanism give up to gain its property? At least 3 rows.
4. **Failure modes the interviewer will drill into.** Q-A format, 4-5 rows. Specific "what if X crashes / partition lasts / queue overflows" questions.
5. **How to derive it from first principles.** Numbered list, 6-8 steps. Show the reader how to reconstruct the mechanism without quoting the paper.
6. **Where this shows up in production.** Named systems with named numbers. At least 3 named systems.

### Quantitative bar

- ~120 lines per deep dive (not per subsection — per dive)
- 2 deep dives per note → ~250 lines of mechanics + ~100 lines of surrounding sections = ~350-line notes
- A deep dive that fits in one paragraph or is just a paper citation = wrong mechanism picked

### What this rule rejects

- One-line citations like *"Dynamo's sloppy quorum (§4.5) keeps a write available when a preferred replica is partitioned."* Pattern-matching, not depth.
- Abstract descriptions without a concrete worked scenario.
- "Many companies do this" production sections without named systems and numbers.

### Reference: a Deep dive that satisfies the contract

A worked example of all 6 subsections applied to "Sloppy quorum + hinted handoff" (Dynamo §4.5–4.7) was produced during the 2026-05-22 brainstorming session and is the canonical reference. The next iteration of the spec will move that example into a `Notes/HLD-TEMPLATE.example.md` companion file.

## 9. Source integration

### 9.1 Three integration points

1. **Catalog-level**: every problem's `sources` array pins canonical papers + blogs + book chapters with `uses_for` fields.
2. **Compose-time**: the composer cites these sources in two places — Deep-dive subsection 6 (production) and the Mechanics walk.
3. **Repo-level**: `Sources/papers.md` and `Sources/blogs.md` are the central indexes — referenced, not duplicated.

### 9.2 `Sources/papers.md` structure

One row per canonical paper:

```markdown
## Dynamo (DeCandia et al. 2007)

**Topics**: consistent hashing, vector clocks, sloppy quorum, anti-entropy
**Powers**: H09 distributed cache, H17 DynamoDB, H23 leaderboard
**One-line**: AP leaderless KV store; introduces sloppy quorum + hinted handoff + Merkle anti-entropy as the three-mechanism stack for AP under partition.
**Key sections**: §4.4 vector clocks · §4.5 sloppy quorum · §4.7 anti-entropy
```

### 9.3 `Sources/blogs.md` structure

One row per blog post; organized by topic:

```markdown
## Storage / KV stores

- **Discord — Cassandra to ScyllaDB migration (2022)**
  https://discord.com/blog/how-discord-stores-billions-of-messages
  *Powers*: H03 WhatsApp · H17 DynamoDB · H22 Slack
  *Key numbers*: 8B messages stored, ~2M writes/sec peak, JVM GC root cause for migration.

- **Werner Vogels — 10 lessons from DynamoDB**
  https://www.allthingsdistributed.com/2017/01/amazon-dynamodb-10-years.html
  *Powers*: H17 DynamoDB
  *Key insights*: multi-tenant adaptive capacity, hot-key mitigation.
```

### 9.4 No pre-fetching

Papers and blogs are referenced by name + URL; the composer relies on training-data knowledge to cite specifics. For recent blogs (last 2 years) or numbers the LLM is uncertain about, the composer may WebFetch and cite. Pre-fetching everything would bloat the repo and most content is stable.

## 10. Seed catalog — 30 problems

### 10.1 HLD seeds (15)

| ID | Title | Asked by | Tests |
|---|---|---|---|
| H01 | URL shortener | Universal | Hashing, base62, cache, write skew |
| H02 | Twitter home timeline | Meta, Twitter, LinkedIn | Fan-out write vs read, celebrity problem |
| H03 | WhatsApp / messaging | Meta, Snap, Discord | Ordering, presence, E2E |
| H09 | Distributed cache (Redis/Memcached) | Universal | Consistent hashing, eviction, replication |
| H10 | Rate limiter | Universal | Token bucket vs sliding window |
| H14 | Stripe payment processing | Stripe, Square, fintech | Idempotency, double-entry, webhooks |
| H16 | Kafka | Confluent, LinkedIn, Datadog | Log abstraction, partitions, ISR, EOS |
| H17 | DynamoDB-style KV store | Amazon, Google, Anthropic | Consistent hashing, vector clocks, sloppy quorum |
| H20 | Ad-click aggregator | Google, Meta | Stream proc, watermarks, lambda vs kappa |
| H23 | Distributed counter / leaderboard | Universal | CRDTs, HyperLogLog |
| H26 | LLM inference serving (10k QPS) | **Anthropic**, Google | KV cache, continuous batching |
| H27 | RAG over 1B documents | **Anthropic**, OpenAI | Vector index, hybrid search, freshness |
| H31 | Time-series DB (Datadog-scale) | **Datadog**, Honeycomb | Husky-style columnar TSDB |
| H32 | Distributed tracing | **Datadog**, Honeycomb | Sampling, span propagation |
| H39 | Spanner | **Google** | TrueTime, Paxos groups, external consistency |

Coverage check — these 15 hit: hashing, fan-out, real-time messaging, caching, rate limiting, payment/ledger, log abstraction, NoSQL, streaming, approximate counting, ML serving, retrieval, TSDB, observability, global consensus. Every major HLD primitive appears at least once.

### 10.2 LLD seeds (15)

| ID | Title | Patterns/primitives | Asked by |
|---|---|---|---|
| L01 | Parking lot | Strategy, Factory | Universal LLD warm-up |
| L02 | Elevator bank (multi-elevator) | State + Strategy + concurrency | Amazon, Microsoft IDC, Atlassian |
| L04 | Vending machine | State pattern | Universal |
| L05 | Tic-Tac-Toe (extensible) | Strategy (win-check) | Universal |
| L10 | Splitwise | Domain modeling + graph algo | Indian product cos, fintech |
| L12 | Meeting scheduler / calendar | Interval tree | Universal |
| L13 | Logging framework | Chain of Responsibility, Builder | Universal |
| L14 | LRU cache (thread-safe) | DLL + hashmap + locking | Universal |
| L17 | Thread pool / executor | Producer-consumer, work-stealing | Senior+ |
| L18 | In-memory file system | Composite pattern | Universal |
| L20 | In-memory pub-sub | Observer + topic registry | Universal |
| L21 | Undo/Redo (text editor) | Command + Memento | Universal |
| L26 | Thread-safe blocking queue | Condition variables | Senior+ |
| L28 | Circuit breaker | State machine (Closed/Open/HalfOpen) | Universal |
| L35 | Workflow engine (DAG of tasks) | Topological eval, retry | **Staff bar** |

Coverage check — these 15 hit: Strategy, State, Factory, Composite, Observer, Command, Memento, Chain of Responsibility, Builder. Plus four levels of concurrency. Plus one staff-bar problem.

### 10.3 Catalog seeding order (queue priority)

HLD queue (start with broad coverage, then push depth):
`H01 → H10 → H02 → H09 → H03 → H14 → H16 → H17 → H20 → H23 → H26 → H27 → H31 → H32 → H39`

LLD queue (start with cleanest patterns, then concurrency, then staff bar):
`L01 → L14 → L20 → L13 → L04 → L05 → L02 → L18 → L21 → L17 → L26 → L10 → L12 → L28 → L35`

## 11. Wave 2 queue (deferred to status: queued)

These 60 problems exist in `Problems/{hld,lld}.json` with `status: "queued"` from day 1. They surface only after the seed set runs dry, or when the user invokes a company-specific cram.

### HLD wave 2 (30)

H04 Uber match · H05 Netflix delivery · H06 YouTube upload · H07 Dropbox sync · H08 Instagram feed · H11 Web crawler · H12 Search autocomplete · H13 Order matching · H15 Notification system · H18 GFS/HDFS · H19 Google Maps · H21 BookMyShow · H22 Slack · H24 Log analytics · H25 Feature flag service · H28 Claude prompt cache (**Anthropic**) · H29 Agent orchestration (**Anthropic**) · H30 LLM evals infra (**Anthropic**) · H33 Alerting engine (**Datadog**) · H34 APM auto-instrumentation (**Datadog**) · H35 Query optimizer for Parquet (**Databricks**) · H36 Delta Lake transaction log (**Databricks**) · H37 Unity Catalog (**Databricks**) · H38 Notebook collaboration (**Databricks**) · H40 YouTube live streaming (**Google**) · H41 Google ad auction (**Google**) · H42 Distributed training infra · H43 Feature store · H44 Model A/B routing · H45 Vector database

### LLD wave 2 (30)

L03 ATM · L06 Chess · L07 Deck of cards / Poker · L08 Library management · L09 Hotel reservation · L11 Snake & Ladder · L15 Rate limiter (LLD) · L16 Connection pool · L19 Trello · L22 Stack Overflow · L23 Shopping cart · L24 Swiggy core domain · L25 Music player · L27 Distributed lock manager interface · L29 Retry library with backoff · L30 Feature flag SDK (client) · L31 Typed config loader · L32 Event-sourced bank account · L33 SQL expression tree · L34 Regex engine (Thompson NFA) · L36 Idempotency-key middleware · L37 Bounded actor system · L38 Metrics library · L39 Structured logger · L40 DI container · L41 Typed message bus · L42 Saga coordinator · L43 OAuth client · L44 Feature flag evaluator (server) · L45 Graceful shutdown coordinator

## 12. Scheduler — `tools/design_scheduler.py` command surface

Mirrors `tools/scheduler.py` shape:

```
python3 tools/design_scheduler.py today
  → prints {problem_id, mode, problem_definition}
  → does NOT modify state

python3 tools/design_scheduler.py tick --problem H26 --confidence 3
  → advances day, updates last_seen, computes revisit_due
  → appends to Schedule/log.md

python3 tools/design_scheduler.py status
  → prints queue depths, recent completions, upcoming revisits

python3 tools/design_scheduler.py revisit
  → lists all problems with revisit_due ≤ today

python3 tools/design_scheduler.py override --problem L02
  → bypasses routing, forces L02 next

python3 tools/design_scheduler.py company --tag anthropic
  → filters next-pick to problems tagged asked_by: anthropic
```

## 13. Sunday revisit flow (concrete)

On Sunday:

1. Scheduler picks all problems with `revisit_due ≤ today`. If multiple, picks lowest confidence first.
2. Skill shows ONLY the `ask` field (verbatim opening) — no other context from the catalog or notes.
3. User re-derives the design in conversation: APIs, architecture, deep dives, trade-offs.
4. Skill grades against the canonical note from `Notes/{hld,lld}-notes.md`:
   - **Architecture diff**: what boxes/arrows did the user miss or add?
   - **Deep-dive coverage**: did the user re-derive both deep dives? Which subsections were skipped?
   - **Trade-off coverage**: did the user surface the same trade-offs?
   - **Follow-ups**: present 2 random follow-ups from the catalog; check answers.
5. Skill prompts: "Rate confidence 0–5." Tick advances state.

If multiple problems are due, repeat steps 2-5 until the user runs out of time (no fixed cap; the user controls cadence).

## 14. Reasonable calls (R1–R10) — captured for future revisit

These are the calls made during 2026-05-22 brainstorming. Each can be revisited via a fresh brainstorming session, but they're the working defaults:

| # | Call | Rationale |
|---|---|---|
| R1 | Code skeletons in **Go** (LLD) and **Go pseudocode** (HLD deep dives) | Matches DSG + LGO already in reading list |
| R2 | **Seed 30 problems** (15 HLD + 15 LLD), grow weekly | Don't pre-populate 90 stale entries |
| R3 | Spaced repetition intervals **7d → 21d → 60d** (with 3d for low confidence) | Standard SM-2 lite |
| R4 | Confidence rating **0–5** after each solve | Drives revisit priority |
| R5 | Revisit format: **show problem only, user re-derives, skill grades** | Re-derivation > recall |
| R6 | Problem IDs **immutable** (`H01`, `L02`) — titles can drift | Stable cross-references |
| R7 | Catalogs **opinionated** — staff-bar curation, user can override | Save user from picking from 90 |
| R8 | One spec doc per design session in `docs/superpowers/specs/` | Mirrors how `/study-today` was designed |
| R9 | After spec approval → **writing-plans** skill creates impl plan; no code in brainstorming | Brainstorming skill HARD-GATE |
| R10 | Reuse `book.py`-style helpers; extend scheduling primitives | Existing scaffolding works |

## 15. Open questions

These are unresolved at spec-time. They'll be answered during implementation planning or first weeks of use.

1. **Should `/design-today` ever produce TWO problems on a single day** (e.g., when the user has time and wants a sprint)? Current spec: no — one per day. Add `--double` flag later if needed.
2. **Should HLD code be in Go too** (matching LLD), or stick with mermaid + pseudocode? Current spec: pseudocode. LLD has full Go skeletons; HLD doesn't need them.
3. **What's the trigger for promoting wave 2 → seeded?** Current spec: when the active seed queue drops to ≤ 3 unsolved. Could be manual instead.
4. **Should we add a `T5` slot in `/study-today` for design problems** as a backup integration path? Current spec: no — separate commands. Revisit if `/design-today` is being skipped frequently.
5. **How do we handle Anthropic-specific problems whose canonical sources are private?** (E.g., Claude's actual context cache implementation.) Current spec: cite public blog posts + papers; treat internal implementation as out-of-scope. Anthropic interviews are graded on reasoning, not insider knowledge.
6. **What happens when the user is preparing for a specific company within 2 weeks** of a loop? Current spec: `--company anthropic` flag re-orders the queue. May need a "cram week" mode that disables spaced repetition temporarily.

## 16. Next step

This spec is approved. The next action is to invoke the **writing-plans** skill to create the implementation plan that turns this design into:

1. `Problems/hld.json` + `Problems/lld.json` (seeded 30, queued 60).
2. `tools/design_scheduler.py` (the command surface in §12).
3. `Notes/HLD-TEMPLATE.md` + `Notes/LLD-TEMPLATE.md` (the templates in §7).
4. `.claude/skills/design-today/SKILL.md` (the daily workflow in §4).
5. `Sources/papers.md` + `Sources/blogs.md` (the source index in §9).
6. `Schedule/design_state.json` (initial state in §6.3).

Implementation plan TBD; will reference back to this spec for canonical decisions.

## 17. Appendix — relationship to `/study-today`

Both rituals run daily. Independent state, independent notes, independent scheduling. Cross-references happen *inside notes* (e.g., an HLD note on Kafka cites your DDIA Ch.11 notes) but the two rituals never touch each other's files.

```
                            DAILY
                              │
                              ├──► /study-today  ──► 4 book notes (T1-T4)
                              │                       Notes/{ddia,n2t,...}-notes.md
                              │                       Schedule/state.json
                              │
                              └──► /design-today ──► 1 design note (HLD or LLD)
                                                      Notes/{hld,lld}-notes.md
                                                      Schedule/design_state.json
```

## 18. Amendment log

### 2026-05-23 — Primer + Lab + narrative-first note

**Trigger**: H01 URL-shortener note (the first /design-today output) landed at 380 lines of finished interview-cheat-sheet. User read it cold and was overwhelmed — the deep dives assumed concepts (base62, sharding, Pareto, CDN-edge semantics, p99) the user hadn't internalized, and the note structure presented the *finished* design instead of *deriving* it.

**Diagnosis**: the original §7.1 template optimized for *revisit* readability (you already know the system, you want dense reference) at the cost of *first-contact* learnability. The depth contract (§8) is correct for deep dives but lethal as the top-level layout.

**Changes**:

1. **§2 non-goals**: removed "not a code-execution playground". Labs are now first-class.
2. **§3 file tree**: added `Notes/{hld,lld}-primers/` and `labs/{ID}-{slug}/` directories.
3. **§4 Step 2 (Compose)**: fresh mode now emits THREE artifacts — primer, lab, note — in that order. Note assumes lab familiarity and may reference lab files.
4. **§7.1 HLD template**: restructured to narrative-first. Order is now Prereqs → Problem → BOE → **Naive design → What breaks → Final architecture** → contracts → deep dives → trade-offs. Deep dives still follow the §8 contract; only the surrounding scaffolding changed.
5. **§7.3 new**: primer template — 5 concepts max, ~80 lines, plain language.
6. **§7.4 new**: lab structure — runnable Go, naive vs prod variants, EXPERIMENTS.md mapping experiments to deep-dive sections.

**Not changed**:
- §8 depth contract (still 6 subsections per deep dive, ~120 lines each).
- Scheduler routing (§5), spaced-repetition intervals (§6.4), revisit flow (§13).
- LLD template (§7.2) — to be revisited if/when an LLD note lands and feels similarly opaque.

**Migration plan for H01**:
- H01 note in `notes/hld-notes.md` stays as-is (it's the interview-grade reference).
- Retroactively author `notes/hld-primers/H01-primer.md` and `labs/H01-url-shortener/`.
- Future /design-today runs produce all three artifacts from day one.

End of spec.
