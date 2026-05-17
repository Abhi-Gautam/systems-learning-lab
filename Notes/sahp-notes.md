# SAHP Notes

_Entries follow the template at `Notes/TEMPLATE.md`. Append-only. **Newest entry at top**, immediately after this header._

---

## [2026-05-21] When There Are No "Best Practices" — Architectural Decision Records, Fitness Functions, and Why "It Depends" Is the Right Answer · pp.19–40 · Ch.1 What Happens When There Are No "Best Practices"? (full chapter: Why "The Hard Parts"? → Giving Timeless Advice → The Importance of Data in Architecture → ADRs → Fitness Functions → Architecture Versus Design → Trade-offs)

### TL;DR
The chapter's central claim is that **software architecture has no best practices** — only **trade-offs evaluated in context**. The authors frame the entire book as a defense against the "framework-of-the-year" mode of architectural advice, arguing instead for two durable tools: **Architectural Decision Records (ADRs)** which capture *why* a decision was made (context + alternatives + consequences) so future-you can decide whether the rationale still holds, and **Fitness Functions** which are executable assertions that verify architectural characteristics (latency, modularity, security) automatically — converting subjective qualities into objective tests. The deeper move: architects are paid for **judgment under uncertainty**, not for knowing the right answer.

### Intuition — "this is like…"
Architecture is **city planning** with no comparable city. A developer looking something up on Stack Overflow is like a homeowner asking "what color should I paint my door?" — has been answered a million times. An architect asks "should we put a tram line through this neighborhood?" — depends on terrain, budget, who lives there, what year it is, what the next mayor wants. There is no Stack Overflow for that. The right artifact isn't an *answer*, it's a **written record of why you chose what you chose** and a **way to detect when your assumptions stop being true**.

### Mechanics

#### 1. The architecture vs design distinction

| Aspect | Design | Architecture |
|---|---|---|
| Scope | Inside a component | Across components |
| Reversibility | Cheap to change | Expensive to change |
| Affects | One team's productivity | Whole system's properties |
| Driven by | Functional requirements | **Non-functional** (-ility) requirements |
| Time horizon | Months | Years |
| Example | "Use builder pattern for this object" | "Split this monolith into 8 services" |

> The whole book is about the *architecture* row — decisions that are hard to undo, span teams, and shape -ility characteristics.

#### 2. Why "best practices" fail at the architecture layer

```
   Stack Overflow (works for design)            Architecture (doesn't work)
   ┌─────────────────────────────────────┐    ┌─────────────────────────────────────┐
   │ "How do I parse JSON in Go?"        │    │ "Should we use event sourcing?"     │
   │  → 5 upvoted answers                │    │  → "It depends" (correctly)         │
   │  → all roughly equivalent           │    │  → answer depends on:               │
   │  → near-universally true            │    │      - team size                    │
   │                                     │    │      - data volume                  │
   └─────────────────────────────────────┘    │      - audit requirements           │
                                              │      - operational maturity         │
                                              │      - regulatory environment       │
                                              │      - existing systems             │
                                              │      - 5-year plan                  │
                                              └─────────────────────────────────────┘
                                              No Stack Overflow answer ever fits
                                              your snowflake situation.
```

Every architectural problem is novel because the *context* is novel. Architects should expect *zero* applicable precedents for most decisions they make.

#### 3. Architectural Decision Records (ADRs) — the load-bearing artifact

A short markdown file per significant architectural decision. The canonical sections:

| Section | Contents |
|---|---|
| **Title** | Short name, e.g. "ADR-007: Adopt Postgres over MongoDB for primary store" |
| **Status** | Proposed / Accepted / Deprecated / Superseded-by ADR-N |
| **Context** | The forces in play — business, technical, team — at the time of the decision |
| **Decision** | The chosen approach |
| **Consequences** | What this implies — good, bad, neutral; what we now can/can't do |
| **Alternatives considered** | What we *didn't* choose and why |

```
   The decision itself is less valuable than the why behind it.

   Bad architecture history:                      Good architecture history (ADRs):
   ┌──────────────────────┐                       ┌──────────────────────────────────┐
   │ "We use Kafka"       │                       │ ADR-003: Chose Kafka over RabbitMQ│
   │  (no one remembers   │                       │ Context: needed >100K msg/s,     │
   │   why or when)       │                       │  durable replay, single team      │
   │                      │                       │  ops capacity, existing JVM stack │
   │ Re-litigated every   │                       │ Decision: Kafka                   │
   │  year, 5 hours per   │                       │ Consequences: heavier ops, locks  │
   │  meeting, never      │                       │  us to JVM-friendly serializers   │
   │  converges           │                       │ Alternatives: RabbitMQ (lighter   │
   │                      │                       │  but no replay), SQS (vendor lock)│
   └──────────────────────┘                       └──────────────────────────────────┘
                                                  → Future debate has a baseline.
                                                  → "Has anything in Context changed?"
```

ADRs are **append-only and immutable** — when a decision is superseded, the new ADR references the old, but the old is never deleted. Same discipline as git history; the *trail* is the artifact.

#### 4. Fitness Functions — make architectural qualities testable

```
   Architectural quality:    "modules should not have circular dependencies"
        │
        ▼
   Fitness function:         test_no_circular_dependencies():
                                graph = build_module_graph()
                                assert nx.is_directed_acyclic_graph(graph)
        │
        ▼
   Runs in:                  CI, every PR
        │
        ▼
   Outcome:                  Architectural drift detected at PR time, not at the
                             "why did everything become tightly coupled?" retrospective
                             three years later.
```

| Quality | Example fitness function |
|---|---|
| Modularity | Import-graph DAG assertion (no cycles between modules) |
| Performance | `p99_latency < 200ms` measured in load test |
| Security | All HTTP responses include CSP / HSTS headers |
| Coupling | Static analysis: max afferent coupling per module ≤ N |
| Data integrity | All migrations are reversible (apply then revert) |
| Cost | Cloud spend per environment ≤ budget (alarm) |

> Fitness functions are to architecture what **unit tests are to design**: they convert assertions about quality into *executable* assertions about reality. Without them, "we value modularity" is aspirational; with them, it's enforced.

#### 5. The trade-off as the unit of analysis

| Decision | Gains | Costs |
|---|---|---|
| Microservices | Independent deployment, team autonomy | Network failures, data consistency, ops complexity |
| Event sourcing | Audit log free, time-travel queries | Storage growth, eventual consistency, projection complexity |
| Sync over async | Simpler reasoning | Worse latency and availability |
| Strong consistency | Easier app code | Worse latency and partition tolerance (CAP) |
| Caching layer | Lower DB load | Cache invalidation, stale data, debugging asymmetry |

The architect's job is **making the trade-off explicit and *recording it*** — never pretending you got something for free.

#### 6. The Fred Brooks ghost — "No Silver Bullet"

| Brooks 1986 | SAHP 2021 |
|---|---|
| "No single technique will deliver 10× productivity in a decade" | "No best practice solves your specific problem" |
| Distinguished essence (hard) from accident (incidental) | Distinguished trade-off (real) from preference (rhetorical) |
| Argued against rhetoric of revolution | Argues against rhetoric of best practices |

Same message: real engineering is **judgment under irreducible complexity**, and the artifacts that survive (essays, ADRs) are the ones that capture *judgment*, not *answers*.

### Where this shows up in real systems
- **AWS / Azure Well-Architected Framework** — codified ADRs at industry scale; pillars (Operational Excellence, Security, Reliability, Performance, Cost, Sustainability) with explicit trade-off discussions.
- **ThoughtWorks Technology Radar** — quarterly "Adopt / Trial / Assess / Hold" judgments with reasoning. Same shape as ADR at industry-trend scale.
- **Spotify's `backstage.io`** — runs fitness-function-like checks across all services (deprecated lib usage, ownership, SLO compliance).
- **Architecture Decision Records repos on GitHub** — Spotify, Heroku, ThoughtWorks publish theirs publicly.
- **CDK / Pulumi tests** — fitness functions against the *cloud topology itself* ("no S3 bucket can be public," "every RDS instance has backups").

### Diagnostic questions
1. *"Why is 'it depends' often the *correct* architectural answer?"* — Because the right answer requires knowing the context (team size, scale, regulatory environment, existing systems, time horizon), and no general answer can encode all those. "It depends" is not evasion — it's the prerequisite to any honest answer. The follow-up is "on what?" (Wrong: "the architect is hedging" — sometimes yes, but the *form* of the answer is correct.)
2. *"What makes an ADR durable when the codebase keeps changing?"* — Its **Context** section captures the *forces at decision time*, not the code. When you read it five years later, you ask "do those forces still apply?" — not "does this code still compile?" Code rots; recorded context becomes evidence about a moment in history. (Wrong: "ADRs are documentation" — they're more like primary historical sources.)
3. *"How is a fitness function different from a unit test?"* — Unit tests verify *functional correctness* of a unit; fitness functions verify *architectural characteristics* of the system (modularity, performance, security, cost). The unit of analysis is the *non-functional requirement*. (Wrong: "fitness functions are integration tests" — overlap exists but the *intent* differs.)
4. *"Why are most architectural problems 'snowflakes'?"* — Because architectural decisions are constrained by an organization's *specific* history, team capabilities, regulatory environment, customer base, and existing systems — and the combinatorics of those factors mean any two organizations face *different* problems even if they look superficially similar. (Wrong: "every problem is unique" — too strong; the *combinations of constraints* are unique even when the elements aren't.)
5. *"Why does the chapter insist on writing down the *alternatives considered*?"* — Because the strongest evidence that a decision was *informed* is showing what was rejected and why. A decision without alternatives looks like a default; a decision with rejected alternatives looks like judgment. Also helps when the rejected alternative becomes relevant again. (Wrong: "for documentation completeness" — it's specifically about *evaluating revisitation* later.)

### See also
- SAHP Ch.2+ — the rest of the book applies these principles to specific hard parts (data architecture, communication, sagas, microservices granularity).
- TPP 2026-05-19 (Estimating, Tracer Bullets) — individual-level analogue: same epistemic humility about uncertainty, applied to one developer's daily work.
- DDIA 2026-05-19, 2026-05-20 — domain-specific trade-off catalogues for data systems; concrete examples of the "no best practice" claim.
- REF 2026-05-21 (Refactoring Ch.1) — design-layer analogue: small, recorded, reversible steps; ADRs are to architecture what commit messages + Fowler's micro-steps are to design.
- LDDD (Learning DDD) — the domain modeling complement; ADRs capture *why* a bounded context was drawn where it was.

