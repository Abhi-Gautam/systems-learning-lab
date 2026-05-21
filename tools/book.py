#!/usr/bin/env python3
"""
book.py — verbatim PDF extraction + glossary tool for the 2026 Reading List.

Subcommands:
  glossary <ALIAS> [--all]        Build TOC glossary -> glossaries/{alias}.json + .md
  read <ALIAS> --pages A-B        Print verbatim text for PDF page range [A, B]
  read <ALIAS> --section "<q>"    Print verbatim text for the section whose title contains <q>
  read <ALIAS> --chapter N        Print verbatim text for top-level entry #N
  locate <ALIAS> --page P         Print the chapter/section containing PDF page P

Page numbers are PDF page numbers (1-indexed for the CLI; pymupdf is 0-indexed internally).
The TOC outline reflects the PDF's embedded bookmarks; some books have richer outlines than others.
PDFs live in books/{ALIAS}.pdf; the legacy root-level {ALIAS}.pdf path is also checked.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import fitz  # pymupdf

ROOT = Path(__file__).resolve().parent.parent
BOOKS_DIR = ROOT / "books"
GLOSSARY_DIR = ROOT / "glossaries"

ALIASES = [
    "DDIA",
    "OSTEP",
    "N2T",
    "COD",
    "DBI",
    "CC",
    "TPP",
    "REF",
    "SAHP",
    "WPF",
    "WELC",
    "LDDD",
    "DSG",
    "LGO",
    "CLRS",
]


def pdf_path(alias: str) -> Path:
    name = f"{alias.upper()}.pdf"
    candidates = [BOOKS_DIR / name, ROOT / name]
    for p in candidates:
        if p.exists():
            return p
    checked = ", ".join(str(p) for p in candidates)
    raise FileNotFoundError(f"PDF not found. Checked: {checked}")


def load_glossary(alias: str) -> dict:
    """Return saved glossary JSON if it exists, else build from embedded TOC."""
    cached = GLOSSARY_DIR / f"{alias.lower()}.json"
    if cached.exists():
        return json.loads(cached.read_text())
    return build_glossary(alias)


def build_glossary(alias: str) -> dict:
    """Extract the embedded TOC outline as a flat list with depth + 1-indexed PDF page."""
    doc = fitz.open(pdf_path(alias))
    toc = doc.get_toc(simple=True)  # [[level, title, page1based], ...]
    entries = [
        {"level": lvl, "title": title.strip(), "page": page} for lvl, title, page in toc
    ]
    glossary = {
        "alias": alias.upper(),
        "pdf_pages": doc.page_count,
        "entries": entries,
    }
    doc.close()
    return glossary


def write_glossary(alias: str, glossary: dict) -> tuple[Path, Path]:
    GLOSSARY_DIR.mkdir(exist_ok=True)
    alias_l = alias.lower()
    json_path = GLOSSARY_DIR / f"{alias_l}.json"
    md_path = GLOSSARY_DIR / f"{alias_l}.md"

    json_path.write_text(json.dumps(glossary, indent=2, ensure_ascii=False))

    lines = [
        f"# {glossary['alias']} Glossary",
        f"_PDF pages: {glossary['pdf_pages']} · TOC entries: {len(glossary['entries'])}_",
        "",
    ]
    if not glossary["entries"]:
        lines.append("_No embedded TOC outline. Use `--pages` directly._")
    for e in glossary["entries"]:
        indent = "  " * (e["level"] - 1)
        lines.append(f"{indent}- p.{e['page']} — {e['title']}")
    md_path.write_text("\n".join(lines) + "\n")
    return json_path, md_path


def cmd_glossary(args) -> int:
    targets = ALIASES if args.all else [args.alias]
    for a in targets:
        try:
            g = build_glossary(a)
            jp, mp = write_glossary(a, g)
            print(
                f"[{a}] {len(g['entries'])} entries · {g['pdf_pages']} pages → {mp.name}"
            )
        except FileNotFoundError as e:
            print(f"[{a}] SKIP: {e}", file=sys.stderr)
    return 0


def read_pages(alias: str, start: int, end: int) -> str:
    doc = fitz.open(pdf_path(alias))
    start = max(1, start)
    end = min(doc.page_count, end)
    chunks = []
    for i in range(start - 1, end):
        page = doc[i]
        chunks.append(f"\n===== {alias.upper()} · PDF p.{i + 1} =====\n")
        chunks.append(page.get_text("text"))
    doc.close()
    return "".join(chunks)


def section_range(glossary: dict, idx: int) -> tuple[int, int]:
    """Page range of entries[idx] = [its page, next-same-or-shallower-level page - 1]."""
    entries = glossary["entries"]
    start = entries[idx]["page"]
    target_level = entries[idx]["level"]
    end = glossary["pdf_pages"]
    for j in range(idx + 1, len(entries)):
        if entries[j]["level"] <= target_level:
            end = entries[j]["page"] - 1
            break
    return start, max(start, end)


def cmd_read(args) -> int:
    alias = args.alias
    if args.pages:
        a, b = args.pages.split("-") if "-" in args.pages else (args.pages, args.pages)
        text = read_pages(alias, int(a), int(b))
        sys.stdout.write(text)
        return 0

    glossary = load_glossary(alias)
    entries = glossary["entries"]
    if not entries:
        print(f"[{alias}] no TOC; use --pages", file=sys.stderr)
        return 2

    if args.chapter is not None:
        top = [i for i, e in enumerate(entries) if e["level"] == 1]
        if args.chapter < 1 or args.chapter > len(top):
            print(f"chapter out of range (1..{len(top)})", file=sys.stderr)
            return 2
        idx = top[args.chapter - 1]
    else:  # --section
        q = args.section.lower()
        matches = [i for i, e in enumerate(entries) if q in e["title"].lower()]
        if not matches:
            print(f"no section matches '{args.section}'", file=sys.stderr)
            return 2
        if len(matches) > 1:
            print(f"ambiguous '{args.section}' — matches:", file=sys.stderr)
            for i in matches:
                print(
                    f"  p.{entries[i]['page']} — {entries[i]['title']}", file=sys.stderr
                )
            return 2
        idx = matches[0]

    start, end = section_range(glossary, idx)
    print(f"# {entries[idx]['title']}  (PDF p.{start}–{end})\n", file=sys.stderr)
    sys.stdout.write(read_pages(alias, start, end))
    return 0


def cmd_locate(args) -> int:
    glossary = load_glossary(args.alias)
    page = args.page
    crumbs = []  # stack of (level, title, page)
    for e in glossary["entries"]:
        if e["page"] > page:
            break
        while crumbs and crumbs[-1][0] >= e["level"]:
            crumbs.pop()
        crumbs.append((e["level"], e["title"], e["page"]))
    if not crumbs:
        print(f"page {page}: (before first TOC entry — front matter)")
        return 0
    print(f"page {page} → " + " › ".join(f"{t} (p.{p})" for _, t, p in crumbs))
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    sub = ap.add_subparsers(dest="cmd", required=True)

    g = sub.add_parser("glossary", help="extract TOC -> glossaries/")
    g.add_argument("alias", nargs="?")
    g.add_argument("--all", action="store_true")
    g.set_defaults(func=cmd_glossary)

    r = sub.add_parser("read", help="verbatim text by pages/section/chapter")
    r.add_argument("alias")
    sel = r.add_mutually_exclusive_group(required=True)
    sel.add_argument("--pages", help="e.g. 10-20 or 42")
    sel.add_argument("--section", help="substring of TOC title")
    sel.add_argument("--chapter", type=int, help="1-indexed top-level entry")
    r.set_defaults(func=cmd_read)

    locate_cmd = sub.add_parser("locate", help="find chapter/section for a page")
    locate_cmd.add_argument("alias")
    locate_cmd.add_argument("--page", type=int, required=True)
    locate_cmd.set_defaults(func=cmd_locate)

    args = ap.parse_args()
    if args.cmd == "glossary" and not args.all and not args.alias:
        ap.error("glossary requires <alias> or --all")
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
