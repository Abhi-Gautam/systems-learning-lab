#!/usr/bin/env python3
"""
design_scheduler.py — daily interview-design problem scheduler.

Sibling to scheduler.py (which handles book reading). This module manages
the /design-today ritual:
  - Mon/Wed/Fri → HLD problem
  - Tue/Thu/Sat → LLD problem
  - Sun → revisit problem from spaced-repetition queue

Commands:
  today                          → print today's pick (no state mutation)
  tick --problem ID --confidence N  → record completion, advance day
  status                         → print queue depths, recent completions
  revisit                        → list problems with revisit_due ≤ today
  override --problem ID          → force a specific problem on next 'today'
  company --tag TAG              → filter next pick to problems tagged for company

State at schedule/design_state.json. Problems at problems/{hld,lld}.json.

Convention follows tools/scheduler.py (book scheduler): pure functions where
possible, single state load/save at command boundary.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass, field
from datetime import date, datetime, timedelta
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PROBLEMS_DIR = ROOT / "problems"
SCHEDULE_DIR = ROOT / "schedule"
STATE_PATH = SCHEDULE_DIR / "design_state.json"
LOG_PATH = SCHEDULE_DIR / "log.md"

# Weekday → branch routing.  Mon=0 … Sun=6.
WEEKDAY_BRANCH = {
    0: "hld",  # Mon
    1: "lld",  # Tue
    2: "hld",  # Wed
    3: "lld",  # Thu
    4: "hld",  # Fri
    5: "lld",  # Sat
    6: "revisit",  # Sun
}

# Confidence → next-revisit-interval (days).
REVISIT_INTERVALS = {
    0: 3,
    1: 3,
    2: 7,
    3: 7,
    4: 21,
    5: 60,
}


# ---------- problem catalog helpers ----------


def load_catalog(kind: str) -> dict:
    """Read problems/{hld,lld}.json."""
    p = PROBLEMS_DIR / f"{kind}.json"
    return json.loads(p.read_text())


def problem_by_id(pid: str) -> dict | None:
    """Look up a problem row across both catalogs by id."""
    kind = "hld" if pid.startswith("H") else "lld"
    catalog = load_catalog(kind)
    for row in catalog["problems"]:
        if row["id"] == pid:
            return row
    return None


# ---------- state ----------


@dataclass
class State:
    start_date: str
    day: int
    queues: dict[str, list[str]] = field(default_factory=lambda: {"hld": [], "lld": []})
    last_seen: dict[str, str] = field(default_factory=dict)
    confidence: dict[str, int] = field(default_factory=dict)
    revisit_due: dict[str, str] = field(default_factory=dict)
    history: list[dict] = field(default_factory=list)
    completed: dict[str, int] = field(
        default_factory=lambda: {"hld_count": 0, "lld_count": 0, "revisits_count": 0}
    )
    override: str | None = None  # one-shot: if set, next today() returns this id


def load_state() -> State:
    if not STATE_PATH.exists():
        sys.exit(
            f"No state at {STATE_PATH}. Initialize it (see spec §6.3) before running."
        )
    p = json.loads(STATE_PATH.read_text())
    return State(
        start_date=p["start_date"],
        day=p["day"],
        queues=p.get("queues", {"hld": [], "lld": []}),
        last_seen=p.get("last_seen", {}),
        confidence=p.get("confidence", {}),
        revisit_due=p.get("revisit_due", {}),
        history=p.get("history", []),
        completed=p.get("completed", {"hld_count": 0, "lld_count": 0, "revisits_count": 0}),
        override=p.get("override"),
    )


def save_state(s: State) -> None:
    SCHEDULE_DIR.mkdir(exist_ok=True)
    payload = {
        "schema_version": 1,
        "start_date": s.start_date,
        "day": s.day,
        "queues": s.queues,
        "last_seen": s.last_seen,
        "confidence": s.confidence,
        "revisit_due": s.revisit_due,
        "history": s.history,
        "completed": s.completed,
        "override": s.override,
    }
    STATE_PATH.write_text(json.dumps(payload, indent=2))


# ---------- core: picking today's problem ----------


def today_branch(today: date) -> str:
    return WEEKDAY_BRANCH[today.weekday()]


def due_for_revisit(s: State, today: date) -> list[str]:
    """All problems whose revisit_due ≤ today, sorted by lowest confidence first."""
    today_iso = today.isoformat()
    due = [pid for pid, dt in s.revisit_due.items() if dt <= today_iso]
    return sorted(due, key=lambda pid: (s.confidence.get(pid, 0), s.revisit_due[pid]))


def pick_today(s: State, today: date, company: str | None = None) -> dict:
    """Returns {problem_id, mode, problem, branch}. Never mutates state."""
    if s.override:
        pid = s.override
        problem = problem_by_id(pid)
        return {
            "problem_id": pid,
            "mode": "override",
            "problem": problem,
            "branch": "hld" if pid.startswith("H") else "lld",
        }

    branch = today_branch(today)

    if branch == "revisit":
        due = due_for_revisit(s, today)
        if not due:
            return {"problem_id": None, "mode": "revisit", "problem": None, "branch": "revisit"}
        pid = due[0]
        return {
            "problem_id": pid,
            "mode": "revisit",
            "problem": problem_by_id(pid),
            "branch": branch,
        }

    # fresh pick from active queue
    queue = s.queues.get(branch, [])
    if company:
        # filter queue to problems whose asked_by contains the company tag
        catalog = load_catalog(branch)
        by_id = {row["id"]: row for row in catalog["problems"]}
        queue = [pid for pid in queue if company in by_id.get(pid, {}).get("asked_by", [])]

    # skip problems already in revisit_due (they're not "fresh" anymore)
    queue = [pid for pid in queue if pid not in s.revisit_due]

    if not queue:
        return {"problem_id": None, "mode": "fresh", "problem": None, "branch": branch}

    pid = queue[0]
    return {
        "problem_id": pid,
        "mode": "fresh",
        "problem": problem_by_id(pid),
        "branch": branch,
    }


# ---------- tick: record completion ----------


def compute_revisit_due(today: date, confidence: int) -> str:
    interval = REVISIT_INTERVALS.get(confidence, 7)
    return (today + timedelta(days=interval)).isoformat()


def tick(s: State, pid: str, confidence: int, today: date, note_lines: int | None = None) -> None:
    """Record problem completion. Mutates state in place."""
    if confidence not in REVISIT_INTERVALS:
        sys.exit(f"Confidence must be 0..5, got {confidence}")

    branch = "hld" if pid.startswith("H") else "lld"
    today_iso = today.isoformat()

    # remove from fresh queue if present (first completion)
    fresh = pid in s.queues.get(branch, []) and pid not in s.last_seen
    if pid in s.queues.get(branch, []):
        s.queues[branch] = [x for x in s.queues[branch] if x != pid]

    s.last_seen[pid] = today_iso
    s.confidence[pid] = confidence
    s.revisit_due[pid] = compute_revisit_due(today, confidence)

    if fresh:
        if branch == "hld":
            s.completed["hld_count"] += 1
        else:
            s.completed["lld_count"] += 1
    else:
        s.completed["revisits_count"] = s.completed.get("revisits_count", 0) + 1

    s.history.append({
        "day": s.day,
        "date": today_iso,
        "problem_id": pid,
        "branch": branch,
        "mode": "fresh" if fresh else "revisit",
        "confidence": confidence,
        "note_lines": note_lines,
    })

    s.day += 1
    s.override = None  # consume the override

    append_log(today, pid, branch, "fresh" if fresh else "revisit", confidence, note_lines)


def append_log(today: date, pid: str, branch: str, mode: str, confidence: int, note_lines: int | None) -> None:
    SCHEDULE_DIR.mkdir(exist_ok=True)
    if not LOG_PATH.exists():
        LOG_PATH.write_text("# Schedule Log\n\n")
    lines_part = f" · {note_lines} lines" if note_lines else ""
    line = f"- **{today.isoformat()}** — design {branch.upper()} {pid} · {mode} · confidence {confidence}{lines_part}\n"
    with LOG_PATH.open("a") as f:
        f.write(line)


# ---------- status + reporting ----------


def print_today(today: date, company: str | None = None) -> None:
    s = load_state()
    pick = pick_today(s, today, company)
    print(json.dumps(
        {
            "date": today.isoformat(),
            "weekday": today.strftime("%A"),
            "day_index": s.day,
            "branch": pick["branch"],
            "mode": pick["mode"],
            "problem_id": pick["problem_id"],
            "problem": pick["problem"],
            "override_in_effect": bool(s.override),
        },
        indent=2,
    ))


def print_status(today: date) -> None:
    s = load_state()
    due = due_for_revisit(s, today)
    print(f"Day {s.day} · start_date {s.start_date}")
    print(f"  HLD fresh queue: {len(s.queues.get('hld', []))} problems remaining")
    print(f"    next 5: {', '.join(s.queues.get('hld', [])[:5])}")
    print(f"  LLD fresh queue: {len(s.queues.get('lld', []))} problems remaining")
    print(f"    next 5: {', '.join(s.queues.get('lld', [])[:5])}")
    print(f"  Revisit due today: {len(due)}")
    if due:
        for pid in due[:10]:
            print(f"    {pid} (conf {s.confidence.get(pid, 0)}, last_seen {s.last_seen.get(pid)})")
    print(f"  Completed: HLD={s.completed.get('hld_count', 0)}, "
          f"LLD={s.completed.get('lld_count', 0)}, "
          f"revisits={s.completed.get('revisits_count', 0)}")
    if s.override:
        print(f"  OVERRIDE pending: {s.override}")


def print_revisit(today: date) -> None:
    s = load_state()
    due = due_for_revisit(s, today)
    if not due:
        print("Nothing due today.")
        return
    print(f"{len(due)} problem(s) due for revisit on {today.isoformat()}:")
    for pid in due:
        print(f"  {pid} · conf {s.confidence.get(pid, 0)} · last_seen {s.last_seen.get(pid)} · due {s.revisit_due.get(pid)}")


def set_override(pid: str) -> None:
    s = load_state()
    if problem_by_id(pid) is None:
        sys.exit(f"Unknown problem id: {pid}")
    s.override = pid
    save_state(s)
    print(f"Override set: next 'today' will return {pid}")


# ---------- CLI ----------


def parse_today_arg(value: str | None) -> date:
    if value is None:
        return date.today()
    return datetime.strptime(value, "%Y-%m-%d").date()


def main() -> None:
    p = argparse.ArgumentParser(prog="design_scheduler.py")
    sub = p.add_subparsers(dest="cmd", required=True)

    pt = sub.add_parser("today", help="print today's pick (no state change)")
    pt.add_argument("--date", default=None, help="override today's date (YYYY-MM-DD)")
    pt.add_argument("--company", default=None, help="filter fresh pick to problems asked by this company")

    pk = sub.add_parser("tick", help="record completion")
    pk.add_argument("--problem", required=True, help="problem id, e.g. H17")
    pk.add_argument("--confidence", required=True, type=int, choices=range(6))
    pk.add_argument("--date", default=None, help="override today's date (YYYY-MM-DD)")
    pk.add_argument("--lines", type=int, default=None, help="note length in lines (informational)")

    ps = sub.add_parser("status", help="print queue depths + completion counts")
    ps.add_argument("--date", default=None, help="override today's date (YYYY-MM-DD)")

    pr = sub.add_parser("revisit", help="list problems due for revisit today")
    pr.add_argument("--date", default=None, help="override today's date (YYYY-MM-DD)")

    po = sub.add_parser("override", help="force a specific problem on next 'today'")
    po.add_argument("--problem", required=True, help="problem id")

    args = p.parse_args()
    today = parse_today_arg(getattr(args, "date", None))

    if args.cmd == "today":
        print_today(today, company=args.company)
    elif args.cmd == "tick":
        s = load_state()
        tick(s, args.problem, args.confidence, today, args.lines)
        save_state(s)
        print(f"Ticked: {args.problem} confidence={args.confidence} day={s.day}")
    elif args.cmd == "status":
        print_status(today)
    elif args.cmd == "revisit":
        print_revisit(today)
    elif args.cmd == "override":
        set_override(args.problem)


if __name__ == "__main__":
    main()
