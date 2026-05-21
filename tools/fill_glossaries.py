#!/usr/bin/env python3
"""One-shot: build glossaries for books whose embedded TOCs are missing/sparse.

Data hand-curated from each book's printed Contents page (extracted via tools/book.py).
Page numbers below are PRINTED book pages; the per-book offset converts to PDF page.
"""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "glossaries"

# (level, title, printed_page)
N2T = [
    (1, "Introduction: Hello, World Below", 1),
    (1, "1. Boolean Logic", 7),
    (1, "2. Boolean Arithmetic", 29),
    (1, "3. Sequential Logic", 41),
    (1, "4. Machine Language", 57),
    (1, "5. Computer Architecture", 79),
    (1, "6. Assembler", 103),
    (1, "7. Virtual Machine I: Stack Arithmetic", 121),
    (1, "8. Virtual Machine II: Program Control", 153),
    (1, "9. High-Level Language", 173),
    (1, "10. Compiler I: Syntax Analysis", 199),
    (1, "11. Compiler II: Code Generation", 223),
    (1, "12. Operating System", 247),
    (1, "13. Postscript: More Fun to Go", 277),
    (1, "Appendix A: Hardware Description Language (HDL)", 281),
    (1, "Appendix B: Test Scripting Language", 297),
]

REF = [
    (1, "Chapter 1. Refactoring, a First Example", 13),
    (1, "Chapter 2. Principles in Refactoring", 46),
    (1, "Chapter 3. Bad Smells in Code", 63),
    (1, "Chapter 4. Building Tests", 73),
    (1, "Chapter 5. Toward a Catalog of Refactorings", 85),
    (1, "Chapter 6. Composing Methods", 89),
    (1, "Chapter 7. Moving Features Between Objects", 115),
    (1, "Chapter 8. Organizing Data", 138),
    (1, "Chapter 9. Simplifying Conditional Expressions", 192),
    (1, "Chapter 10. Making Method Calls Simpler", 220),
    (1, "Chapter 11. Dealing with Generalization", 259),
    (1, "Chapter 12. Big Refactorings", 293),
    (1, "Chapter 13. Refactoring, Reuse, and Reality", 311),
    (1, "Chapter 14. Refactoring Tools", 328),
    (1, "Chapter 15. Putting It All Together", 333),
]

TPP = [
    (1, "Part 1. A Pragmatic Philosophy", 1),
    (2, "1. The Cat Ate My Source Code", 2),
    (2, "2. Software Entropy", 4),
    (2, "3. Stone Soup and Boiled Frogs", 7),
    (2, "4. Good-Enough Software", 9),
    (2, "5. Your Knowledge Portfolio", 12),
    (2, "6. Communicate!", 18),
    (1, "Part 2. A Pragmatic Approach", 25),
    (2, "7. The Evils of Duplication", 26),
    (2, "8. Orthogonality", 34),
    (2, "9. Reversibility", 44),
    (2, "10. Tracer Bullets", 48),
    (2, "11. Prototypes and Post-it Notes", 53),
    (2, "12. Domain Languages", 57),
    (2, "13. Estimating", 64),
    (1, "Part 3. The Basic Tools", 71),
    (2, "14. The Power of Plain Text", 73),
    (2, "15. Shell Games", 77),
    (2, "16. Power Editing", 82),
    (2, "17. Source Code Control", 86),
    (2, "18. Debugging", 90),
    (2, "19. Text Manipulation", 99),
    (2, "20. Code Generators", 102),
    (1, "Part 4. Pragmatic Paranoia", 107),
    (2, "21. Design by Contract", 109),
    (2, "22. Dead Programs Tell No Lies", 120),
    (2, "23. Assertive Programming", 122),
    (2, "24. When to Use Exceptions", 125),
    (2, "25. How to Balance Resources", 129),
    (1, "Part 5. Bend, or Break", 137),
    (2, "26. Decoupling and the Law of Demeter", 138),
    (2, "27. Metaprogramming", 144),
    (2, "28. Temporal Coupling", 150),
    (2, "29. It's Just a View", 157),
    (2, "30. Blackboards", 165),
    (1, "Part 6. While You Are Coding", 171),
    (2, "31. Programming by Coincidence", 172),
    (2, "32. Algorithm Speed", 177),
    (2, "33. Refactoring", 184),
    (2, "34. Code That's Easy to Test", 189),
    (2, "35. Evil Wizards", 198),
    (1, "Part 7. Before the Project", 201),
    (2, "36. The Requirements Pit", 202),
    (2, "37. Solving Impossible Puzzles", 212),
    (2, "38. Not Until You're Ready", 215),
    (2, "39. The Specification Trap", 217),
    (2, "40. Circles and Arrows", 220),
    (1, "Part 8. Pragmatic Projects", 223),
    (2, "41. Pragmatic Teams", 224),
    (2, "42. Ubiquitous Automation", 230),
    (2, "43. Ruthless Testing", 237),
    (2, "44. It's All Writing", 248),
    (2, "45. Great Expectations", 255),
    (2, "46. Pride and Prejudice", 258),
    (1, "Appendix A. Resources", 261),
    (1, "Appendix B. Answers to Exercises", 279),
]

OSTEP = [
    (1, "1. A Dialogue on the Book", 1),
    (1, "2. Introduction to Operating Systems", 3),
    (1, "Part I. Virtualization", 21),
    (2, "3. A Dialogue on Virtualization", 23),
    (2, "4. The Abstraction: The Process", 25),
    (2, "5. Interlude: Process API", 39),
    (2, "6. Mechanism: Limited Direct Execution", 55),
    (2, "7. Scheduling: Introduction", 73),
    (2, "8. Scheduling: The Multi-Level Feedback Queue", 85),
    (2, "9. Scheduling: Proportional Share", 97),
    (2, "10. Multiprocessor Scheduling (Advanced)", 107),
    (2, "11. Summary Dialogue on CPU Virtualization", 121),
    (2, "12. A Dialogue on Memory Virtualization", 123),
    (2, "13. The Abstraction: Address Spaces", 125),
    (2, "14. Interlude: Memory API", 135),
    (2, "15. Mechanism: Address Translation", 147),
    (2, "16. Segmentation", 161),
    (2, "17. Free-Space Management", 173),
    (2, "18. Paging: Introduction", 187),
    (2, "19. Paging: Faster Translations (TLBs)", 201),
    (2, "20. Paging: Smaller Tables", 217),
    (2, "21. Beyond Physical Memory: Mechanisms", 229),
    (2, "22. Beyond Physical Memory: Policies", 239),
    (2, "23. Complete Virtual Memory Systems", 251),
    (2, "24. Summary Dialogue on Memory Virtualization", 273),
    (1, "Part II. Concurrency", 275),
    (2, "25. A Dialogue on Concurrency", 277),
    (2, "26. Concurrency: An Introduction", 279),
    (2, "27. Interlude: Thread API", 297),
    (2, "28. Locks", 309),
    (2, "29. Lock-based Concurrent Data Structures", 329),
    (2, "30. Condition Variables", 341),
    (2, "31. Semaphores", 357),
    (2, "32. Common Concurrency Problems", 377),
    (2, "33. Event-based Concurrency (Advanced)", 391),
    (2, "34. Summary Dialogue on Concurrency", 401),
    (1, "Part III. Persistence", 403),
    (2, "35. A Dialogue on Persistence", 405),
    (2, "36. I/O Devices", 407),
    (2, "37. Hard Disk Drives", 421),
    (2, "38. Redundant Arrays of Inexpensive Disks (RAIDs)", 435),
    (2, "39. Interlude: File and Directories", 451),
    (2, "40. File System Implementation", 467),
    (2, "41. Locality and The Fast File System", 481),
    (2, "42. Crash Consistency: FSCK and Journaling", 491),
    (2, "43. Log-structured File Systems", 511),
    (2, "44. Flash-based SSDs", 525),
    (2, "45. Data Integrity and Protection", 545),
    (2, "46. Summary Dialogue on Persistence", 557),
    (2, "47. A Dialogue on Distribution", 559),
    (2, "48. Distributed Systems", 561),
    (2, "49. Sun's Network File System (NFS)", 579),
    (2, "50. The Andrew File System (AFS)", 595),
    (2, "51. Summary Dialogue on Distribution", 607),
]

BOOKS = {
    "n2t": {"offset": 14, "pdf_pages": 331, "entries": N2T},
    "ref": {"offset": 0, "pdf_pages": 337, "entries": REF},
    "tpp": {"offset": 25, "pdf_pages": 352, "entries": TPP},
    "ostep": {"offset": 25, "pdf_pages": 687, "entries": OSTEP},
}


def write_book(alias_l: str, data: dict):
    OUT.mkdir(exist_ok=True)
    entries = [
        {"level": lvl, "title": title, "page": bp + data["offset"], "book_page": bp}
        for lvl, title, bp in data["entries"]
    ]
    glossary = {
        "alias": alias_l.upper(),
        "pdf_pages": data["pdf_pages"],
        "book_to_pdf_offset": data["offset"],
        "source": "hand-curated from printed Contents page",
        "entries": entries,
    }
    (OUT / f"{alias_l}.json").write_text(
        json.dumps(glossary, indent=2, ensure_ascii=False)
    )

    lines = [
        f"# {glossary['alias']} Glossary",
        f"_PDF pages: {glossary['pdf_pages']} · TOC entries: {len(entries)} · "
        f"book→PDF offset: +{data['offset']}_",
        "_Source: hand-curated from printed Contents page._",
        "",
    ]
    for e in entries:
        indent = "  " * (e["level"] - 1)
        lines.append(
            f"{indent}- p.{e['page']} (book p.{e['book_page']}) — {e['title']}"
        )
    (OUT / f"{alias_l}.md").write_text("\n".join(lines) + "\n")
    print(
        f"[{alias_l.upper()}] wrote {len(entries)} entries (offset +{data['offset']})"
    )


if __name__ == "__main__":
    for alias_l, data in BOOKS.items():
        write_book(alias_l, data)
