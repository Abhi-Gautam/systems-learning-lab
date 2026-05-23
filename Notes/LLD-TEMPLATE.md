# LLD Note Entry Template

Every entry in `Notes/lld-notes.md` follows this structure. **New entries are inserted at the top of the file** — newest first. Prior entries are append-only.

Sections are **all mandatory** unless explicitly marked conditional. The depth contract for Deep-dive sections matches HLD — see `docs/superpowers/specs/2026-05-22-design-skills-architecture.md` §8.

Total target length: **~350 lines per entry** (250 lines in 2 Deep dives, ~100 lines in surrounding sections).

Language for code skeletons: **Go** (matches DSG + LGO in the reading list).

---

## [YYYY-MM-DD] {ID} · {Short Title} · {company-tags}

- {Pattern 1 — e.g. "State machine per elevator"}
- {Pattern 2 — e.g. "Strategy for scheduling algorithm"}
- {Concurrency model summary — e.g. "Channels + worker goroutine per elevator"}

### Problem (as asked)

Quote the catalog `ask` field verbatim.

**Clarifying questions to ask back** (numbered list of 5):
1. {Question about scope: which features in-scope?}
2. {Question about scale or constraints: how many actors, throughput?}
3. {Question about concurrency: single-threaded or multi-threaded?}
4. {Question about extensibility: what's likely to change?}
5. {Question about persistence / state durability}

### Requirements teardown

| Functional | Non-functional |
|---|---|
| {capability 1} | {concurrency requirement} |
| {capability 2} | {SLA / latency target} |
| {capability 3} | {extensibility expectations} |
| {capability 4} | {testability constraint} |

### Actor & entity inventory

One line per class. Keep to ≤8 entries (more = the model is fragmented).

| Entity | Responsibility (single sentence) |
|---|---|
| `ClassA` | {what it owns and decides} |
| `ClassB` | {...} |
| `ClassC` | {...} |
| ... | ... |

### Class diagram

ASCII or mermaid. Inheritance vs composition explicit. Aggregation arrows labeled.

```
                ┌────────────────┐
                │  ElevatorBank  │  ◄── aggregate root
                └────┬─────┬─────┘
                     │     │
        owns N       │     │       holds 1
       ┌─────────────┘     └─────────────┐
       ▼                                  ▼
┌─────────────┐                  ┌─────────────────┐
│  Elevator   │                  │ Scheduler       │  ◄── strategy
└─────┬───────┘                  │ (interface)     │
      │ has state                └────────┬────────┘
      ▼                                   │ impls
┌────────────────┐                ┌───────┴────────┐
│ ElevatorState  │                │ NearestCarSched│
│ (enum)         │                │ LookSched      │
└────────────────┘                │ FCFSSched      │
                                  └────────────────┘
```

### Patterns applied + alternatives rejected

For each pattern used, name it and say why. Then name the patterns you considered but rejected.

| Pattern | Where | Why this over alternatives |
|---|---|---|
| Strategy | `Scheduler` interface + 3 impls | Algorithm changes more than callers; Visitor would couple algorithms to elevator internals |
| State | `Elevator` internal state machine | Cleaner than nested if/else; State pattern handles transition rules |
| Observer | Floor button → bank notification | Decouples UI from scheduler |

**Rejected patterns:**
- Singleton scheduler — testability nightmare; instance-per-bank is fine.
- Visitor for movement logic — overkill; State enum suffices.

### Code skeleton (Go)

Real signatures, no bodies (or stub bodies). **50–80 lines.** This is the spine — the interviewer will fill in 1–2 methods live.

```go
package elevator

import (
    "context"
    "sync"
    "time"
)

// Direction is the elevator's current movement direction.
type Direction int

const (
    DirIdle Direction = iota
    DirUp
    DirDown
)

// Request models a rider's call (from a floor, or destination from inside).
type Request struct {
    FromFloor int
    ToFloor   int
    Submitted time.Time
}

// Elevator is one car in the bank.
type Elevator struct {
    mu         sync.Mutex
    id         int
    floor      int
    direction  Direction
    requests   chan Request
    state      State
}

// State is the elevator's behavioral state (Idle, Moving, OpeningDoors, ...).
type State interface {
    OnRequest(*Elevator, Request)
    OnArrival(*Elevator, int)
}

// Scheduler decides which elevator handles a new request.
type Scheduler interface {
    Assign(req Request, elevators []*Elevator) *Elevator
}

// NearestCarScheduler — picks elevator with smallest weighted distance.
type NearestCarScheduler struct{}

func (s *NearestCarScheduler) Assign(req Request, elevators []*Elevator) *Elevator {
    // Implementation outlined in Deep dive 1.
    return nil
}

// ElevatorBank is the aggregate root — owns all elevators + the scheduler.
type ElevatorBank struct {
    elevators []*Elevator
    scheduler Scheduler
    inbox     chan Request
}

func NewElevatorBank(n int, sched Scheduler) *ElevatorBank { /* ... */ return nil }

// Submit places a request into the bank; returns immediately.
func (b *ElevatorBank) Submit(ctx context.Context, req Request) error {
    select {
    case b.inbox <- req:
        return nil
    case <-ctx.Done():
        return ctx.Err()
    }
}

// run is the bank's coordinator loop — pulls requests, assigns elevators.
func (b *ElevatorBank) run(ctx context.Context) { /* ... */ }
```

### Deep dive — 2 mechanisms, ~120 lines each

For each, follow the **6-subsection depth contract** (same as HLD):

#### Deep dive 1: {primary mechanism — e.g. scheduling algorithm}

**1. Why does this mechanism exist?**
{Problem before. Alternative rejected. Why this design.}

**2. Walk it concretely.**
{Numbered scenario with named values.}

```
Initial state: E1 idle at floor 5, E2 idle at floor 12, E3 moving up at floor 8.
New request: floor 6 → 14 (up direction).

NearestCarScheduler:
  E1: distance 1, idle → score = 1
  E2: distance 6, idle, wrong direction → score = 6 + 10 penalty = 16
  E3: distance 2, moving up (same direction) → score = 2 - 5 bonus = -3
  Winner: E3 (already moving up, just 2 floors away).
```

**3. The cost you're accepting.**

| Property gained | Cost paid |
|---|---|
| Minimizes wait time for typical loads | Pathological cases (all requests at top floor) starve idle cars |
| Simple to implement | Doesn't optimize energy / wear |
| Easy to reason about | Hard to add SLA constraints (VIP, emergency) |

**4. Failure modes the interviewer will drill into.**

| Failure | What goes wrong | Mitigation |
|---|---|---|
| Q1: All requests at floor 20 | E3 keeps winning; E1/E2 idle | Add idle-time penalty |
| Q2: Two requests submitted simultaneously | Race in `Assign` | Mutex on bank; or single-coordinator goroutine |
| Q3: Elevator crashes mid-trip | Held requests lost | Persistent request log; re-submit on recovery |
| Q4: SCAN vs LOOK — when to prefer? | LOOK reverses sooner | LOOK on residential; SCAN on freight where energy matters less |

**5. How to derive it from first principles.**
{Numbered, 6-8 steps.}

**6. Where this shows up in production.**
- **Linux CFS scheduler**: uses red-black tree of runqueues; "scheduling" here means picking next process — analogous to nearest-car for CPU time.
- **Kubernetes default scheduler**: scoring framework — multiple plugins contribute scores; final pick is highest-scoring node. Same architecture as a pluggable elevator scheduler.
- **Real elevators (Otis, KONE)**: Destination dispatch — riders pre-enter destination at lobby; group-control algorithm assigns car *before* boarding. Improves throughput 20-30% in tall buildings.

#### Deep dive 2: {secondary mechanism — e.g. concurrency model}

[Same 6-subsection structure.]

### Concurrency model

How does the system stay consistent under concurrent submissions, dispatches, and elevator state changes?

```
                  ┌──────────────────────┐
                  │  Inbox (chan)        │  ◄── bounded buffer
                  └──────────┬───────────┘
                             │ 1 reader
                             ▼
                  ┌──────────────────────┐
                  │  Bank coordinator    │  (single goroutine)
                  │  (Scheduler.Assign)  │
                  └──────────┬───────────┘
                             │ dispatch via per-elevator chan
            ┌────────────────┼────────────────┐
            ▼                ▼                ▼
       ┌─────────┐      ┌─────────┐      ┌─────────┐
       │ E1 loop │      │ E2 loop │      │ E3 loop │
       │ (goroutine) │  │ ...     │      │ ...     │
       └─────────┘      └─────────┘      └─────────┘
```

**Deadlock-freedom argument:**
- Bank coordinator reads from one inbox, writes to N elevator channels — no cycle.
- Elevator goroutines only read their own channel; never write back to the bank.
- Therefore: no circular wait → no deadlock.

### Extension points

For each "now add..." prompt, show *where* the change lands and *what stays the same*.

| Extension | Where it lands | What doesn't change |
|---|---|---|
| Priority floors | New `Scheduler` impl: `PriorityScheduler` | `Elevator`, `ElevatorBank`, `State` |
| Emergency stop | New `State`: `EmergencyState`; signal goroutine on each elevator | Scheduler, request flow |
| VIP rider SLA | Scheduler wraps existing impl with deadline check | Bus interface, elevator state |
| Destination dispatch | New entry point on `ElevatorBank.SubmitWithDestination(...)` | Internal scheduling unchanged |

### SOLID self-audit

| Principle | Did we obey? | Why / why not |
|---|---|---|
| Single Responsibility | ✅ | Elevator owns motion, Scheduler owns assignment, Bank owns coordination |
| Open/Closed | ✅ | New schedulers slot in via `Scheduler` interface; no `Elevator` changes |
| Liskov | ✅ | All `Scheduler` impls satisfy `Assign(Request, []*Elevator) *Elevator` |
| Interface Segregation | ✅ | `Scheduler` is 1-method; `State` is 2-method — narrow |
| Dependency Inversion | ✅ | `ElevatorBank` depends on `Scheduler` interface, not concrete |

### Where this shows up in production

- **{Named system 1}**: {analogous structure}
- **{Named system 2}**: {analogous structure}
- **{Named system 3}**: {analogous structure}

### Common follow-ups

5 "now add..." prompts and their answers (2–3 lines each).

1. **{Follow-up 1}** — {answer}
2. ...

### Diagnostic questions

5 questions, each with a wrong-answer interpretation.

1. **{Q}** — wrong: {weak pattern}
2. ...

---

## Title format

```
## [YYYY-MM-DD] {ID} · {Short Title} · {company-tags}
```

- **ID**: `L01` through `L45` — immutable; identifies the row in `Problems/lld.json`.
- **Short Title**: 2–6 words.
- **Company-tags**: bold the signature company if any.

## Language choice

LLD code skeletons are in **Go**. Rationale: matches DSG (Distributed Services with Go) and LGO (Learning Go) already in the reading list — depth in one language beats breadth across three. Interfaces in Go map cleanly to Strategy pattern; channels make concurrency idioms explicit.

If a problem screams for inheritance (e.g., Chess pieces), Go's embedding is the equivalent — show both the struct embedding and the interface satisfaction.

## Visual + depth contract

Same as HLD template — tables > prose, ≥1 diagram per Deep dive, low-level mechanisms get expanded (memory ordering, channel internals, goroutine scheduling) when they appear.
