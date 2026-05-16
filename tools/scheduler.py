#!/usr/bin/env python3
"""
scheduler.py — multiprocessor reading scheduler.

Two modes:
  * Simulation (pure, deterministic): `day N`, `range A B`, `status`.
    Runs forward from Day 0 with no state I/O. Use for "what does day 100 look like?".
  * Live (state-backed): `init`, `today`, `tick`, `override`.
    Reads/writes Schedule/state.json. Use for actual daily study practice.

Model: 4 typed processors (T1..T4), all touched every day.
- T1 = systems foundations (45 min)   T2 = craft (35 min)
- T3 = arch/Go/algos (25 min)          T4 = supplemental (15 min)
- Quantum: yield at chapter end on day >=5, hard-cap at day 10.
- Aging within tier: when a slot opens, pick the tier-mate idle the longest.
- DSG gated behind LGO completion.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass, field, asdict
from datetime import date, timedelta
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GLOSSARY_DIR = ROOT / "Glossaries"
SCHEDULE_DIR = ROOT / "Schedule"
STATE_PATH = SCHEDULE_DIR / "state.json"
LOG_PATH = SCHEDULE_DIR / "log.md"

START_DATE = date(2026, 5, 11)  # Day 0

TIER_TIME = {"T1": 45, "T2": 35, "T3": 25, "T4": 15}

# Per-book pages-per-session. Calibrated to ~0.25–0.5 pp/min for technical reading,
# weighted by book density. Override at runtime via Schedule/state.json `rates`.
DEFAULT_RATES = {
    # T1 (45 min): dense systems texts
    "DDIA": 14, "OSTEP": 14, "COD": 11, "N2T": 12,
    # T2 (35 min): engineering craft, lighter prose
    "CC": 16, "TPP": 16, "REF": 14, "WELC": 14, "WPF": 14,
    # T3 (25 min): mixed depth
    "LGO": 10, "DSG": 10, "SAHP": 9, "LDDD": 9, "CLRS": 6,
    # T4 (15 min)
    "DBI": 5, "CUDA": 4,
}

# Initial book on each slot (Day 0).
INITIAL_SLOT = {"T1": "COD", "T2": "CC", "T3": "LGO", "T4": "DBI"}

# Tier membership in priority order (used as aging tie-break).
TIER_BOOKS = {
    "T1": ["DDIA", "OSTEP", "COD", "N2T"],
    "T2": ["CC", "TPP", "REF", "WELC", "WPF"],
    "T3": ["LGO", "CLRS", "SAHP", "LDDD", "DSG"],
    "T4": ["DBI", "CUDA"],
}

# Books gated behind another book's completion.
GATES = {"DSG": "LGO"}

# Quantum policy
SOFT_YIELD_DAY = 5  # earliest day a chapter-end can trigger yield
HARD_CAP_DAY = 10   # forced yield at this day


# ---------- glossary helpers ----------

FRONT_MATTER = ("front cover", "preface", "contents", "copyright", "acknowledg",
                "foreword", "dedication", "about the", "title page", "trademarks",
                "to everyone", "to educators", "to students", "final words",
                "references\n", "halftitle", "frontmatter")


def is_front_matter(title: str) -> bool:
    t = title.lower().strip()
    return any(t.startswith(p) or t == p.strip() for p in FRONT_MATTER)


def load_glossary(alias: str) -> dict:
    p = GLOSSARY_DIR / f"{alias.lower()}.json"
    return json.loads(p.read_text())


def chapter_max_level(g: dict) -> int:
    """Shallow TOCs (max level<=2): treat level 2 as chapter-boundary too.
    Deep TOCs (level 3+, like DDIA): only level 1 counts as chapter."""
    if not g["entries"]:
        return 1
    return 1 if max(e["level"] for e in g["entries"]) >= 3 else 2


def chapter_entries(g: dict) -> list[dict]:
    cl = chapter_max_level(g)
    return [e for e in g["entries"] if e["level"] <= cl and not is_front_matter(e["title"])]


def first_chapter_page(g: dict) -> int:
    ce = chapter_entries(g)
    return ce[0]["page"] if ce else 1


def locate(g: dict, page: int) -> tuple[str, int]:
    current = None
    for e in chapter_entries(g):
        if e["page"] > page:
            break
        current = (e["title"], e["page"])
    return current if current else ("(front matter)", 0)


def chapter_changed(g: dict, old_page: int, new_page: int) -> bool:
    return locate(g, old_page) != locate(g, new_page)


# ---------- state ----------

@dataclass
class SlotState:
    book: str | None
    page: int
    days_on_slot: int = 0
    chapters_done_this_quantum: int = 0

@dataclass
class BookState:
    last_active: int = -1     # day index; -1 = never
    page: int = 0             # bookmark when off-slot
    completed: bool = False


@dataclass
class SchedulerState:
    day: int = 0
    slots: dict[str, SlotState] = field(default_factory=dict)
    books: dict[str, BookState] = field(default_factory=dict)
    rates: dict[str, int] = field(default_factory=lambda: dict(DEFAULT_RATES))


def init_state() -> SchedulerState:
    s = SchedulerState()
    for alias in sum(TIER_BOOKS.values(), []):
        g = load_glossary(alias)
        s.books[alias] = BookState(page=first_chapter_page(g))
    for tier, book in INITIAL_SLOT.items():
        s.slots[tier] = SlotState(book=book, page=s.books[book].page)
        s.books[book].last_active = 0
    return s


# ---------- persistence ----------

def save_state(s: SchedulerState) -> None:
    SCHEDULE_DIR.mkdir(exist_ok=True)
    payload = {
        "start_date": str(START_DATE),
        "day": s.day,
        "rates": s.rates,
        "slots": {t: asdict(v) for t, v in s.slots.items()},
        "books": {a: asdict(v) for a, v in s.books.items()},
    }
    STATE_PATH.write_text(json.dumps(payload, indent=2))


def load_state() -> SchedulerState:
    if not STATE_PATH.exists():
        raise FileNotFoundError(f"No state at {STATE_PATH}. Run `scheduler.py init` first.")
    p = json.loads(STATE_PATH.read_text())
    s = SchedulerState(day=p["day"], rates=p.get("rates", dict(DEFAULT_RATES)))
    s.slots = {t: SlotState(**v) for t, v in p["slots"].items()}
    s.books = {a: BookState(**v) for a, v in p["books"].items()}
    return s


def append_log(rec: dict) -> None:
    SCHEDULE_DIR.mkdir(exist_ok=True)
    line_parts = []
    for t in ("T1", "T2", "T3", "T4"):
        sl = rec["slots"][t]
        if sl["book"] is None:
            line_parts.append(f"{t}: —")
        else:
            line_parts.append(f"{t} {sl['book']} pp.{sl['pages']}")
    line = f"- **Day {rec['day']} · {rec['date']}** — " + " · ".join(line_parts) + "\n"
    if not LOG_PATH.exists():
        LOG_PATH.write_text("# Schedule Log\n\n")
    with LOG_PATH.open("a") as f:
        f.write(line)


# ---------- core scheduling ----------

def tier_of(book: str) -> str:
    for t, bs in TIER_BOOKS.items():
        if book in bs:
            return t
    raise KeyError(book)


def eligible_in_tier(tier: str, state: SchedulerState, on_slot_books: set[str]) -> list[str]:
    out = []
    for b in TIER_BOOKS[tier]:
        if b in on_slot_books or state.books[b].completed:
            continue
        gate = GATES.get(b)
        if gate and not state.books[gate].completed:
            continue
        out.append(b)
    return out


def pick_next(tier: str, state: SchedulerState, on_slot_books: set[str]) -> str | None:
    candidates = eligible_in_tier(tier, state, on_slot_books)
    if not candidates:
        return None
    # aging: smallest last_active wins (longest idle). Tie-break by TIER_BOOKS order.
    return min(candidates, key=lambda b: (state.books[b].last_active, TIER_BOOKS[tier].index(b)))


def should_yield(slot: SlotState, glossary: dict) -> bool:
    if slot.book is None:
        return False
    if slot.days_on_slot >= HARD_CAP_DAY and slot.chapters_done_this_quantum >= 1:
        return True
    if slot.days_on_slot >= SOFT_YIELD_DAY and slot.chapters_done_this_quantum >= 1:
        return True
    return False


def advance_one_day(state: SchedulerState, actual_ends: dict[str, int] | None = None) -> dict:
    """Mutate state by one day. `actual_ends` maps slot→end-page (LLM-adjusted); falls back to rate."""
    actual_ends = actual_ends or {}
    state.day += 1
    day_record = {"day": state.day, "date": str(START_DATE + timedelta(days=state.day)), "slots": {}}

    for tier in ("T1", "T2", "T3", "T4"):
        slot = state.slots[tier]
        if slot.book is None:
            day_record["slots"][tier] = {"book": None, "note": "tier exhausted"}
            continue

        g = load_glossary(slot.book)
        start_page = slot.page
        rate = state.rates.get(slot.book, 10)
        planned_end = min(start_page + rate - 1, g["pdf_pages"])
        end_page = actual_ends.get(tier, planned_end)
        end_page = min(max(end_page, start_page), g["pdf_pages"])
        chap_title, chap_page = locate(g, start_page)
        finished_book = end_page >= g["pdf_pages"]
        finished_chapter = chapter_changed(g, start_page, end_page + 1) or finished_book

        day_record["slots"][tier] = {
            "book": slot.book,
            "minutes": TIER_TIME[tier],
            "pages": f"{start_page}-{end_page}",
            "planned_end": planned_end,
            "actual_end": end_page,
            "divergence": end_page - planned_end,
            "chapter": chap_title,
            "chapter_page": chap_page,
            "days_on_slot": slot.days_on_slot + 1,
            "finished_chapter": finished_chapter,
            "finished_book": finished_book,
        }

        # commit per-day mutations
        new_page = end_page + 1
        slot.page = new_page
        slot.days_on_slot += 1
        if finished_chapter:
            slot.chapters_done_this_quantum += 1
        state.books[slot.book].last_active = state.day
        state.books[slot.book].page = new_page

        # eviction or yield?
        on_slot = {state.slots[t].book for t in ("T1", "T2", "T3", "T4") if state.slots[t].book}
        if finished_book:
            state.books[slot.book].completed = True
            on_slot.discard(slot.book)
            nxt = pick_next(tier, state, on_slot)
            day_record["slots"][tier]["eviction"] = f"book complete → {nxt or '(tier exhausted)'}"
            if nxt:
                state.slots[tier] = SlotState(book=nxt, page=state.books[nxt].page)
                state.books[nxt].last_active = state.day
            else:
                state.slots[tier] = SlotState(book=None, page=0)
        elif should_yield(slot, g):
            on_slot.discard(slot.book)
            nxt = pick_next(tier, state, on_slot)
            if nxt and nxt != slot.book:
                day_record["slots"][tier]["yield"] = f"{slot.book} → {nxt}"
                state.slots[tier] = SlotState(book=nxt, page=state.books[nxt].page)
                state.books[nxt].last_active = state.day
            # if only this book remains in tier, keep it (reset quantum counter)
            else:
                slot.days_on_slot = 0
                slot.chapters_done_this_quantum = 0
    return day_record


def simulate_to(target_day: int) -> tuple[SchedulerState, list[dict]]:
    state = init_state()
    history = []
    for _ in range(target_day):
        history.append(advance_one_day(state))
    return state, history


# ---------- output ----------

def print_day(rec: dict) -> None:
    print(f"\n=== Day {rec['day']} · {rec['date']} ===")
    for tier in ("T1", "T2", "T3", "T4"):
        s = rec["slots"][tier]
        if s["book"] is None:
            print(f"  {tier} ({TIER_TIME[tier]:>2}min):  — {s.get('note', '')}")
            continue
        flags = []
        if s.get("finished_chapter"): flags.append("✓chapter")
        if s.get("finished_book"):    flags.append("★BOOK DONE")
        if s.get("yield"):            flags.append(f"⇄ yield: {s['yield']}")
        if s.get("eviction"):         flags.append(s["eviction"])
        flag_str = ("  [" + " · ".join(flags) + "]") if flags else ""
        print(f"  {tier} ({TIER_TIME[tier]:>2}min): {s['book']:<5} pp.{s['pages']:<10} "
              f"{s['chapter']} (ch.p.{s['chapter_page']}) · day {s['days_on_slot']}/{HARD_CAP_DAY}{flag_str}")


def cmd_day(args) -> int:
    state, hist = simulate_to(args.n)
    if args.n == 0:
        print(f"=== Day 0 · {START_DATE} (initial pool) ===")
        for tier in ("T1", "T2", "T3", "T4"):
            b = INITIAL_SLOT[tier]
            g = load_glossary(b)
            ch, cp = locate(g, state.books[b].page)
            print(f"  {tier} ({TIER_TIME[tier]:>2}min): {b:<5} starting p.{state.books[b].page}  ({ch})")
        return 0
    print_day(hist[-1])
    return 0


def cmd_range(args) -> int:
    state, hist = simulate_to(args.b)
    for rec in hist[args.a - 1 : args.b]:
        print_day(rec)
    return 0


def cmd_status(args) -> int:
    print(f"Start date: {START_DATE} (Day 0)")
    live = " · live state present" if STATE_PATH.exists() else " · no live state (simulation only)"
    print(f"State: {STATE_PATH}{live}")
    print("Initial slots:")
    for t, b in INITIAL_SLOT.items():
        print(f"  {t} ({TIER_TIME[t]:>2}min, {DEFAULT_RATES[b]} pp/sess): {b}")
    print("Tier ready-queues (priority order, aging breaks ties):")
    for t, bs in TIER_BOOKS.items():
        q = [b for b in bs if b != INITIAL_SLOT[t]]
        print(f"  {t}: {q}")
    print(f"Gates: {GATES}")
    print(f"Quantum: soft yield day≥{SOFT_YIELD_DAY}+chapter, hard cap day {HARD_CAP_DAY}")
    return 0


# ---------- live-mode commands ----------

def cmd_init(args) -> int:
    if STATE_PATH.exists() and not args.force:
        print(f"State already at {STATE_PATH}; pass --force to overwrite.", file=sys.stderr)
        return 2
    s = init_state()
    save_state(s)
    print(f"Wrote initial state to {STATE_PATH} (Day 0, {START_DATE}).")
    return 0


def cmd_today(args) -> int:
    s = load_state()
    print(f"Today is Day {s.day} · {START_DATE + timedelta(days=s.day)}")
    print("Slots (no mutation):")
    for tier in ("T1", "T2", "T3", "T4"):
        slot = s.slots[tier]
        if slot.book is None:
            print(f"  {tier}: — (tier exhausted)")
            continue
        g = load_glossary(slot.book)
        ch, cp = locate(g, slot.page)
        rate = s.rates.get(slot.book, 10)
        end = min(slot.page + rate - 1, g["pdf_pages"])
        print(f"  {tier} ({TIER_TIME[tier]:>2}min): {slot.book:<5} read pp.{slot.page}-{end}  "
              f"({ch}, ch.p.{cp}) · day {slot.days_on_slot}/{HARD_CAP_DAY}")
    return 0


def cmd_tick(args) -> int:
    s = load_state()
    actual = {}
    for tier in ("T1", "T2", "T3", "T4"):
        v = getattr(args, tier.lower(), None)
        if v is not None:
            actual[tier] = v
    rec = advance_one_day(s, actual)
    save_state(s)
    append_log(rec)
    print_day(rec)
    if actual:
        print("\nLLM-adjusted ends:", ", ".join(f"{k}={v}" for k, v in actual.items()))
    print(f"State updated to Day {s.day}. Log: {LOG_PATH}")
    return 0


def cmd_override(args) -> int:
    s = load_state()
    if args.slot not in s.slots:
        print(f"Unknown slot {args.slot}", file=sys.stderr); return 2
    if args.book is not None:
        if args.book not in s.books:
            print(f"Unknown book {args.book}", file=sys.stderr); return 2
        s.slots[args.slot] = SlotState(book=args.book, page=s.books[args.book].page)
    if args.page is not None:
        s.slots[args.slot].page = args.page
        s.books[s.slots[args.slot].book].page = args.page
    save_state(s)
    print(f"Override applied; {args.slot} → {s.slots[args.slot].book} p.{s.slots[args.slot].page}")
    return 0


def cmd_rate(args) -> int:
    s = load_state()
    if args.book not in s.rates and args.book not in DEFAULT_RATES:
        print(f"Unknown book {args.book}", file=sys.stderr); return 2
    s.rates[args.book] = args.pages
    save_state(s)
    print(f"Rate for {args.book} set to {args.pages} pp/session.")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)

    d = sub.add_parser("day", help="(sim) schedule for day N from Day 0")
    d.add_argument("n", type=int); d.set_defaults(func=cmd_day)

    r = sub.add_parser("range", help="(sim) print every day in [A,B]")
    r.add_argument("a", type=int); r.add_argument("b", type=int); r.set_defaults(func=cmd_range)

    st = sub.add_parser("status", help="config + live-state presence")
    st.set_defaults(func=cmd_status)

    ini = sub.add_parser("init", help="(live) write initial Schedule/state.json")
    ini.add_argument("--force", action="store_true"); ini.set_defaults(func=cmd_init)

    td = sub.add_parser("today", help="(live) show today's slots without mutating state")
    td.set_defaults(func=cmd_today)

    tk = sub.add_parser("tick", help="(live) advance one day, persist, log. "
                                     "Pass --T1/--T2/--T3/--T4 N to override rate with actual end page.")
    for t in ("T1", "T2", "T3", "T4"):
        tk.add_argument(f"--{t.lower()}", type=int, metavar="END_PAGE",
                        help=f"actual end page for {t} (defaults to rate-based)")
    tk.set_defaults(func=cmd_tick)

    ov = sub.add_parser("override", help="(live) manually set slot's book/page when reality diverges")
    ov.add_argument("slot", choices=("T1", "T2", "T3", "T4"))
    ov.add_argument("--book"); ov.add_argument("--page", type=int)
    ov.set_defaults(func=cmd_override)

    rt = sub.add_parser("rate", help="(live) calibrate per-book pages/session after real reading")
    rt.add_argument("book"); rt.add_argument("pages", type=int)
    rt.set_defaults(func=cmd_rate)

    args = ap.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
