# DBI Notes

_Entries follow the template at `Notes/TEMPLATE.md`. Append-only. **Newest entry at top**, immediately after this header._

---

## [2026-05-27] Rightmost pointers & overflow pages · pp.116–120 · Ch.4 §Rightmost Pointers → §Node High Keys → §Overflow Pages

- The N-keys / (N+1)-pointers asymmetry and how SQLite and Postgres handle the extra pointer differently
- Blink-Tree high keys: upper-bound stored per node, simplifying rightmost-pointer handling
- Overflow pages: linked-page extensions for variable-size values that exceed `max_payload_size`

### History — "why does this exist?"

The **rightmost pointer problem** is inherent in B-Trees since **Bayer and McCreight's 1972 paper**: with N separator keys, you have N+1 child pointers, and the last one doesn't pair with any key. **SQLite** (D. Richard Hipp, 2000) stores it in the page header as a special field. **Lehman and Yao's Blink-Tree (1981)** solved this differently by adding a **high key** to each node — the maximum possible key in the subtree — which gives each pointer a paired key and simplifies concurrent access. **PostgreSQL's `nbtree`** implementation uses Blink-Trees and stores high keys in every internal node. Overflow pages trace to **IMS and VSAM (IBM, 1960s–70s)**, where variable-length records that didn't fit a fixed block size spilled into extension blocks — the same linked-page technique modern B-Tree engines use.

### Intuition — "this is like…"

The rightmost pointer is the **`default` case in a switch statement**. Separator keys define explicit boundaries (`< 10`, `< 20`, `< 30`), but something has to handle `≥ 30`. In a standard B-Tree, that "default" pointer is stored outside the key-pointer pairs — it's the one pointer without a matching key. Blink-Trees eliminate the special case by adding a sentinel key (the high key) that says "this subtree handles values up to 30" — now every pointer has a label, and there's no `default` branch.

Overflow pages are **Git LFS for B-Trees**: when a blob is too big to store inline, you replace it with a pointer to external storage and link the pages together.

### Mechanics

#### The N+1 pointer problem

A B-Tree node with N keys has N+1 child pointers. The layout:

```
Standard layout (SQLite style):
┌──────────────────────────────────────────────────────────────┐
│ Header: [ ... | rightmost_ptr ]                              │
├──────────────────────────────────────────────────────────────┤
│ Cell 0: [ key₀ | ptr₀ ]   ← subtree where all keys < key₀  │
│ Cell 1: [ key₁ | ptr₁ ]   ← subtree where key₀ ≤ keys < key₁│
│ Cell 2: [ key₂ | ptr₂ ]                                     │
│ ...                                                          │
│ rightmost_ptr              ← subtree where keys ≥ keyₙ₋₁    │
└──────────────────────────────────────────────────────────────┘
```

When the rightmost child splits, the parent must:
1. Append a new cell with the promoted key pointing to the *old* rightmost child (the split node)
2. Update the header's `rightmost_ptr` to point to the *new* node

```
Before split:                          After split:
┌─────────────┐                       ┌──────────────────┐
│ keys: 10 20 │                       │ keys: 10 20 [30] │ ← promoted
│ ptrs: A B   │                       │ ptrs: A  B  [C]  │ ← old rightmost
│ right: C    │                       │ right: D          │ ← new node
└─────────────┘                       └──────────────────┘
```

This is a special case that every B-Tree implementation must handle — the rightmost pointer lives in a different location (header) than all other pointers (cells), so split logic needs an `if` branch.

#### Blink-Tree high keys (PostgreSQL's approach)

Blink-Trees eliminate the asymmetry by adding a **high key** — the upper bound of all keys in the subtree:

```
Blink-Tree layout:
┌──────────────────────────────────────────────────────────────┐
│ Cell 0: [ key₀ | ptr₀ ]   ← keys < key₀                    │
│ Cell 1: [ key₁ | ptr₁ ]   ← key₀ ≤ keys < key₁             │
│ Cell 2: [ key₂ | ptr₂ ]   ← key₁ ≤ keys < key₂             │
│ Cell 3: [ HIGH | ptr₃ ]   ← key₂ ≤ keys ≤ HIGH             │
│                              (HIGH = upper bound of subtree) │
└──────────────────────────────────────────────────────────────┘
```

Now **every pointer is paired with a key**. No header field, no special case in split logic. The cost: one extra key per node, reducing effective fanout by 1. For nodes with hundreds of keys, this is negligible.

**Side-by-side comparison:**

| | Standard (SQLite) | Blink-Tree (Postgres) |
|---|---|---|
| Keys per node | N | N+1 (includes high key) |
| Pointers per node | N+1 (last in header) | N+1 (all in cells) |
| Split edge case | Yes — must update header ptr | No — uniform cell handling |
| Search bound | Implicit +∞ for rightmost | Explicit high key |
| Concurrency | Complex (rightmost is shared state) | Simpler (high key aids latch coupling) |
| Space cost | Slightly less (no high key) | One extra key per node |

The concurrency advantage is the real motivation for Blink-Trees: during a node split, a concurrent reader that arrives at the old node can check the high key and realize the target key has moved to the new sibling — without holding a parent latch. This is why Postgres chose Blink-Trees: `nbtree` must handle concurrent reads and writes without global locks.

#### Overflow pages

When a value is too large to fit in a B-Tree cell (exceeds `max_payload_size = page_size / fanout`), it **spills** to a linked chain of overflow pages:

```
Primary page:
┌──────────────────────────────────────────────────┐
│ Cell: [ key | inline_prefix | overflow_ptr ──────┼──┐
│       (first max_payload_size bytes)             │  │
└──────────────────────────────────────────────────┘  │
                                                      ▼
                                              ┌───────────────┐
                                              │ Overflow pg 1 │
                                              │ (4K of data)  │
                                              │ next_ptr ─────┼──► ...
                                              └───────────────┘
```

**Key invariant:** the primary page always has room for at least `max_payload_size` bytes per cell, so the B-Tree algorithm's "is this node full?" check works purely on cell count, not on actual data size. The overflow mechanism is transparent to the B-Tree split/merge logic.

**How SQLite does it:** cells store the first `min(payload_size, max_payload_size)` bytes inline; the rest goes to overflow pages linked in a chain. `max_payload_size` is approximately `(usable_page_size - 12) × 64 / 255 - 23` for table B-Trees — roughly 25% of the page. This means a 4K page can store ~1K inline before spilling.

**How InnoDB (MySQL) does it:** for `DYNAMIC` row format, only a 20-byte pointer is stored inline for large columns; the entire value lives on overflow pages. For `COMPACT` format, 768 bytes are stored inline. The choice trades read performance (inline = fewer I/Os) against page utilization (inline = fewer cells per page = deeper tree).

**Trade-off:**

```
┌───────────────────────────────────────────────────────────┐
│  More inline storage (SQLite, InnoDB COMPACT)              │
│  + Point reads need fewer I/Os for small-to-medium values  │
│  – Large values waste primary-page space → lower fanout    │
│                                                            │
│  Less inline storage (InnoDB DYNAMIC)                      │
│  + Higher fanout → shallower tree → faster range scans     │
│  – Every large-value read needs at least one extra I/O     │
└───────────────────────────────────────────────────────────┘
```

### Where this shows up in real systems

- **PostgreSQL's `nbtree`** uses Blink-Tree high keys in every internal page. You can see them with `pageinspect`: `SELECT * FROM bt_page_items('idx_name', 1)` — the last item on each page is the high key, with `itemoffset = 0` in the output. Concurrent readers use the high key to detect in-progress splits without blocking.
- **SQLite's `btreeInitPage()`** reads the rightmost pointer from bytes 8–11 of every internal page header. The split routine `balance_nonroot()` has explicit logic to handle the rightmost pointer reassignment — the exact edge case the textbook describes.
- **InnoDB's BLOB storage** uses a 20-byte external pointer (space ID + page number + offset + length) for off-page columns. The `btr_store_big_rec_extern_fields()` function in MySQL source allocates overflow pages and chains them — the linked-list structure from the textbook, at production scale.

### Diagnostic questions

1. **Q:** Why does SQLite store the rightmost pointer in the header instead of as a regular cell?
   *Wrong-answer trap:* "To save space." It's because a regular cell pairs a key with a pointer, but the rightmost pointer has *no key* — it handles everything above the last separator. There's nothing to store in the key field. SQLite chose the header; Blink-Trees chose to invent a key (the high key) to fill the gap.

2. **Q:** A Blink-Tree node has 100 keys. How many are separator keys vs high keys?
   *Wrong-answer trap:* "100 separators." 99 are separators; 1 is the high key (the last entry). The high key doesn't separate children — it bounds the subtree.

3. **Q:** A B-Tree on a 4K page stores 2K values. What happens to fanout?
   *Wrong-answer trap:* "Fanout drops to 2." With overflow pages, fanout stays high — each cell stores only `max_payload_size` bytes inline (maybe ~1K), and the rest spills to overflow. Without overflow, yes — only 2 values per page, making the tree pathologically deep.

4. **Q:** Why do overflow pages form a linked list rather than being indexed?
   *Wrong-answer trap:* "Simplicity." It's because overflow data is always read sequentially (you need the full value, not a random byte). A linked list minimizes metadata overhead — one pointer per page vs an array of pointers. For random access within a large value, you'd need a different structure (like an extent tree), but B-Tree values are read whole.

---

## [2026-05-26] Page header & sibling links · pp.111–115 · Ch.4 *Implementing B-Trees* (intro → Page Header → Magic Numbers → Sibling Links)

- What lives in a page header and why each field is there
- Magic numbers as cheap on-disk sanity checks (the `PAGE` = `50 41 47 45` trick)
- Sibling links: range-scan win vs split/merge cost

### History — "why does this exist?"
On-disk page formats are old enough that **the first paged storage manager — IBM IMS (1966)** — already used per-page headers to encode page type and free-space markers. The **B-Tree** specifically arrives with **Bayer & McCreight at Boeing in 1972**; their original paper sketched a per-page record that's recognisable in today's PostgreSQL `PageHeaderData`. **Sibling links** are the B+Tree's defining feature, introduced when Bayer added them in the late 1970s to make range scans O(n/B) instead of O(n log B). **Lehman & Yao's "Blink-tree" (1981)** combined sibling links with a right-link discipline that made concurrent B-Trees lock-friendly — the lineage every modern engine (Postgres, MySQL InnoDB, MongoDB WiredTiger, SQLite) traces back to. Magic numbers as a sanity-check convention are even older — `0xCAFEBABE` at the head of every Java `.class` file (1995) and `0x7F ELF` at the head of every Linux binary (1993) are the most-touched examples.

### Intuition — "this is like…"
A page header is **the HTTP response header of a B-Tree page**: a small fixed-shape prefix that tells the reader what to expect in the body. Just as an HTTP `Content-Length` tells you where the body ends and `Content-Type` tells you how to parse it, the page header's cell count tells you how many slots to iterate and the flags tell you whether the body holds keys+pointers (internal node) or keys+values (leaf). The magic number is the equivalent of HTTP's status line — if you don't see `HTTP/1.1 200` at byte 0, you know you're reading garbage before you parse a single header field.

### Mechanics

**What a B-Tree page header typically carries:**

```
 ┌─────────────────────────────────────────────────────────┐
 │  Page header (fixed-size prefix, e.g. 24 bytes)         │
 ├─────────────────────────────────────────────────────────┤
 │  magic number       (4B)  e.g. 0x50414745  ("PAGE")     │
 │  page kind / flags  (1B)  leaf? internal? overflow?     │
 │  format version     (1B)  schema-evolution marker       │
 │  cell count         (2B)  how many slot entries follow  │
 │  lower offset       (2B)  end of slot array (grows ↓)   │
 │  upper offset       (2B)  start of cell data (grows ↑)  │
 │  rightmost pointer  (4B)  child for keys > last slot    │
 │  prev sibling       (4B)  ← optional (Bayer+) for leaves│
 │  next sibling       (4B)  ← optional (Bayer+) for leaves│
 │  ...                                                    │
 └─────────────────────────────────────────────────────────┘
```

**Three real-world headers, compared:**

| Engine | Header carries | Page size default |
|---|---|---|
| PostgreSQL | `pd_lsn`, `pd_checksum`, `pd_flags`, `pd_lower`, `pd_upper`, `pd_special`, `pd_pagesize_version` | 8 KB |
| MySQL InnoDB | `FIL_PAGE_OFFSET`, `FIL_PAGE_PREV`, `FIL_PAGE_NEXT`, `FIL_PAGE_LSN`, `FIL_PAGE_TYPE`, plus 38-byte FIL trailer | 16 KB |
| SQLite | cell count, rightmost pointer, first-freeblock offset, fragmented-free byte count | 4 KB (configurable) |

Note InnoDB's `FIL_PAGE_PREV`/`FIL_PAGE_NEXT` — those are the sibling links the book is about to introduce, sitting at fixed offsets in every page. PostgreSQL stores them not in the header but in `BTPageOpaqueData` (the page's "special" area, accessed via `pd_special`).

**Magic numbers — the cheapest sanity check on disk.** The trick:

```
On write:   memcpy(page_buffer, "PAGE", 4);   // 0x50 41 47 45
            ...
            write(fd, page_buffer, PAGE_SIZE);

On read:    read(fd, page_buffer, PAGE_SIZE);
            if (memcmp(page_buffer, "PAGE", 4) != 0)
                panic("page %u corrupt or wrong type", pgno);
```

```
Why this works:
  - Random 4-byte sequence matching "PAGE" exactly:  1 / 2^32 ≈ 1 / 4 billion
  - If the page is truncated, misaligned, or has been
    overwritten by a different page type, the first 4
    bytes almost certainly aren't "PAGE"
  - Cost: 4 bytes per page + 4-byte compare on read.  Negligible.
  - Catches: torn writes, mis-paged file extension,
    confused fd offset, file truncation, partial writes
```

Real magic numbers in the wild — Giampaolo's *Practical File System Design* (referenced as `[GIAMPAOLO98]` in the source) is the canonical citation:

| Format | Magic | Where |
|---|---|---|
| ELF binary | `7F 45 4C 46` | bytes 0–3 of every Linux executable |
| Java class | `CA FE BA BE` | bytes 0–3 of every `.class` |
| PNG image | `89 50 4E 47 0D 0A 1A 0A` | bytes 0–7 |
| PostgreSQL WAL segment | `0xD093` (`XLOG_PAGE_MAGIC`) | bumped per major version! |
| SQLite db file | `"SQLite format 3\000"` (16 bytes) | bytes 0–15 |

The Postgres trick of *bumping the magic per major version* doubles as a refusal-to-replay check — wrong-version WAL is rejected before any record is parsed.

**Sibling links — the optimisation that defines a B+Tree:**

```
Without sibling links — range scan from key K₁ to K₂:

       Root
      /  |  \
    ...  N  ...           For each leaf in range:
       /   \                walk up to common ancestor,
     L₁    L₂               then back down to next leaf.
                            O(log B) per step → O((K₂−K₁)/B × log B) total.

With sibling links — same range scan:

  ┌─ L₁ ─┐ ←→ ┌─ L₂ ─┐ ←→ ┌─ L₃ ─┐   Find L₁ via root descent,
  │ ... │     │ ... │     │ ... │   then follow next pointers.
  └─────┘     └─────┘     └─────┘   O(log B) + O((K₂−K₁)/B) total.
```

**The cost of sibling links** — the maintenance burden Khononov flags:

```
Split (non-rightmost leaf L):
  L: [a, b, c, d, e]   →   L: [a, b, c]  +  L': [d, e]

  Updates required:
    1. L.next  = L'        (new right sibling)
    2. L'.prev = L         (back link)
    3. L'.next = (old L.next) = R
    4. R.prev  = L'        ← THIS is the extra cost; R is a different page
    5. parent gets new entry (would happen anyway)
```

Step 4 dirties an extra page that wasn't even involved in the split. In a concurrent B-Tree this is the moment when **the right-sibling page has to be locked** — and not just any latch, a write latch — purely because we're rebinding its back-pointer. Lehman & Yao's Blink-Tree solution: **use only forward links** + a "high key" per page that doubles as a split detector, so a concurrent reader can notice "this page split under me, follow next-pointer once to recover." The book defers that to a later section.

**Trade-off summary:**

```
┌──────────────────────────────────────────────────────┐
│  WITHOUT sibling links                               │
│    + simpler split/merge (no neighbour to touch)     │
│    – range scan is O(n × log B)                      │
│  WITH sibling links (B+Tree)                         │
│    + range scan is O(n)                              │
│    – split touches 3 pages; merge touches 3 pages    │
│    – concurrent maintenance needs extra latches      │
│      (Blink-Tree solves part of this; see Ch.6)      │
└──────────────────────────────────────────────────────┘
```

Every production OLTP B-Tree picks "with sibling links". Range queries are too common to give up the win.

### If you were the storage-engine author…

A user reports a crash with `ERROR: page corrupt at file offset 0x80000`. You have a magic-number-tagged page format. What's the first thing you check?

You **dump the first 16 bytes of that offset** and compare with the expected magic. Three possibilities:

- *Magic matches → page header is intact*; the corruption is later (cell offsets, body). Diff the checksum against recomputed value; check the slot array for overlapping ranges.
- *Magic is zeros* → the page is **partially extended** but never written (file system gave you sparse zeroes). Probably a `posix_fallocate` followed by a power loss; check WAL replay status.
- *Magic is a different 4-byte value* → the page was overwritten by a **different page type** (the same file holding heap pages and index pages with type-discriminating magics). Now you know exactly which kind of corruption you have — fd-mixed write, or an off-by-one in your page-table indexing.

The magic number turned a one-bit "page is broken" signal into a three-way diagnostic.

### Cross-language view
*(n/a — page-header design is a binary-encoding concern, not a language one. The same fixed-prefix-with-magic discipline shows up in any language's serializer for on-disk data — e.g., Rust's `bincode` deliberately has no magic (a known footgun); Cap'n Proto and FlatBuffers prepend identifiers.)*

### Where this shows up in real systems

- **Postgres `pg_filedump` utility** parses on-disk pages purely from headers and slot offsets — the existence of `pg_filedump` is direct testimony that the header carries everything you need to interpret the body without consulting any catalog. That's the *contract* the page header encodes.
- **InnoDB's `FIL_PAGE_PREV`/`FIL_PAGE_NEXT`** are the *exact* sibling links the book is introducing — and the reason MySQL's `SELECT ... ORDER BY id LIMIT 100, 1000000` does not collapse: it walks leaf pages via `FIL_PAGE_NEXT` rather than re-descending the tree.
- **WiredTiger** (MongoDB) uses page-type magics (`WT_PAGE_*`) and a per-page checksum, but **does not store sibling links in B+Tree leaves**. Instead, range scans use the in-memory cache's sibling pointers and re-descend on a miss — a different point on the trade-off curve.
- **ZFS block pointers** carry a per-block `dva` (Data Virtual Address) and a 256-bit checksum in the *parent* block, not in the block itself. The magic-number-in-header pattern is replaced by checksum-in-parent — which catches the same torn-write class of failures but pushes the metadata one level up.

### Diagnostic questions

1. **Q:** What does a magic number protect against that a checksum doesn't?
   *Wrong-answer trap:* "Nothing — checksums are stronger." The trap is conflating the two. A checksum confirms *integrity within the page*; a magic confirms *that you're reading the right page type at this offset*. If your fd is at the wrong offset, the page may have a *valid* checksum (for the *wrong* page) but a *wrong* magic — magic catches the misaligned read class of bug; checksum doesn't.

2. **Q:** Why does a non-rightmost B+Tree leaf split require touching three pages instead of two?
   *Wrong-answer trap:* "It only touches two — the splitting page and its new sibling." Wrong: the **right neighbour's back-pointer** must be updated to point to the new sibling. Three pages → three latches in a concurrent implementation.

3. **Q:** If sibling links are so useful, why doesn't every B-Tree implementation use them?
   *Wrong-answer trap:* "Storage cost." Storage is negligible. The real cost is **concurrent-write complexity** — the right neighbour needs to be latched during a split. Lehman & Yao's Blink-Tree mitigates this with forward-only links + high keys, but the cost shaped MongoDB WiredTiger's "no on-disk sibling links" decision.

4. **Q:** Postgres bumps `XLOG_PAGE_MAGIC` per major version. What problem does this prevent?
   *Wrong-answer trap:* "Generic corruption detection." More specific: it **refuses to replay WAL records from an older format** during pg_upgrade scenarios, preventing silent format-skew bugs where a v15 server would otherwise apply v14-encoded records and produce subtly wrong state.

5. **Q:** A `FIL_PAGE_TYPE` of `0` (InnoDB-allocated but never written). Why is this distinct from a torn write?
   *Wrong-answer trap:* "It's just corruption — handle the same way." Wrong: `0` indicates a page that was *extended* (e.g., by `posix_fallocate`) but not initialised; recovery should reinitialise it, not refuse to start. A torn write would have a partial magic and a checksum mismatch — different recovery path.

---

## [2026-05-25] File versioning & checksums · pp.106–110 · Ch.3 *File Formats* — Managing Variable-Size Data (end) → Versioning → Checksumming → Summary

- Binary formats have no schema-text to fall back on — versioning must be **explicit**
- Three strategies: filename prefix, sidecar file, header field (each with one industry exemplar)
- Error-detection hierarchy: parity → checksum → CRC → cryptographic hash — each catches more, costs more

### History — "why does this exist?"
**Hamming codes (Richard Hamming, Bell Labs, 1950)** were the first systematic error-detection-and-correction codes — born from Hamming's frustration that overnight batch jobs would crash on a single bit-flip and lose a weekend's work. **CRC (Cyclic Redundancy Check, W. Wesley Peterson, IBM, 1961)** became the dominant detection code in storage and networks because polynomial division can be implemented with shift-and-XOR — a single hardware loop on cheap silicon. **Ethernet (1980)** mandated CRC-32 on every frame, **TCP (1981)** chose a weaker 16-bit one's-complement checksum for speed, and the choice still defines those protocols' error profiles 45 years later. **The Unix `file(1)` command (1973, McIlroy)** introduced "magic numbers" — the first few bytes of a file identifying its format — which is the ancestor of header-field versioning. **Postgres `PG_VERSION` (since 6.0, ~1997)** picked the sidecar-file approach. **Cassandra (2008)** went the filename-prefix route. **ZFS (Sun, 2003)** was the first mainstream filesystem to make per-block cryptographic-strength checksums the default — "trust nothing the disk says" became a real engineering position.

### Intuition — "this is like…"
**File versioning is HTTP API versioning, applied to bytes.** The same three strategies appear in both worlds:

| Versioning strategy | Database / file world | HTTP API world |
|---|---|---|
| **Filename prefix** | Cassandra: `na-1-big-Data.db` (na = v4.0+) | URL path: `/v2/users` |
| **Sidecar file** | Postgres: `PG_VERSION` file alongside data | OpenAPI spec file: `openapi-v2.yaml` |
| **Header field / magic number** | SSTable header byte: `0x5A 0x5A 0x00 0x01` | HTTP header: `Accept: application/vnd.api+v2` |

The trade-offs port across too: filename versioning is **discoverable without opening anything**, sidecar files are **easy to update independently**, header fields are **inseparable from the data they describe**.

**Checksumming is ECC RAM at the file level.** It doesn't *fix* corruption — it *detects* it before bad bytes propagate to layers above. The whole engineering point is "fail loudly, not silently." A disk that returns wrong bits and reports success is **the most dangerous failure mode in storage**, and checksums are how you turn silent corruption into a crash.

### Mechanics

#### Three versioning strategies — one row each, with the byte-level reality

```
┌──────────────────────────────────────────────────────────────────┐
│ Strategy 1: Filename prefix (Cassandra)                          │
├──────────────────────────────────────────────────────────────────┤
│  na-1-big-Data.db                                                │
│  ↑↑                                                              │
│  └─ "na" prefix = SSTable version 4.0 wire format                │
│     Previous: "ma" = 3.0, "lb" = 3.x, etc.                       │
│                                                                  │
│  Reader's job: parse the filename, dispatch to the right         │
│  decoder before opening the file at all.                         │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ Strategy 2: Sidecar file (Postgres)                              │
├──────────────────────────────────────────────────────────────────┤
│  $PGDATA/                                                        │
│  ├── PG_VERSION   ← contains "15\n" or similar (ASCII)           │
│  ├── base/...                                                    │
│  └── pg_wal/...                                                  │
│                                                                  │
│  Reader's job: read PG_VERSION first, then start the rest of     │
│  the server in version-appropriate mode.                         │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ Strategy 3: Header field with magic number (SQLite, ELF, PNG)    │
├──────────────────────────────────────────────────────────────────┤
│  SQLite database header (first 100 bytes of every .db file)      │
│                                                                  │
│  Offset  Size  Description                                       │
│  ──────  ────  ───────────────────────────────────────           │
│  0       16    "SQLite format 3\0"   ← magic string              │
│  16      2     Page size in bytes    (big-endian u16)            │
│  18      1     File format write version                         │
│  19      1     File format read version                          │
│  20      1     Reserved bytes per page                            │
│  ...                                                             │
│                                                                  │
│  Reader's job: read first ~100 bytes, validate magic, branch     │
│  on read/write version. The format is self-describing.           │
└──────────────────────────────────────────────────────────────────┘
```

The header-with-magic-number approach is dominant for files that travel (download, email, S3) because **the version is inseparable from the bytes**. Filename and sidecar strategies break the moment someone renames the file or strips the directory metadata.

#### The error-detection hierarchy — what catches what

| Code | Bits | Catches | Misses | Cost (cycles/byte, modern x86) | Where used |
|---|---|---|---|---|---|
| **Parity (XOR)** | 1 bit | Any odd number of bit-flips | Two-bit flips (2 wrongs cancel) | ~0.1 | RS-232 (1960s), legacy memory |
| **One's-complement sum** | 16 bit | Single-bit flips | Bit-flips that sum to 0 (rearrangements undetectable) | ~0.3 | TCP, UDP, IP |
| **CRC-32 (Castagnoli, polynomial 0x1EDC6F41)** | 32 bit | All burst errors ≤ 32 bits, ~99.999999998% of random | Adversarial collisions (trivial to construct) | ~0.5 (hw: 0.1 w/ SSE4.2 `crc32` instr) | Ethernet, ZIP, gzip, ext4 metadata, Postgres |
| **Fletcher-4** | 64 bit | Most random corruptions | Adversarial | ~0.4 | ZFS default |
| **xxHash (XXH64)** | 64 bit | Random + most adversarial | Cryptographic attacks | ~0.2 (faster than memcpy!) | RocksDB, LZ4 |
| **SHA-256** | 256 bit | Everything except 2^128 work | Nothing realistic | ~7 (hw: 1 w/ SHA-NI) | Bitcoin, ZFS opt-in, content-addressing |
| **BLAKE3** | 256 bit | Same as SHA-256 | Same | ~1.5 (parallelizable) | IPFS, modern content-hashing |

Two non-obvious points the book gestures at:

1. **Checksums (parity, sum) catch *random* corruption; CRCs catch *burst* corruption (consecutive bit-flips)**. Storage failure modes are usually burst errors (a sector goes bad, a whole strip flips), so CRC fits storage better than checksums even though both are "small."
2. **None of parity/sum/CRC are safe against an adversary**. The book's warning is explicit: do not use CRCs to verify tamper-resistance. CRC + known plaintext = trivial collision. That's why ZFS made cryptographic hashes opt-in for hostile-environment deployments.

#### What XOR-parity actually misses — a worked failure

Page contents (4 bytes, hex): `0xDE 0xAD 0xBE 0xEF`. Parity = `0xDE ⊕ 0xAD ⊕ 0xBE ⊕ 0xEF = 0xCC`.

Single bit-flip in last byte: `0xDE 0xAD 0xBE 0xEE`. New parity: `0xDE ⊕ 0xAD ⊕ 0xBE ⊕ 0xEE = 0xCD`. **Mismatch caught.** ✅

Two bit-flips, one in byte 2, one in byte 4: `0xDE 0xAD 0xBF 0xEE`. New parity: `0xDE ⊕ 0xAD ⊕ 0xBF ⊕ 0xEE = 0xCC`. **Mismatch NOT caught.** ❌

This is why XOR-parity is only useful at the *bit* level (single-bit detection within a byte), never at the *byte* level. Real-world corruption is rarely a single-bit flip — disk sectors fail in bursts of consecutive bits, and adversaries are arbitrary.

#### CRC-32 in three lines of pseudo-code

```python
def crc32(data, poly=0xEDB88320):
    crc = 0xFFFFFFFF
    for byte in data:
        crc ^= byte
        for _ in range(8):
            crc = (crc >> 1) ^ poly if crc & 1 else crc >> 1
    return crc ^ 0xFFFFFFFF
```

That's the entire algorithm. The reason it's everywhere is hardware: x86 CPUs since SSE4.2 (Nehalem, 2008) have a `crc32` instruction (~0.1 cycle/byte). Modern CRC-32C (Castagnoli polynomial) at full memory bandwidth costs essentially nothing — which is why Postgres, ext4, and the Linux block layer all run CRC-32 by default on every metadata page.

#### Per-page vs per-file checksums — why the granularity matters

```
PER-FILE checksum                  PER-PAGE checksum (modern DBs)

  ┌──────────┐                     ┌──────────┐ ← page 1 + checksum
  │          │                     ├──────────┤
  │  10 GB   │                     │          │ ← page 2 + checksum
  │   data   │   one 32-bit        ├──────────┤
  │          │     CRC at          │          │ ← page 3 + checksum  ← CORRUPTED
  │          │   the end           ├──────────┤
  │          │                     │          │ ← page 4 + checksum
  └──────────┘                     └──────────┘

  If page 3 corrupts:              If page 3 corrupts:
  - whole file is invalid            - only page 3 is invalid
  - must re-read 10 GB to check     - O(1) to validate per page
  - O(N) verification cost          - bad page identifies itself
```

The database lesson: **checksums must be local to the unit of read**. Postgres pages are 8KB by default with a checksum stored in the page header — a single corruption flags exactly that 8KB block as bad, not the whole 100GB table. This is the same engineering decision as TCP's per-segment checksum vs a per-stream checksum: corruption localization is more valuable than density.

#### What corruption looks like at the byte level

Postgres page header (24 bytes):

```
Offset  Size  Field           Example value
──────  ────  ──────────────  ──────────────
0       8     PageLSN         0x0000000018A4F210
8       2     Checksum        0x4F8A           ← CRC-32C truncated to 16 bits
10      2     Flags           0x0000
12      2     LowerOffset     0x18  (24 = header end)
14      2     UpperOffset     0xFE0 (free space starts here)
16      2     SpecialOffset   0x2000
18      4     PageVersion     0x0004           ← format version
22      2     PruneXid        0x0000
```

When Postgres reads this page from disk, it:
1. Recomputes CRC-32C over the page (excluding the checksum field itself).
2. Compares to the stored `Checksum` field.
3. On mismatch, returns `ERROR: invalid page in block N of relation` — and **does not** return the page's data to the SQL layer.

That last step is the entire point. The database has chosen `ERROR + crash` over `silent wrong answer`. ZFS, the strictest, will even refuse to serve a checksum-mismatched block in a mirror configuration unless one of the mirrors has a matching checksum — silent corruption literally cannot leak through.

### If you were the storage engine designer…
Two coupled choices:

**Versioning:** you'll pick header-field-with-magic-number, almost always. Filename prefix breaks the moment someone uses `mv`; sidecar files break the moment someone uses `cp file.db /backup/` and forgets the sidecar. Magic-number-in-header travels with the bytes. Cassandra got away with filename because SSTable files are *internal* to the data directory — they're never moved as standalone artifacts. Postgres got away with `PG_VERSION` because it controls the whole data directory layout. If your file might leave its directory, the version must be in the file.

**Checksumming:** start with CRC-32C and benchmark. It's free in hardware on modern CPUs, catches 99.99999998% of random/burst corruption, and is the industry default. Only reach for cryptographic hashes (SHA-256, BLAKE3) when you have a real hostile-actor threat model — backup integrity in untrusted cloud storage, content-addressable systems where collisions enable cache poisoning, or systems where corruption could be deliberate (filesystems on shared media). The book's warning is right: **do not use CRC for security**.

### Where this shows up in real systems
- **Postgres** uses **CRC-32C** for page checksums (introduced in 9.3, on by default since 12), the WAL stream (every record), and `pg_control`. A corrupt page raises `invalid page in block N` and refuses to serve the data — exactly the "crash rather than corrupt" stance.
- **SQLite** added per-page checksums in 3.7.13 (2012) via a `PRAGMA cell_size_check = ON`. Magic-number versioning is in the first 16 bytes: `"SQLite format 3\0"` — recognizable in any hex dump.
- **ZFS** uses **Fletcher-4** by default (~0.4 cycle/byte), with SHA-256 opt-in for hostile environments. Per-block, stored in the parent block pointer — so reading any block also tells you the expected checksum without trusting the block itself. This is the basis of ZFS's "self-healing" claim.
- **Bitcoin** uses **double SHA-256** for block hashes (the second hash neutralizes a theoretical length-extension attack on the first). Every block's content is committed by its 256-bit hash — content-addressable storage at protocol level.
- **Git** uses **SHA-1** for content-addressing (transitioning to SHA-256 in newer git versions). The hash *is* the identifier — `git show 7276c2b` is content-addressed lookup.
- **TCP's 16-bit one's-complement checksum** is famously weak — it catches single-bit errors but misses many byte rearrangements. In practice it's "ok" because Ethernet's CRC-32 below it and application-layer checksums above it catch what TCP misses. The stack of weak checks composes to acceptable end-to-end integrity, which is the **end-to-end argument** (Saltzer/Reed/Clark, 1981) in action.

### Diagnostic questions
1. **A Cassandra SSTable file is renamed by an ops engineer from `na-1-big-Data.db` to `data.db`. What breaks?** *Wrong:* "nothing, file contents unchanged." *Right:* **Cassandra refuses to load it** — the version is *only* in the filename. Versioning strategy that puts metadata outside the bytes fails as soon as the bytes are moved alone. This is the canonical critique of filename-based versioning.
2. **You see two databases store version metadata in two different ways. Postgres uses `PG_VERSION` sidecar; SQLite uses a 100-byte header. Why the difference?** *Wrong:* "preference." *Right:* **deployment shape**. Postgres owns a multi-file *data directory* that's not portable across systems; the sidecar approach fits. SQLite databases are **single files** that get copied to phones, attached to emails, embedded in apps — the version *must* be in the bytes or the file can't survive movement.
3. **TCP's checksum is much weaker than Ethernet's. Why isn't the internet drowning in corrupted packets?** *Wrong:* "checksums are mostly cosmetic." *Right:* **end-to-end argument** — TCP's weak check composes with Ethernet's CRC-32 below and TLS/application-layer MACs above. Each layer catches the others' misses; no single layer needs to be strong. (Counterpoint: there *are* documented cases of TCP-checksum-passing corruption causing silent data loss; it's not zero, just rare enough to live with.)
4. **You're choosing a hash function for an internal-only deduplication system processing 10 GB/s. SHA-256 or xxHash?** *Wrong:* "SHA-256 — security." *Right:* **xxHash**. The threat model is *random* corruption, not *adversarial* — a deduplication key doesn't need cryptographic strength, only collision resistance against natural inputs. xxHash at 35 GB/s (3.5x your throughput) leaves CPU for the actual workload; SHA-256 at 1-2 GB/s would bottleneck. Crypto when you have an adversary, fast hash when you don't.
5. **A 4KB Postgres page has a single-bit flip in the data area (not the checksum field). What happens on next read?** *Wrong:* "Postgres returns the wrong row." *Right:* **the recomputed CRC-32C mismatches, Postgres raises `ERROR: invalid page in block N`, the query fails, nothing is returned to the application**. The corruption is *contained* — bad bytes never reach the SQL layer. This is the entire engineering point of per-page checksums: turn silent corruption into a loud error.

---

## [2026-05-24] Cell Layout + the Sorted-Pointer Trick + Availability Lists — How Slotted Pages Keep Logical Order Without Moving Bytes · pp.101–105 · Ch.3 *File Formats* — Cell Layout → Combining Cells into Slotted Pages → Managing Variable-Size Data

### TL;DR
Yesterday's entry covered the *why* and gross anatomy of slotted pages; today's chunk fills in three precise mechanics. **First, cell layout** — every cell groups fixed-size fields (sizes, flags, page-ID) at its front and variable-size bytes (key, value) at its back, so any cell can be sliced with O(1) offset arithmetic. **Second, the sorted-pointer trick** — cells are appended to the cell region in **insertion order** (no relocation, ever), while logical order (lexicographic key order) is maintained by **sorting the offset array**, not the cells themselves. Insert "Tom" then "Leslie" then "Ron" → cells laid out as `[Ron|Leslie|Tom]` in arrival order, but the offset array reads `[→Leslie, →Ron, →Tom]` so binary search returns the right tuple. **Third, the availability list** — deletes don't compact; they push the freed `(offset, size)` onto an in-page free-list (SQLite calls these *freeblocks*), and inserts consult it with either **first-fit** (fast, more fragmentation) or **best-fit** (slower, less fragmentation) before resorting to defragmentation. Together these three rules let a slotted page absorb arbitrary insert/delete/lookup traffic with **zero external pointer invalidation** and **constant-time append in steady state.**

### Intuition — "this is like…"
The sorted-pointer trick is a **librarian's index card system**. Books arrive at the library in whatever order publishers ship them; the librarian crams each new book onto the next empty shelf without sorting. Meanwhile, the card catalog (offset array) is kept alphabetical — when a patron asks for "Ronson, J.," the librarian binary-searches the cards, reads "shelf 7, slot 12" off the card, and walks there. **Adding a book is O(1) shelf placement + O(log n) card insertion**; sorting the books themselves would be O(n) shelf relocation. The library has decoupled *storage order* from *lookup order* by paying for one level of indirection — the same trade as virtual memory, the same trade as Java object references, the same trade you already saw at the slot-directory level yesterday. Now the trick is applied **inside** the slot array itself.

### Mechanics

#### 1. Cell layout — fixed-prefix, variable-suffix

Every cell type follows the same shape: a fixed-size header containing all the **size descriptors** the cell needs to parse itself, followed by the variable-length payload.

##### Key cell (used in **internal** B-tree nodes — separator key + child pointer)

```
   byte:     0                4                8           8+key_size
            ┌────────────────┬────────────────┬──────────────────┐
            │ int  key_size  │ int  page_id   │   bytes  key     │
            └────────────────┴────────────────┴──────────────────┘
            └──── fixed header (8 B) ──────┘ └── variable payload ┘
```

##### Key-value cell (used in **leaf** nodes — key + data record)

```
   byte:     0       1                5                9        9+key_size       9+key_size+value_size
            ┌───────┬────────────────┬────────────────┬──────────────────┬──────────────────┐
            │ flags │ int  key_size  │ int value_size │  bytes  key      │  bytes  data     │
            └───────┴────────────────┴────────────────┴──────────────────┴──────────────────┘
            └─────── fixed header (9 B) ─────────────┘ └─────── variable payload ──────────┘
```

**The grouping convention — fixed before variable — is not aesthetic; it's algorithmic.** To locate the `value` field, you need `key_size` to know how much to skip. If `key_size` lived after the key bytes, you couldn't *find* `key_size` without first knowing where the key ends — a chicken-and-egg problem. By placing all size descriptors at known fixed offsets (`key_size @ byte 1, value_size @ byte 5`), the parser:

1. Reads `key_size` and `value_size` with two O(1) fixed-offset loads.
2. Then computes `key_offset = 9` and `value_offset = 9 + key_size` directly.

This is the **same encoding discipline N2T showed for the C-instruction**: type-discriminating bits first (at fixed positions), payload fields second (whose positions depend on prior fields). Universal pattern in binary protocols.

##### Why a `flags` byte exists in the leaf cell only
Internal-node cells are uniform (always key + page-ID). Leaf cells need flags for things like "is this a tombstoned cell?", "does the value overflow to another page?", "is the value compressed?" Each bit of flags is one binary attribute (yesterday's bit-packed booleans entry from 2026-05-22). For a 9-byte header, one flags byte costs less than 12% — cheap insurance for variant-cell-shape extensibility.

##### Page-ID vs. offset — the chapter's quiet design clarification
Notice the key cell stores `page_id` (an int referring to a page in the file) but cell pointers (in the slot directory) store **offsets relative to the page start**. Two different addressing modes because they live at different scopes:

| Reference kind | Scope | Stored as | Why |
|---|---|---|---|
| Cell → another page | File-wide | `page_id` (int, then × `PAGE_SIZE` via the buffer manager) | Pages have fixed size; ID indexes into the page array. Buffer manager (Ch.4) translates IDs to RAM addresses. |
| Slot directory → cell | Page-local | byte offset within page (uint16 if PAGE_SIZE ≤ 64K) | Slots are intra-page; smaller integer suffices, denser directory. |

This **scoped addressing** is why a 16-bit offset (2 bytes) is plenty for slot pointers — pages are 4–16 KB, and `log₂(16 KB) = 14 bits`. Using a 32-bit pointer would double the directory tax for no benefit.

#### 2. The sorted-pointer trick — cells in insertion order, pointers in key order

The chapter's worked example, expanded:

```
   Step 1: insert "Tom"
   ─────────────────────
   slots:   [→Tom]
   cells:   ........................|Tom |
                                    ↑

   Step 2: insert "Leslie"  (alphabetically Leslie < Tom)
   ──────────────────────────────────────────────────────
   slots:   [→Leslie, →Tom]          ← sorted by key
   cells:   .................|Leslie|Tom |     ← insertion order: Tom first, Leslie appended later
            
   Step 3: insert "Ron"  (alphabetically Leslie < Ron < Tom)
   ────────────────────────────────────────────────────────
   slots:   [→Leslie, →Ron, →Tom]    ← Ron's pointer inserted between Leslie's and Tom's
   cells:   .........|Ron|Leslie|Tom |     ← Ron's bytes appended at the front of the free space
```

The data bytes for Tom, Leslie, and Ron are **never moved**. Only the offset-array entries are reordered. When the slot directory needs to insert `→Ron` between `→Leslie` and `→Tom`, the entries after the insertion point shift right by one slot-pointer width (typically 2 bytes) — an O(n) memmove **on the directory only**, never on the cells.

##### Why this is a critical optimization
A naïve "keep cells sorted in storage order" page would have to shift all cells past the insertion point on every insert — O(n) **bytes**, often kilobytes. The sorted-pointer trick reduces it to O(n) **slot pointers**, typically 2 bytes each. For a 4 KB page with ~100 records of 40 bytes each, the cost asymmetry is:

| Approach | Bytes moved per insert (worst case, middle insert) |
|---|---|
| Sort cells in place | ~2 KB (half the page) |
| Sort pointers only | ~100 bytes (half the directory) |

A **20× reduction** in copy cost per insert, which translates directly to insert throughput. This is also why **range scans are fast**: walk the sorted slot directory linearly, dereferencing each pointer. Random key-order access to cell payloads, sequential cache-friendly access to the directory.

##### Binary search works on the directory, not the cells
Because the slot directory is sorted by key, you can binary-search **without dereferencing cells until the final step**:
1. Probe slot `mid = (lo + hi) / 2`.
2. Dereference its offset to read the cell's `key` bytes.
3. Compare; update `lo`/`hi`; repeat until found or window empty.

`O(log n)` cell-dereferences per lookup. Each dereference is one cache-line load (assuming the cell fits in 64 bytes). On a 4 KB page with 100 cells, **~7 probes** to find any key. This is why B-tree leaf scans are so fast: the per-page lookup is essentially-instant compared to the I/O that brought the page into cache.

#### 3. Managing variable-size deletes — the availability list

A delete doesn't shift cells. Instead:

```
   Before delete of "Ron":
   ───────────────────────
   slots:   [→Leslie, →Ron, →Tom]
   cells:   .........|Ron|Leslie|Tom |
   header:  free_space_ptr = 9, freeblocks_head = null

   After delete of "Ron":
   ──────────────────────
   slots:   [→Leslie, →Tom]                       ← Ron's slot removed (or tombstoned)
   cells:   .........|XXX|Leslie|Tom |            ← Ron's bytes left in place
   header:  free_space_ptr = 9, freeblocks_head = (offset=9, size=4)   ← availability list
```

The freed region `(offset=9, size=4)` is pushed onto the in-page **availability list** (SQLite name: **freeblocks**, stored as a linked list with the head pointer in the page header, plus a `total_free_bytes` counter for fast capacity checks).

##### Fit strategies for the next insert

When a new cell needs space, the engine walks the freeblock list looking for a usable hole. Two classic strategies, both inherited from malloc-style allocators:

| Strategy | Algorithm | Pros | Cons |
|---|---|---|---|
| **First fit** | Take the first freeblock ≥ requested size. | O(1)-ish (short list walk); fast | Tends to leave small useless remainders ("sliver fragmentation") |
| **Best fit** | Walk entire list; take the smallest freeblock ≥ requested. | Tighter packing, less fragmentation | O(n) list walk per insert |

A third option (not in this chunk, alluded to next page in DBI Ch.3): **defragment** — rewrite the cell region contiguously, eliminating all freeblocks at once. Engines trigger this when `total_free_bytes` is large but no single freeblock fits the requested insert. Compaction is O(page_size) but rare in steady state.

##### Why the availability list is a linked list, not an array
A page can have any number of variable-size holes after a string of inserts and deletes. A fixed-size array would need either (a) a hard cap on hole count or (b) repeated resizing. A **linked list threaded through the freed cells themselves** (each freed block's first 4 bytes = pointer to next freeblock) is the standard trick — it costs zero metadata bytes outside the freed space and grows naturally with the number of holes. This is the same intrusive-free-list trick `dlmalloc` uses, just scoped to a single page.

##### SQLite's actual numbers
SQLite's freeblock chain is stored with two-byte offsets (since pages are at most 64 KB). Each freeblock begins with `next_freeblock_offset (2B), block_size (2B)` — total 4 bytes of overhead per freeblock. The page header carries `first_freeblock_offset (2B), num_free_bytes (2B)`. This is concrete enough to memorize: **a SQLite page with 100 deleted-then-replaced cells has at most ~400 bytes of free-list overhead** out of 4 KB.

#### The complete picture (yesterday + today)

```
   ┌──────────────────────────────────────────────────────────────┐
   │ Page header                                                  │
   │   - cell_count, free_space_ptr,                              │
   │     first_freeblock_offset, total_free_bytes, LSN, ...       │
   ├──────────────────────────────────────────────────────────────┤
   │ Slot directory (sorted by key):                              │
   │   [→Leslie] [→Ron] [→Tom]                                    │  ← grows down
   │                                                              │
   │              ↓ free space ↓                                  │
   │                                                              │
   │   ┌──────── freeblock chain (XXX = freed) ─────────┐         │
   │   │                                                │         │
   │   │next→null │  cell: Ron  │ cell: Leslie │ cell: Tom        │  ← grows up
   │   XXXX                                                       │
   │   ▲ first_freeblock_offset                                   │
   └──────────────────────────────────────────────────────────────┘
```

Three independent data structures, three indirections, one page. Each indirection bought a property: **slot directory** = stable external IDs over compaction; **sorted pointers** = O(log n) lookup without sorted storage; **freeblock chain** = O(1) delete without rewriting.

### If you were the storage engine deciding between first-fit and best-fit…
You'd ask **how variable are the record sizes in your workload?** If most records are similar-sized (uniform schemas, fixed-length VARCHARs), first-fit's "leaves slivers" weakness rarely bites — the next insert is the same size as the one before. If records vary wildly (mixed BLOBs and tiny rows), best-fit's tight packing pays for its O(n) walk. SQLite ships first-fit. PostgreSQL's `heap` does something closer to best-fit at the page level but combined with `VACUUM`'s background compaction so the cost of imperfect packing is bounded. **The winning strategy is workload-dependent**, which is why production engines often offer it as a knob (or auto-tune based on observed fragmentation).

The deeper lesson: the page is a **micro-allocator**, and every allocator-design trade-off (fit policy, fragmentation tolerance, compaction cadence) reappears at this scope. Database internals are operating-system-internals scaled down to a single 4 KB block.

### Cross-language view
The sorted-pointer + payload-in-arrival-order pattern shows up beyond databases:

| System | "Cells" | "Sorted pointers" | Why same pattern |
|---|---|---|---|
| Java `String.intern()` interning table | Interned string bytes | Hash bucket → string offset | Strings arrive in any order; lookup is by hash |
| Protobuf `Message::ParseFromArray` with packed fields | Field bytes in wire order | Field-number → offset index built during parse | Wire order may differ from declaration order |
| Linux `ext4` directory entries | Variable-length `dirent` records in insert order | `htree` index of `(hash → block, slot)` | Same problem: variable-size names, frequent lookups |
| Lucene's `PostingsFormat` | Term postings in doc-order | Skip lists / FST for sorted access | Same: sequential write, sorted read |

The pattern's name in algorithms vocabulary is **"unsorted log + sorted index"** — the same shape that makes LSM-trees (DDIA Ch.3) outperform B-trees on write-heavy workloads at a higher scale. Slotted pages are the within-page version of that idea.

### Where this shows up in real systems
- **SQLite B-tree pages** are the closest production realization of this chunk's design: page header at offset 0, cell pointer array (sorted by key), free space, then cells growing up from the page end. SQLite's source file `src/btree.c` is ~10 KLOC of exactly this logic — and that file has barely changed in 15 years because the design is good.
- **PostgreSQL's heap pages** use the slotted layout *without* the sorted-pointer trick — heap tuples are stored in arrival order and slot pointers track them by `ctid`, with **separate B-tree indexes** providing the sorted access. The chapter's "sort the pointers" move is collapsed into "build a separate index." Both work; Postgres trades a denser heap for slower in-page key lookup. Different point on the same design spectrum.
- **InnoDB compact rows** use a clever variant — records are linked into a sorted singly-linked list by storing a "next record" offset in each row header, with a sparse slot directory ("page directory") storing every 4th–8th slot for binary-search start points. This compresses the directory ~8× at the cost of a final linear scan. Useful when you have many cells per page (small rows).
- **WAL replay and recovery** depends on the availability-list discipline: because deletes only update the freelist (not the cell bytes), recovering a partial-write page from the WAL is straightforward — replay the slot/directory updates and the freelist updates, the actual cell payloads were never touched by the failed transaction. If deletes compacted cells in-place, crash recovery would require re-emitting the entire page contents to the WAL.

### Diagnostic questions
1. **Why are fixed-size fields placed before variable-size fields in a cell, not after?** *Because the parser needs the size descriptors to locate the variable fields — and it can only locate the size descriptors trivially if they're at known fixed offsets. Reversing the order creates a chicken-and-egg parse problem requiring sentinels or trailing length markers.*
2. **A page has 4 KB free according to `total_free_bytes`, but an insert of a 1 KB cell fails. What's going on?** *Fragmentation: the 4 KB is spread across many small freeblocks, none large enough for the 1 KB cell. The engine must defragment (compact the cell region, merge all freeblocks into one) before the insert can succeed.*
3. **Why doesn't the slot directory just store keys directly, instead of pointing to cells?** *Because keys are variable-size — a directory of variable-size entries can't be binary-searched in O(log n) since you can't compute "the middle entry" in O(1). Fixed-size pointers preserve random-access into the directory, and the actual variable-size key lives in the (random-access) cell.*
4. **First-fit can degrade to terrible packing over time. What's the cheap mitigation?** *Periodic compaction (rewrite cells contiguously, rebuild slot directory). Postgres calls it `VACUUM`. SQLite triggers it implicitly when an insert finds no fit. The amortized cost is low because compaction is O(page_size) ≈ a single page write.*
5. **In the Tom/Leslie/Ron example, the slot directory shifts 2 bytes per insert. Why is that acceptable when "shifting cell bytes" was considered too expensive?** *Asymmetry of magnitudes. Slot pointers are 2 bytes; cell records are 40–400 bytes. Shifting 50 slot pointers = 100 bytes; shifting 50 cells = 2–20 KB. The 20–200× ratio is the entire payoff of the indirection.*

### See also
- DBI 2026-05-23 entry — the slotted-page *anatomy* (page header, bidirectional growth, slot stability) that today's mechanics live inside.
- DBI 2026-05-22 entry — *binary encoding primitives* (Pascal strings, bit-packed flags, fixed/variable layout discipline) that every cell uses field-by-field.
- DDIA Ch.3 *Storage and Retrieval* — the B-tree section uses these exact pages as its leaf format; LSM-tree section shows the larger-scale version of the same "unsorted log + sorted index" pattern.
- N2T 2026-05-24 entry — the C-instruction's fixed-prefix/variable-tail encoding is the same discipline at the CPU level; understanding one accelerates understanding the other.
- *Modern B-Tree Techniques* (Goetz Graefe, 2011) — exhaustive treatment of leaf-page layouts, including space-management strategies beyond first/best fit (e.g., bin-packing-style "next-fit").
- Forward link: DBI Ch.4 *Implementing B-Trees* — splits and merges propagate these cell layouts across pages; the cell-format invariants today let those operations stay simple.

---

## [2026-05-23] Slotted Pages — The Universal Variable-Record Page Layout (Why Every Real DB Has Pointers Growing Down and Records Growing Up) · pp.96–100 · Ch.3 *File Formats* §General Principles → Page Structure → Slotted Pages

### TL;DR
A database page must store **variable-size records** (rows with VARCHAR/BLOB columns) while letting outside pointers reference them stably, even as records are inserted, deleted, and the page is compacted. The naïve solutions — concatenated records, fixed-size segments — either require relocating bytes on every insert or waste up to 1 segment of space per record. The industry's near-universal answer is the **slotted page**: a fixed header at the top, a **slot directory growing downward** from the header, and **record cells growing upward** from the bottom of the page. The slot at index `i` is an immutable pointer "this record lives at offset X" — so external references use slot IDs, not byte offsets, and the storage engine is free to compact/rewrite the cell region without invalidating any pointer outside the page. PostgreSQL, MySQL/InnoDB, SQLite, and almost every B-tree implementation since System R use this layout.

### Intuition — "this is like…"
A slotted page is a **filing cabinet drawer with numbered tabs on the inside of the lid**. The tabs (slot directory) grow downward as you add files. The files themselves (cells) get crammed in from the back of the drawer forward. You don't tell your colleague "the contract is at byte 1,247 from the front of the drawer" — that breaks the moment someone defragments. You say "the contract is **tab #3**," and tab #3's internal annotation tells you where in the drawer to actually reach. When the drawer gets messy and you defragment, you slide files around but you **update tab #3's annotation**; the tab itself, and your colleague's reference to "tab #3," stays valid forever.

### Mechanics

#### What the chunk actually covers
The chunk straddles three sections of Ch.3: the **General Principles** of building structured binary formats (compose fields → cells → pages → sections → regions), then **Page Structure** (DBs partition files into 4–16 KB pages, the unit of I/O), then the **Slotted Pages** layout itself with the diagram on p.99 (Figure 3-5).

#### Why naïve layouts fail
Two predecessors the chapter dismisses:

| Layout | How it works | Why it fails |
|---|---|---|
| **Concatenated triplets** (Bayer's original 1972 B-tree paper) | `k₁ v₁ p₁ k₂ v₂ p₂ ...` packed contiguously | Inserts anywhere except the right edge require **shifting all following bytes**. Only works for fixed-size records (variable-size = no way to know where the next triplet starts). |
| **Fixed-size segments** | Split the page into N equal-size buckets, store each record in one | Wastes `64 - (n mod 64)` bytes per record at 64-byte segments. A 70-byte record wastes 58 bytes of the next segment. **Internal fragmentation tax** on every insert. |

The slotted page's job is to support three properties simultaneously:
1. **Variable-size records with minimal per-record overhead**
2. **Reclaim space from removed records** (so deletes aren't permanent leaks)
3. **External pointers to records survive page reorganization**

The third is the subtle one — and the reason for the directory's existence.

#### The slotted-page anatomy

```
   ┌─────────────────────────────────────────────────────┐  offset 0
   │  Page header                                        │  (fixed-size: page LSN, slot count,
   │  - free-space pointer                               │   free-space pointer, checksum,
   │  - slot count                                       │   page type, ...)
   │  - other metadata                                   │
   ├─────────────────────────────────────────────────────┤
   │  Slot 0 → offset 4080, length 16                    │  ← slot directory grows DOWN
   │  Slot 1 → offset 4020, length 60                    │
   │  Slot 2 → (free / tombstone)                        │
   │  Slot 3 → offset 3900, length 120                   │
   │  ▼ free-space pointer points here ▼                 │
   │                                                     │
   │                  free space                         │  (where new slots + new cells
   │                                                     │   converge as the page fills)
   │                                                     │
   │  ▲ free-space pointer points here ▲                 │
   │  [ cell #3: 120 bytes ]                             │  ← cells grow UP
   │  [ cell #1: 60 bytes  ]                             │
   │  [ cell #0: 16 bytes  ]                             │
   └─────────────────────────────────────────────────────┘  offset PAGE_SIZE (e.g. 4096)
```

**Two regions, two growth directions, one shared free-space middle.** The page is full when slot-directory-end meets cell-region-start.

#### Why two growth directions?
A common student question. The reason is **insertion ergonomics**:
- New cells are appended to the *bottom* of the cell region (which is the *top* of the cell stack, i.e., they grow upward toward the directory). No existing cells move.
- New slots are appended to the *bottom* of the directory (which grows downward toward the cells). No existing slots move.
- Both grow toward each other; the free-space pointer in the header tracks where they meet.

If both grew the same direction, every insertion would require shifting either all slots or all cells. The dual-direction layout makes **O(1) appends from both sides** the steady-state operation.

#### Three operations and what they do

| Operation | Slot directory | Cell region | External pointer impact |
|---|---|---|---|
| **Insert** | Append slot at end of directory pointing to new cell | Append cell at bottom of free-space region | None — new slot ID is the reference |
| **Delete** | Mark slot as free (tombstone) **or** remove + shift | Leave cell data (or zero it); space becomes reclaimable | If slots are renumbered after delete, external pointers break. **Most engines tombstone instead of shifting** to preserve slot stability. |
| **Compact / defragment** | Update each slot's offset to its cell's new position | Rewrite cells contiguously, eliminating gaps | None — slot IDs are stable; only their internal offsets change |

The genius is in row 3: **defragmentation is invisible to anything outside the page** because the addressing layer (slot ID) is decoupled from the storage layer (cell offset). This is the same indirection trick the OS uses for virtual memory page tables, or that Java uses for object references vs. raw pointers — pay one level of indirection, gain freedom to move the underlying data.

#### The reorganization invariant

```
Before compaction (page has dead space from deletes):
   slots:      [s0→4080] [s1→4020] [tombstone] [s3→3900]
   cells:      ..|cell3 120B|.....gap....|cell1 60B|cell0 16B|

After compaction:
   slots:      [s0→4080] [s1→4040] [tombstone] [s3→3920]
   cells:      .............................|cell3 120B|cell1 60B|cell0 16B|
   ▲ all free space now contiguous in the middle ▲
```

External references to `s0`, `s1`, `s3` are unchanged. The slot offsets changed — that's the whole point of going through the directory.

#### The page header (preview of next section)
Vernon will detail the header in the next section, but the chunk introduces the key fields: **free-space pointer** (where insert can append), **slot count** (directory length), and metadata for crash recovery (LSN — Log Sequence Number, covered in Ch.5–6 on WAL). The header is the *fixed-overhead tax* of the slotted layout — typically 24–40 bytes per page, amortized over hundreds of records.

#### Where slot IDs flow into the rest of the system
A B-tree leaf cell stores `(key, slot_pointer)` where `slot_pointer = (page_id, slot_id)` — a **tuple identifier** (Postgres calls it a `ctid`; InnoDB calls it a "record ID"). The whole B-tree structure rests on the slot ID being **stable under intra-page reorganization** but rewritten on cross-page moves (splits, merges). When InnoDB does a leaf split, the records move between pages and their TIDs change — which is why secondary indexes in InnoDB point at the primary-key value rather than the TID directly (to avoid cascading index updates on every split). That design choice traces back to the slot-ID stability boundary defined right here.

### If you were the storage engine handling a `DELETE` on a slotted page…
You face the **shift-or-tombstone** decision. Shifting (compact slots, renumber, also compact cells) gives you contiguous free space immediately and a smaller directory — but every external pointer to the deleted-and-later slots is now wrong, which means you'd need to chase them all (B-tree leaves, secondary indexes, in-flight transactions) and fix them up. That's a transitive write storm. Tombstoning (mark the slot free, leave a hole; reclaim the cell space lazily during a later compaction triggered by an insert or vacuum) keeps every external pointer valid for free, at the cost of carrying dead-slot baggage until the next compaction. Real engines (Postgres `VACUUM`, InnoDB purge, SQLite's freelist) all tombstone — the cost of maintaining external pointer validity is paid once at compaction time, not on every delete.

### Cross-language view
Slotted pages aren't a language feature, but the **slot-table / data-region split** appears all over systems software:

| Domain | Slot equivalent | Data region | Indirection benefit |
|---|---|---|---|
| OS virtual memory | Page table entry (VPN → PFN) | Physical frames | Process sees stable virtual addresses while OS moves physical pages |
| Java HotSpot | Object reference (`oop`) | Heap | GC can relocate objects during compaction |
| Filesystem | Inode (`(device, inum)`) | Disk blocks | File can be moved/defragmented; path lookup stable |
| HTTP routing | URL path | Backend instance | Service can scale/move; client URL stable |
| DNS | Hostname | IP address | Server can move; clients use the name |

The pattern's name in software-architecture vocabulary is **"add a level of indirection"** (Lampson's famous *"all problems in computer science can be solved by another level of indirection"*). The slotted page is the on-disk instance of the pattern.

### Where this shows up in real systems
- **PostgreSQL's heap pages** are textbook slotted pages: 8 KB by default, page header at top, line-pointer array (ItemIdData) growing down, tuples growing up. The `ctid` you see in `SELECT ctid, * FROM t` is literally `(page_number, slot_number)`. The "HOT update" mechanism (Heap-Only Tuple) is an optimization that lets new tuple versions reuse the same slot's pointer chain to avoid touching every index — the chain *lives in the slot directory*.
- **InnoDB's compact row format** uses the same layout with a twist: records are linked into a sorted singly-linked list via offsets in the record header, so the slot directory ("page directory") only stores every 4th–8th slot as a sparse index, with binary-search-then-linear-scan for lookup. Saves ~7/8 of the directory cost.
- **SQLite's B-tree page format** stores cell pointers as 2-byte offsets in a "cell pointer array" right after the page header — same idea, smaller scale. The format is so stable it's been backward-compatible since SQLite 3 launched in 2004.

### Diagnostic questions
1. **Why can't external references point directly at byte offsets within a page?** *Compaction moves cells. Any direct byte-offset reference would silently rot the moment the page is defragmented. The slot directory is the stable-naming layer that makes compaction safe.*
2. **What's the worst-case waste of fixed-size segmentation that slotted pages avoid?** *Up to (segment_size − 1) bytes per record. At 64-byte segments, a 65-byte record wastes 63 bytes of the next segment. Slotted pages waste only the per-slot pointer overhead (~4 bytes) regardless of record size variance.*
3. **A page has 100 slots with 30 tombstones. The free-space pointer says 0 bytes available, but tombstoned cells hold 2 KB. What does the engine do on the next insert?** *Trigger an in-place compaction: rewrite live cells contiguously, update their slot offsets, reset the free-space pointer. Then perform the insert. Tombstoned slots may or may not be reclaimed depending on the engine (Postgres reclaims them; some engines never reuse slot IDs).*
4. **Why might an engine choose to never reuse a slot ID even after the cell is gone?** *Slot IDs become parts of higher-level identifiers (TIDs, MVCC version chains, undo logs, replication streams). Reuse can confuse anything that's still holding the old ID — e.g., a long-running read transaction. Some engines burn through 32-bit slot space and just rebuild the page when they wrap.*
5. **Could you use slotted pages for fixed-size records too?** *Yes, but the slot directory becomes wasteful overhead — for fixed-size records, just compute `cell_addr = base + i × cell_size` and skip the indirection. The slotted layout's value is variable-size support; fixed-size workloads (some columnar formats, e.g., Parquet row groups) skip it for that reason.*

### See also
- DBI 2026-05-22 entry — *Binary Encoding Primitives* — the fixed-width and variable-length encoding primitives that the cells themselves use; the slotted page composes them at the page level.
- DDIA Ch.3 *Storage and Retrieval* (B-trees section) — page-level structure of B-trees; the slotted page is the page format on which B-trees are built.
- Forward link: DBI Ch.4 *Implementing B-Trees* — will use the slot directory's stability properties when discussing splits, merges, and right-most insertion optimization.
- N2T Ch.5 §5.3.2 (Memory) — the same "logical address space hiding multiple regions" trick, but at the hardware level (RAM16K + Screen + Keyboard behind a single Memory chip interface). Indirection scales from gates to gigabytes.

---

## [2026-05-22] Binary Encoding Primitives — Fixed-Width Numbers, Pascal vs. Null-Terminated Strings, and Bit-Packed Flags · pp.91–95 · Ch.3 File Formats § Binary Encoding (Primitive Types → Strings & Variable-Size Data → Bit-Packed Data: Booleans, Enums, Flags → General Principles)

### TL;DR
On disk and on the wire, everything is a byte sequence; turning a record into that sequence is **serialization**, the reverse is **deserialization**. The chapter walks the three primitive cases every storage engine must answer: (1) **fixed-width numbers** (`byte`/`short`/`int`/`long` = 1/2/4/8 bytes; floats follow IEEE 754 with sign/exponent/fraction layout), (2) **variable-length data** (Pascal-string `[size: u16][bytes]` vs. C-style null-terminated — Pascal wins on O(1) length and zero-copy slicing), and (3) **bit-packed booleans/flags** where each named bit costs **1/8th of a byte** and is manipulated by the classic `|`/`&`/`~`/`<<` quartet. These three primitives compose into every file header, every page format, every wire protocol you'll ever decode in a hex dump.

### Intuition — "this is like…"
A binary file is a **train of fixed-length and variable-length cars**, and the conductor (the parser) only knows where the next car starts if it's told either (a) the car has a fixed size baked into the schema or (b) the *current* car ends with a length-prefix saying how big the next one is. Null-terminated strings are the train where you find out a car ended by *walking the whole car until you trip on a sentinel sticker*; Pascal strings are the train where the conductor reads a number on the door before stepping in. Bit-packed flags are the train where eight binary stickers (open/closed) are crammed onto a single 1-byte door label — efficient if you can read the label without removing the other stickers.

### Mechanics

#### Fixed-width numeric primitives (the bedrock)
| Type | Size | Bit count | Range (signed, two's complement) |
|---|---|---|---|
| `byte` | 1 B | 8  | −128 … 127 |
| `short` | 2 B | 16 | −32,768 … 32,767 |
| `int` | 4 B | 32 | ≈ −2.1B … 2.1B |
| `long` | 8 B | 64 | ≈ ±9.2 × 10¹⁸ |
| `float` (IEEE 754 single) | 4 B | 32 | 1 sign · 8 exponent · 23 fraction |
| `double` (IEEE 754 double) | 8 B | 64 | 1 sign · 11 exponent · 52 fraction |

#### IEEE 754 single-precision layout (`float`)
```
 bit 31         bit 30 .... bit 23       bit 22 ........... bit 0
 +-----+----------------------------+-------------------------+
 |  S  |          exponent          |        fraction         |
 |  1b |           8 bits           |         23 bits         |
 +-----+----------------------------+-------------------------+
   sign     biased by 127           mantissa, implicit leading 1
```
Value = `(-1)^S × 1.fraction × 2^(exponent − 127)`. This is **why floating-point is approximate**: 0.1 has no finite binary fraction, so storing 0.15652 loses precision in the last few bits. The implicit leading `1.` in the mantissa is the encoding's free bit — by *not* storing it, you get 24 bits of precision out of 23 bits of fraction.

**Endianness footnote** (not in chunk but mandatory for visual grammar): the 4 bytes of a `float` can be laid out high-byte-first (**big-endian**, network byte order) or low-byte-first (**little-endian**, x86/ARM default). A file format must pick one; common protocols pick big-endian; common in-memory layouts on consumer CPUs are little-endian.

#### Strings — two schools
```
 Pascal string ("UCSD string"):
 +--------+---------+---------+----- ... -----+
 | size   |  byte 0 |  byte 1 |    byte size-1|
 | u16    |                                   |
 +--------+---------+---------+----- ... -----+
  ^ length prefix    ^ raw bytes (no terminator)

 C / null-terminated:
 +---------+---------+----- ... -----+---------+
 |  byte 0 |  byte 1 |    byte n-1   |   0x00  |
 +---------+---------+----- ... -----+---------+
  ^ walk until you hit the sentinel
```

| Property | Pascal string | Null-terminated |
|---|---|---|
| `len()` cost | **O(1)** — read prefix | **O(n)** — `strlen` walks |
| Max length | 2¹⁶ − 1 with u16 prefix (or 2³² with u32) | Unbounded, but unsafe |
| Embeds null byte? | **Yes** — bytes are opaque | **No** — `0x00` is the terminator |
| Slicing into a language string | Zero-copy: bytes are contiguous, length is known | Must scan first, then allocate |
| Memory-safety failure mode | None if prefix is honored | Buffer overrun, "off-by-one" classic (`strcpy`, `gets`) |

The DBMS-relevant punchline: **Pascal strings dominate file formats** because the parser is decoding many records per page and the O(n) scan to find a length would dominate the read cost. C strings dominate *language runtimes* because they were cheap to interop with the kernel in 1971.

#### Bit-packed booleans, enums, flags
**Booleans** waste 7 bits if stored as `0x00` / `0x01`. Pack 8 of them into a byte and you've reclaimed those bits — important for node headers where you might have 4-6 flag bits and storing each as a byte would bloat every page in the DB by tens of MB across a large table.

**Enums** are integers with names. Small (low-cardinality) enums fit in a single byte:
```
enum NodeType : u8 {
   ROOT     = 0x00,
   INTERNAL = 0x01,
   LEAF     = 0x02,
};
```

**Flags** are a *non-mutually-exclusive* combination. The encoding pattern is **one bit per named flag**, with a mask whose value is a power of two so it has exactly one bit set:
```c
#define IS_LEAF              0x01   // bit 0  (1 << 0)
#define VARIABLE_SIZE_VALUES 0x02   // bit 1  (1 << 1)
#define HAS_OVERFLOW_PAGES   0x04   // bit 2  (1 << 2)
```

#### The four bit-manipulation idioms (memorize once, reuse forever)
| Operation | Formula | Mental model |
|---|---|---|
| **Set** a bit | `flags |= MASK;` | OR-in the 1 |
| **Clear** a bit | `flags &= ~MASK;` | AND with the inverse |
| **Toggle** a bit | `flags ^= MASK;` | XOR flips it |
| **Test** a bit | `(flags & MASK) != 0` | AND isolates, compare to 0 |

```
 flags          = 0 0 0 0 0 1 0 1    <-- IS_LEAF + HAS_OVERFLOW already set
 HAS_OVERFLOW   = 0 0 0 0 0 1 0 0
 ~HAS_OVERFLOW  = 1 1 1 1 1 0 1 1

 SET:    flags |= HAS_OVERFLOW   -> 0 0 0 0 0 1 0 1   (no change, already set)
 CLEAR:  flags &= ~HAS_OVERFLOW  -> 0 0 0 0 0 0 0 1   (bit 2 cleared)
 TEST:   flags & HAS_OVERFLOW    -> 0 0 0 0 0 1 0 0   (nonzero -> true)
```

#### Worked example — a tiny B-Tree node header
Suppose you're designing a node header for the B-Tree variant from Ch. 2:
```
 offset  size   field
 ------  ----   -----
  0      1      node_type  (enum: 0=root, 1=internal, 2=leaf)
  1      1      flags      (bit 0: is_leaf, bit 1: var_size_vals, bit 2: has_overflow)
  2      2      key_count  (u16 — Pascal-style prefix for the keys[] array)
  4      4      next_page  (u32 — page number of right sibling, or 0)
  8      ...    keys[]
```
That's **8 bytes of fixed-size header**, of which 1 byte (`flags`) holds 3 booleans + 5 reserved bits. If your node count is 10 million, packing those 3 flags into a single byte (instead of 3 separate bytes) saves **~20 MB**. At the file-format level, these savings compound — DB engineers fight for every byte because every byte propagates through caches, buffer pools, and replication streams.

### Cross-language view
| Language | Fixed-width int type | "Pascal string" idiom | Bit-flag idiom |
|---|---|---|---|
| **C** | `<stdint.h>`: `int32_t`, `uint64_t` | Manual `[len][bytes]` | `#define` masks + `|`/`&` |
| **Rust** | `i32`, `u64` (built-in, no `<stdint.h>` needed); `f32`/`f64` are IEEE 754 | `Vec<u8>` carries its own length; `bytes::Bytes` zero-copy | `bitflags!` macro generates strongly-typed flag set |
| **Go** | `int32`, `uint64`, `float32`, `float64`; `encoding/binary` for explicit endianness | `[]byte` with explicit length; `binary.BigEndian.PutUint16` for prefix | `const ( FLAG_A = 1 << iota; FLAG_B )` |
| **Python** | `struct.pack("<I", x)` for fixed-width; native `int` is arbitrary-precision | `struct.pack(f"H{len(s)}s", len(s), s.encode())` | `IntFlag` from `enum` module gives flag arithmetic |

**Stdlib-actually-does notes:**
- **Go's `encoding/binary`** is the canonical reference — its source code is a 200-line tour of every primitive in this section.
- **Rust's `bytemuck` and `zerocopy`** crates let you `transmute` a `&[u8]` to a `&MyHeader` zero-copy *if* `MyHeader` is `#[repr(C)]` and contains only POD types — exactly the file-format pattern.
- **Python's `struct` module** is the chapter's API in one module: `struct.pack("<BBHI", node_type, flags, key_count, next_page)` produces the 8-byte header above, byte-for-byte.

### Where this shows up in real systems
- **TCP/IP header format** — every field is a fixed-width integer in network byte order; flag bits (SYN, ACK, FIN, RST) sit in a single 6-bit subfield; the URG/PSH pattern is exactly `mask | bit`. RFC 793 is the spec; tcpdump's `-X` decodes it live.
- **Linux page-table entries (PTE)** — a 64-bit word where bits 0–11 are flags (Present, Writable, User, Accessed, Dirty, …) and bits 12–51 are the physical-frame number. `pte & _PAGE_PRESENT` is the same `flags & MASK` idiom you saw above.
- **Postgres on-disk row format** — each tuple starts with `t_infomask` (16 bits of bit-packed status flags: HAS_NULL, HASVARWIDTH, HASOID, XMIN_COMMITTED, …). When a Postgres bug says "incorrect `t_infomask`," it's this byte being miscomputed.
- **Protocol Buffers / Thrift / Avro** — all three avoid Pascal strings *for numbers* (they use varints — variable-length integers — to save space on small values) but use Pascal strings for byte sequences. The chapter's framing makes the trade-off legible: fixed-width is fast, variable-width is small, and binary formats let you pick per-field.

### Diagnostic questions
1. **Why does the chapter prefer Pascal strings to C strings for on-disk records?**
   *Wrong answer interpretation*: "Because Pascal strings are safer" — true but not the chapter's reason. The chapter's reason is **O(1) length** and **zero-copy slicing**: a record parser doing 1000s of fields per page can't afford a per-string scan.
2. **You see `flags = 0b00000101`. Which flags are set?**
   *Wrong answer interpretation*: "Bits 1 and 3" — no, **bits 0 and 2** (LSB = bit 0). The pattern `1 << n` corresponds to bit position n counted from the LSB.
3. **A 32-bit float stores 0.15652 as an approximation. Why not exactly?**
   *Wrong answer interpretation*: "Not enough bits in general" — the precise reason is that **0.15652 has no finite binary fraction** (just as 1/3 has no finite decimal representation). More bits would tighten the approximation but never make it exact.
4. **Why is `MASK = 0x03` *not* a single flag mask?**
   *Wrong answer interpretation*: "Yes it is" — wrong. `0x03 = 0b11` has *two* bits set. Single-flag masks must be powers of two (`0x01, 0x02, 0x04, 0x08, …`).
5. **You batch 8 booleans into one byte. What's the cost?**
   *Wrong answer interpretation*: "Zero cost — pure win" — the cost is **(a)** atomic-update of a single flag now requires a read-modify-write on the whole byte, which matters under concurrency; and **(b)** every flag read needs a mask and compare, adding 2-3 instructions per access.

### See also
- [[dbi-2026-05-21-btree-mechanics]] — the B-Tree node format we just sketched is built out of these primitives; revisit the header for `IS_LEAF` and you'll recognize the flag-bit pattern.
- [[cod-notes]] — Computer Organization & Design's coverage of two's-complement and IEEE 754 sits one level below this chapter, explaining *why* the bit layouts are what they are.
- [[ddia-notes]] — DDIA's "Encoding and Evolution" chapter generalizes these tactics to schema evolution (Protocol Buffers, Avro, Thrift) — same primitives, plus the question of forward/backward compatibility.
- [[ostep-notes]] — page-table entry bits in OSTEP's VM chapters are *exactly* the bit-packed-flags pattern at the kernel/MMU boundary.

---

## [2026-05-21] B-Tree Mechanics — Node Format, Separator Keys, Fanout, Lookup/Insert/Delete with Splits and Merges · pp.66–90 · Ch.2 B-Tree Basics (FTL recap → On-Disk Structures → Ubiquitous B-Trees → B-Tree Hierarchy → Separator Keys → B-Tree Lookup → Node Splits → Node Merges)

### TL;DR
A B-Tree is a **disk-tuned n-ary search tree** where each node fits in one disk page and stores up to **N − 1 keys + N child pointers**, giving fanout N in the hundreds-to-thousands. Lookups walk **height ≈ log_N(total_keys)** — for a billion keys with fanout 500, that's just **3–4 page reads**, vs ~30 for a BST. Inserts and deletes preserve the tree's invariants by **splitting** overflowing nodes (the median key floats up to the parent) and **merging** under-full sibling nodes (or **rotating** keys between them). Every concrete B-Tree variant (B+Tree, B*Tree, Bw-Tree) is just a tweak on this template.

### Intuition — "this is like…"
A BST is a **single-floor library** where every book has its own room — to find a book you might walk 30 doors. A B-Tree is a **multi-shelf bookcase** where each shelf holds 500 books in sorted order, with the title of the *next* shelf written between shelves — you find the right shelf in 3 hops, then scan locally. The point is to amortize the *expensive thing* (walking to a new shelf = disk seek) over many *cheap things* (scanning books on a shelf = in-memory comparisons after the page is loaded).

### Mechanics

#### 1. Node anatomy

A B-Tree node is one page on disk. Schematically:

```
   ┌─────────────────────────────────────────────────────────────────┐
   │ header │ k₁ │ k₂ │ k₃ │ ... │ kₙ₋₁ │ free space  │← slot dir ← │
   │        │ p₀ │ p₁ │ p₂ │ p₃ │ ... │ pₙ          │             │
   └─────────────────────────────────────────────────────────────────┘
            ↑                                       ↑
            n−1 keys (sorted)                       n child pointers
              ↑
              kᵢ is a SEPARATOR: every key in subtree pᵢ < kᵢ < every key in subtree pᵢ₊₁
```

| Field | Purpose |
|---|---|
| `header` | node type (internal/leaf), key count, sibling pointers (in B+Tree leaves) |
| keys `k₁..kₙ₋₁` | sorted, used as separators |
| pointers `p₀..pₙ` | child page IDs (internal) or value pointers/embedded values (leaves) |
| slot directory | maps logical slot → byte offset, allowing variable-length keys without shifting |

#### 2. Fanout = the entire reason B-Trees exist

Fanout `N = (page_size − header) / (key_size + pointer_size)`. Concrete numbers:

| Page size | Key + ptr | Fanout N | Height for 10⁹ keys (log_N(10⁹)) | Reads per lookup |
|---|---|---|---|---|
| 4 KB | 16 B (8 key + 8 ptr) | ~250 | log₂₅₀(10⁹) ≈ 3.7 | 4 |
| 8 KB | 16 B | ~500 | log₅₀₀(10⁹) ≈ 3.3 | 3–4 |
| 16 KB | 16 B | ~1000 | log₁₀₀₀(10⁹) ≈ 3.0 | 3 |
| BST (fanout 2) | n/a | 2 | log₂(10⁹) ≈ 30 | **30** (each a seek) |

> **The 10× difference is *all* fanout**. The B-Tree is not faster per-comparison than a BST — it's faster because the CPU work inside a page is essentially free compared to the seek that fetched the page. Fanout converts seeks into local scans.

#### 3. The two flavors: B-Tree vs B+Tree

| Aspect | Classic B-Tree | B+Tree (what everyone actually uses) |
|---|---|---|
| Where values live | in any node (internal or leaf) | **only in leaves** |
| Internal nodes hold | keys + values + child pointers | keys + child pointers only |
| Range scans | traverse tree → emit | **walk linked leaves left→right** |
| Lookup ends | possibly at internal node | always at a leaf |
| Internal-node fanout | reduced by value size | **higher** (no values) → shorter tree |

```
   B+Tree shape (the real-world default):

                  ┌─────[ 40 │ 80 ]─────┐                  internal: separator keys only
                  │       │       │
            ┌────┘       │        └────┐
            ▼            ▼             ▼
        [10│25│40]   [50│65│80]   [85│92│99]               internal level (if more levels)
            │            │             │
            ▼            ▼             ▼
        leaf───────►  leaf───────►  leaf                   leaves linked left-to-right
        {k,v}          {k,v}         {k,v}                 leaves hold the actual values
```

Linked leaves are the secret weapon: range scans like `SELECT * WHERE date BETWEEN x AND y` become a single descent to the start, then a sequential walk — no re-traversing the tree.

#### 4. Lookup — walk the separators

```
   Find key K = 67:
     current = root
     while current is internal:
         find the smallest separator k_i in `current` such that K < k_i
         current = child pointer p_i  (or last pointer if K >= all separators)
     scan current (a leaf) for K
```

In our shape above: K=67 → root: 40 ≤ 67 ≤ 80, descend p₁ → [50,65,80] internal: 65 ≤ 67 ≤ 80, descend p₂ → leaf → linear scan finds 67. Three reads total.

> **In-page search**: once the page is in memory, you can binary-search the keys (Θ(log N) per page). Binary search inside a page is essentially free vs the page read — the chapter's point is that **you don't care about the in-page constant**, you care about the *number of pages touched*.

#### 5. Insert — splits propagate upward

Three cases:

```
   Case A: leaf has space → just insert sorted, done.

   Case B: leaf is full → SPLIT
   ────────────────────────────────────────────────────────────────
   before:   parent
              │
              ▼
            [10│25│40│50│65│80]   ← full leaf (assume max 6 keys)
   insert 35:
            [10│25│35│40│50│65│80]  ← 7 keys, overflow

   split at median:    [10│25│35]  ───linked──► [40│50│65│80]
                            ▲                         ▲
                            └────┐               ┌────┘
                                 ▼               ▼
                                 parent: insert separator 40

   Case C: parent now overflows → recursively split parent.
           If split reaches root → root splits → tree grows one level.
```

**Tree growth is bottom-up**: this is the *only* way a B-Tree gets taller. So height grows by 1 only when the *root* splits, which is rare — for fanout 500 the root splits once per ~500 inserts at the root level, which corresponds to billions of leaf inserts at lower levels.

#### 6. Delete — under-flow, merge, or rotate

```
   After delete, if a leaf is < min_keys (typ. ⌈N/2⌉):

   Option 1: ROTATE — borrow one key from a sibling through the parent
   ────────────────────────────────────────────────────────────────────
            parent:  [...│40│...]
                          │
                ┌─────────┴─────────┐
                ▼                   ▼
           [10│25]              [50│65│80]
                                       ↑ borrow 50 leftward
            ↓ result:
           [10│25│40]           [65│80]
            ↑ separator becomes parent's old 40
            parent: [...│50│...] (new separator)


   Option 2: MERGE — combine with sibling, pull separator down
   ────────────────────────────────────────────────────────────────────
            parent:  [...│40│...]
                          │
                ┌─────────┴─────────┐
                ▼                   ▼
           [10│25]              [50]
                ↓ merge
           [10│25│40│50]   (note 40 came down from parent)
            ↑ parent loses 40 → may itself underflow → recursive merge
```

> **Merges can propagate upward and shrink the tree.** Root with only one child becomes its own child — tree height drops by one. Symmetric to insert's growth.

#### 7. The invariants every B-Tree maintains

| Invariant | Why |
|---|---|
| All leaves at the same depth | Worst-case lookup time is uniform |
| Every node (except root) has ⌈N/2⌉ ≤ keys ≤ N − 1 | Bounded fill ratio → bounded height |
| Root has ≥ 1 key (if not empty) | Standard handling |
| Keys within a node are sorted | Enables binary search per page |
| For an internal node with keys k₁..kₘ and children c₀..cₘ: every key in cᵢ is in (kᵢ, kᵢ₊₁) | Tree is searchable |

The **min-fill rule** (⌈N/2⌉) is what limits the storage waste — at worst, a B-Tree is half-full and uses 2× the optimal disk space.

#### 8. Worked numerical example — million keys, real costs

| Parameter | Value |
|---|---|
| Page size | 8 KB |
| Avg entry size | 32 B (16 B key + 16 B value or ptr) |
| Fanout N | ~250 |
| Number of keys | 10⁶ |
| Tree height | log₂₅₀(10⁶) ≈ 2.5 → **3 levels** |
| Random-key lookup cost (SSD) | 3 × ~100 µs = **300 µs** |
| Random-key lookup cost (HDD, no buffer) | 3 × ~10 ms = **30 ms** |
| Same problem in a BST (height 20) | 20 × 10 ms = **200 ms HDD** — 7× worse |

In practice the upper levels of a B-Tree are pinned in the buffer pool, so a "cold" lookup is really 1 disk read (the leaf), not 3. That's why Postgres can do 100K+ point lookups per second on a hot B-Tree even though each touches a multi-GB table.

### Where this shows up in real systems
- **PostgreSQL B-tree indexes** — classic B+Tree with 8 KB pages; the planner uses index-only scans when no payload columns are needed.
- **MySQL InnoDB primary index** — IOT-style B+Tree; values are the entire row. Secondary indexes hold PK, leading to two B-tree walks per secondary lookup (see DBI 2026-05-20).
- **MongoDB WiredTiger** — B+Tree (default storage engine). Leaves linked for range scans.
- **SQLite** — B-Tree per table; pages typically 4 KB. The whole DB is one file with embedded B-Trees.
- **Filesystem indexes** — NTFS, ext4 directory indexes, HFS+ catalog tree, XFS metadata — all variants of B-Tree. The Linux kernel maintains generic `<linux/rbtree.h>` for in-memory but `lib/btree.c` for disk-resident.
- **Bw-Tree** (Microsoft) — lock-free, immutable B-Tree with delta chains; pioneered for SSD where in-place updates are expensive.

### Diagnostic questions
1. *"Why does B-Tree height almost never exceed 4 in practice?"* — Because fanout is in the hundreds and tree size is bounded by physical reality. For fanout N=500 and 4 levels: N⁴ = 6.25 × 10¹⁰ leaf slots — bigger than most databases. Most tables fit in 3 levels. (Wrong: "the algorithm enforces it" — no, the algorithm enforces *balanced* depth; the *value* of the depth is a function of fanout × size, both physical.)
2. *"What's the difference between a split and a rotation?"* — A split happens on insert into a full node (creates a new node, pushes a separator up). A rotation happens on delete when a sibling has spare keys (borrow one through the parent without changing parent count). Rotations are cheaper (one parent update, no new node). (Wrong: "rotations are for balancing" — that's BSTs; B-Trees stay balanced structurally.)
3. *"Why are B+Tree leaves linked left-to-right?"* — For range scans. `WHERE date BETWEEN x AND y` descends to the leaf containing x, then walks linked-leaf pointers until y. Without the links you'd have to re-traverse the tree for each next leaf. (Wrong: "for deletion" — not specifically.)
4. *"What happens if the root has only one child?"* — The single child is promoted to root and the old root is deallocated; tree shrinks one level. Mirror of root-split. (Wrong: "nothing — it's a valid B-Tree" — true, but wasteful; standard implementations collapse this.)
5. *"Why does fanout matter more than per-page search efficiency?"* — Because the dominant cost is the *number of disk pages touched*, not the CPU work per page. Once a page is in memory, binary search through 500 keys is ~10 comparisons in nanoseconds. The 100 µs to read the page from SSD dwarfs that. Fanout reduces page count; per-page search affects negligible constants. (Wrong: "binary search per page makes it faster" — true but irrelevant to the asymptotic win.)

### See also
- DBI 2026-05-20 (Files, Pages, Indexes, HDD/SSD) — the hardware + file-layout layer this B-Tree sits on.
- DDIA Ch.3 (upcoming) — B-Trees vs LSM-trees from the storage-engine choice perspective.
- CLRS 2026-05-20 (Merge Sort) — same divide-and-conquer logic at the algorithm level; B-Tree splits/merges are the persistent-data analogue of merge sort's combine step.
- N2T 2026-05-21 (Hack Machine Language) — far below this layer: the addressable memory unit (a 16-bit word) Hack uses is what a B-Tree node is built out of, scaled to a page.

---

## [2026-05-20] Files, Pages, Indexes, B-Tree Setup, and the Hardware Underneath — Why Storage Engines Look the Way They Do · pp.44–65 · Ch.1 close (Webtable physical layout → Data/Index Files → Buffering·Immutability·Ordering → Summary) → Ch.2 open (BST recap → Balancing → Disk-fit properties → HDDs → SSDs)

### TL;DR
A storage engine is **specialized files + indexes** sitting on top of hardware whose physical units (sector / page / block) and access costs (seek vs sequential, program vs erase) leak upward into every design decision above. The chapter's punchline is that every B-Tree variant, every LSM, every Bw-Tree is just a different point in a 3-variable space — **{buffering · mutability · ordering}** — chosen to fit how the underlying device wants to be talked to.

### Intuition — "this is like…"
The OS gives you a filesystem; the DB throws it away and builds its own. Why? Because the filesystem speaks "files of bytes" but the DB needs to speak "pages of records," and because the *disk* speaks "sectors of 512 B–4 KB" with brutal asymmetry between seek and scan. The DB is a translator that takes a logical query down through three reformatting layers — **record → page → block → sector** — and back up again. Each layer's quirks (block-aligned writes, erase-before-program, sequential cheaper than random) constrains the layer above.

### Mechanics

#### 1. File organization — three families

| Family | Order on disk | Read by key | Range scan | Insert cost | Real example |
|---|---|---|---|---|---|
| **Heap file** | insertion order | needs separate index | terrible without index | cheap (append) | Postgres tables |
| **Hash-organized** | hash(key) bucket | O(1) avg | terrible (no order) | cheap; rebalance on grow | DBM, some KV stores |
| **Index-organized (IOT)** | key order | one lookup | excellent (sequential) | expensive (in-place shift) | MySQL InnoDB primary, SQLite |

> **Why IOTs save a seek**: index leaf already *holds* the record. Heap files need `index lookup → follow pointer → data file seek` = 2 logical seeks. IOTs collapse it to 1.

#### 2. Page / record layout (logical)

```
   File on disk
   ┌───────────────────────────────────────────────────────┐
   │  Page 0  │  Page 1  │  Page 2  │ ... │  Page N-1     │   each page = 1+ disk blocks
   └───────────────────────────────────────────────────────┘

   One page (e.g., 8 KB)
   ┌──────────────────────────────────────────────────────┐
   │ header │ slot dir → ← record3 │ record2 │ record1    │   "slotted page": records grow left,
   │        │ [s1][s2][s3]│   ...   │         │           │   slot dir grows right; gap shrinks
   └──────────────────────────────────────────────────────┘
```

- **Tombstones**: deletes don't shrink in-place. A tombstone {key, ts, "deleted"} marks the record dead. GC later compacts pages. Why? In-place delete would force shifting every following record's slot offset — expensive, and worse, it would corrupt readers mid-flight without locks.

#### 3. Index taxonomy

| Axis | Options | Meaning |
|---|---|---|
| Primary vs secondary | primary = on PK, secondary = any other column | Every table has ≤1 primary, many secondaries |
| Clustered vs non-clustered | clustered = data lives in key order on disk | Range scans cheap iff clustered |
| What leaves hold | direct offset / primary key / actual record | InnoDB secondaries hold **PK**, not file offset → 2 lookups per secondary read |

**MySQL InnoDB secondary-index walk** (read `WHERE email = 'x'`):
```
  ┌── secondary idx on email ──┐         ┌── primary idx on id (IOT) ──┐
  │  walk B-tree by email      │ → PK →  │  walk B-tree by id          │ → row
  └───── ~log(N) seeks ────────┘         └──── another ~log(N) seeks ──┘
```

vs Postgres heap-table secondary index: one B-tree walk → CTID → one heap-page fetch. The InnoDB choice (PK indirection) trades read cost for not having to update every secondary index when the data row physically moves.

#### 4. The 3-variable design space (the chapter's load-bearing idea)

| Variable | "Yes" picks | "No" picks | Example |
|---|---|---|---|
| **Buffer in memory before write?** | LSM (memtable), Lazy B-Tree | classic in-place B-Tree | LSM amortizes; B-Tree pays per-op |
| **Immutable files?** | LSM, Bw-Tree, Bitcask, WiscKey | classic B-Tree | Immutable → append-only writes (SSD-friendly), but read amplification |
| **Ordered on disk?** | B-Tree, sorted LSM levels, IOT | Bitcask (insertion order) | Ordered → range scans cheap; unordered → faster writes |

Every storage engine you know is a point in this {0,1}³ cube. B-Tree = (no, no, yes). LSM = (yes, yes, yes). Bitcask = (no, yes, no). Bw-Tree = (yes, yes, yes) but with delta chains.

#### 5. B-Trees motivated by hardware (Ch.2 open)

The chapter walks BST → balanced BST → "why this fails on disk" → B-Tree, and the failure modes are *all hardware*:

| Property of BST | Why it loses on disk |
|---|---|
| Fanout = 2 | Tree height = log₂N. For N=10⁹ that's ~30 seeks. At ~10 ms/seek on HDD = **300 ms per lookup** |
| Random allocation | New nodes scattered across disk; pointer-chase pays full seek every level |
| Tiny node | One node ≪ one block; 99% of each block read is wasted I/O |
| Rotation on balance | Each rotation rewrites multiple nodes, each costing a seek |

B-Tree fixes all four: **fanout = block_size / entry_size** (~100–1000), so height drops to ~3–4 even for billions of keys. We'll see the actual node format in the next chunk.

#### 6. The hardware floor — HDD vs SSD (the *low-level* part)

**HDD physical hierarchy:**
```
   Platter ──► Track ──► Sector (512 B classic, 4 KB Advanced Format)
   ▲
   Head positions over track via arm + rotation
```
- **Seek time** = arm move (~3–10 ms) + rotational latency (½ × 60 / RPM, e.g., 7200 RPM → 4.2 ms avg)
- **Sequential throughput**: ~100–200 MB/s
- **Random IOPS**: ~100–200/s on a single 7200 RPM drive
- **Why this drives DB design**: 1 seek ≈ reading **1 MB sequentially**. Pack densely, avoid pointer-chasing.

**SSD physical hierarchy** (this is where the cell-level detail matters):

```
   Die
   └── Plane(s)
       └── Block ────► smallest ERASE unit (typ. 64–512 pages = 256 KB–4 MB)
           └── Page ─► smallest READ/PROGRAM unit (typ. 4–16 KB)
               └── Cell (string of 32–64 cells)
                   └── bits: SLC=1, MLC=2, TLC=3, QLC=4
```

| Operation | Granularity | Latency (typ.) | Constraint |
|---|---|---|---|
| Read | page (4–16 KB) | 25–100 µs | none |
| Program (write) | page | 200–500 µs | **only into pre-erased cells** |
| Erase | **block** (256 KB+) | 1.5–3 ms | must erase whole block before re-programming |

**The asymmetry is the key**: you cannot overwrite. To "update" a single byte:
1. Read the whole block into RAM (or onto a spare block)
2. Erase the block (the slow step)
3. Re-program the entire block with the modified page

This is what the **FTL (Flash Translation Layer)** hides. It maintains a logical→physical page map so writes always go to *already-erased* pages, while old physical pages are marked invalid and reclaimed by **garbage collection** later — exactly analogous to the tombstone+GC pattern at the DB level. The DB's preference for **immutable, append-only** files (LSM, Bitcask) is *hardware-shaped*: it matches what the FTL is already doing internally.

| Operation pattern | HDD-friendly? | SSD-friendly? |
|---|---|---|
| Random reads | ✗ (seek) | ✓ (no moving parts) |
| Random writes | ✗✗ (seek + rotational) | ✗ (write amplification: 1 logical write → many physical writes due to erase-block GC) |
| Sequential writes | ✓ | ✓✓ (best case — minimal WAF) |
| Sequential reads | ✓ | ✓ |

> **Write amplification factor (WAF)** = physical bytes written / logical bytes written. On poorly-tuned random workloads WAF can exceed 10×, eating SSD lifetime (each cell has finite program/erase cycles: SLC ~100K, TLC ~3K, QLC ~1K).

### Where this shows up in real systems
- **Postgres heap + B-tree secondary**: heap is insertion-ordered file; every secondary index stores `(key, CTID)` where CTID = (page#, slot#). HOT updates exist precisely to avoid invalidating secondary indexes when a row updates without changing indexed columns.
- **MySQL InnoDB**: primary index *is* the table (IOT). Secondary indexes store PK not row pointer — so secondary lookups always pay a primary-index traversal. PK choice matters enormously (random UUIDs = bad locality; auto-increment = good).
- **RocksDB / Cassandra / LevelDB**: LSM = {buffered: yes, immutable: yes, ordered: yes}. Memtable buffers; SSTables are immutable; compaction merges in sorted order. Built for SSDs.
- **Bitcask (Riak)**: {buffered: no, immutable: yes, ordered: no}. Pure append-log, in-memory hash index over offsets. Astonishingly fast writes, but full dataset's *keys* must fit in RAM.
- **Modern NVMe**: the page/erase-block asymmetry still exists but is hidden behind ~100 µs latencies. The DB-level designs (LSM, copy-on-write B-trees) remain optimal because they minimize WAF.

### Diagnostic questions
1. *"Why can't I just update a record in place on an SSD?"* — Because flash cells must be erased before re-programmed, and erase granularity is a *block* (256 KB+), not a page. Updating one byte means rewriting an entire block. (Wrong answer: "you can, SSDs allow random writes" — no, the *FTL* fakes that; the hardware can't.)
2. *"Why is B-Tree fanout always ~hundreds, never 2?"* — Because the node size is tuned to the disk page (4–16 KB), and one node holds (page_size / entry_size) entries. Fanout 2 = tree height ~30; fanout 500 = height ~3, even for billions of keys. The disk only pays per-seek, not per-byte-within-a-seek. (Wrong: "more comparisons per node" — that's CPU work, which is ~1000× cheaper than the seek it saves.)
3. *"Why does deleting a row not free disk space immediately?"* — Tombstone semantics: the page entry is marked deleted, GC reclaims later. Doing it inline would require shifting every following record's slot offset, breaking any in-flight reads and ballooning latency. (Wrong: "Postgres bug" — it's a deliberate MVCC design.)
4. *"Why do InnoDB secondary indexes store the PK and not a row pointer?"* — Because when a row physically moves (page split, compaction), every secondary index would need a pointer update. Storing PK means secondary indexes are invariant under physical motion, at the cost of a primary-index walk on every secondary lookup. (Wrong: "primary index is smaller" — Postgres uses pointers and is faster on secondary reads.)
5. *"Why does LSM win on write-heavy workloads on SSD?"* — Buffered + immutable + ordered: writes are batched in memtable, flushed as immutable SSTables (purely sequential writes), so WAF is low and the SSD's FTL stays happy. B-Trees do in-place updates → random writes → high WAF → SSD wear. (Wrong: "LSM is always faster" — read amplification is real; cold reads can touch many SSTables.)

### See also
- DDIA 2026-05-19 (Document vs Relational) — DBI is the storage-engine layer underneath those data models.
- DDIA Ch.3 (upcoming) — Storage and Retrieval; same territory from the systems-design angle.
- N2T Ch.5 / COD Ch.5 — memory hierarchy. Same locality argument one level higher: cache lines (64 B) are to L1 what pages are to disk.
- OSTEP Ch.36–43 (I/O, FFS, log-structured FS) — the FS layer the DB chose to bypass; LFS in particular pioneered the append-only, immutable approach DBs later adopted.

---

## [2026-05-15] Column-Oriented Storage — Layout, Compression, and the CPU Vectorization Win · pp.39–43 · Ch.1 § Column-Oriented Data Layout → § Wide Column Stores

### TL;DR
A **column-oriented store** physically lays out a table's data **one column at a time**, contiguously on disk, rather than one row at a time. Logically the table looks identical; physically, every value in a column sits next to other values *of the same type*. This unlocks three coupled wins: **(1)** analytical scans read only the columns the query touches (no wasted I/O on columns the query discards), **(2)** type-homogeneous values compress far better than mixed-type rows (run-length, dictionary, bit-packing), and **(3)** modern CPUs can apply **SIMD vector instructions** across a column run with one instruction per N values. The price is that *reconstructing* a row for a point lookup requires reading N column files and stitching values by **implicit position (virtual ID)** or explicit key — slow for OLTP, irrelevant for OLAP. The chapter is careful to distinguish columnar stores from **wide-column stores** (BigTable, HBase, Cassandra) which are still *row-oriented inside column families* — the name overlaps, the layout doesn't.

### History — "why does this exist?"
The columnar idea is older than people think. **Sybase IQ (1996)** was the first commercially successful column store, sold for data warehousing. Academia caught up with **MonetDB (CWI Amsterdam, 1993)** and especially **C-Store (Stonebraker et al., MIT/Brown, 2005)**, whose paper laid out the formal arguments — vectorized execution, late materialization, type-specific compression. C-Store directly spawned **Vertica (2005, acquired by HP 2011)**. The Hadoop ecosystem produced **RCFile (Facebook, 2011)** then **ORC (Hortonworks, 2013)** and **Parquet (Twitter+Cloudera, 2013)** as on-disk columnar file formats — chosen explicitly because Hadoop's MapReduce workloads were analytical scans over wide tables, exactly the case row-oriented HDFS files punished. The **2010s Cambrian explosion** — **ClickHouse (Yandex, 2016)**, **Apache Kudu (Cloudera, 2015)**, **Druid (Metamarkets, 2011)**, **Snowflake (2014)** — all rest on this same C-Store recipe with refinements. The textbook's NOTE about *vectorized instructions* points at a parallel evolution: **Intel's SSE (1999) and AVX (2011)** brought 128-/256-bit SIMD to commodity CPUs, and column stores were perfectly positioned to use them — `SUM(price)` over a million rows becomes 250,000 AVX2 adds instead of 1,000,000 scalar adds.

### Intuition — "this is like…"
A **filing cabinet for a large company**. Row-oriented is one folder per employee — each folder contains their name, salary, hire date, manager. Perfect when HR asks "tell me everything about Alice" (one folder pull). Column-oriented is one drawer per attribute — a "salary" drawer with every employee's salary cards in employee-ID order, a "hire date" drawer with every hire date in ID order. Painful when HR wants Alice's full record (walk to every drawer, find slot 4271 in each). Glorious when Finance asks "average salary" (open one drawer, scan it, done — never touch the hire-date drawer). And since the salary drawer holds only numbers, you can **compress** it (RLE: "47 cards in a row showing $50k"), which you couldn't do in the mixed-type employee folders.

### Mechanics

**Same table, two physical layouts:**

```
LOGICAL VIEW (what SQL sees):

  ID | Symbol | Date        | Price
  ---+--------+-------------+----------
  1  | DOW    | 08 Aug 2018 | 24,314.65
  2  | DOW    | 09 Aug 2018 | 24,136.16
  3  | S&P    | 08 Aug 2018 |  2,414.45
  4  | S&P    | 09 Aug 2018 |  2,232.32

ROW-ORIENTED (one record per disk block):
   Block 1: [1, DOW, 08 Aug 2018, 24,314.65]
   Block 2: [2, DOW, 09 Aug 2018, 24,136.16]
   Block 3: [3, S&P, 08 Aug 2018,  2,414.45]
   Block 4: [4, S&P, 09 Aug 2018,  2,232.32]

COLUMN-ORIENTED (one column per file, position = row index):
   Symbol file: [DOW, DOW, S&P, S&P]
   Date   file: [08 Aug, 09 Aug, 08 Aug, 09 Aug]
   Price  file: [24314.65, 24136.16, 2414.45, 2232.32]
```

**Why each optimization is *only* possible with the columnar layout:**

| Optimization | Row store | Column store |
|---|---|---|
| Read only needed columns | ✗ (must read full rows) | ✓ (skip whole files) |
| Type-specific compression | weak (mixed types per block) | strong (e.g., RLE on `Symbol`, delta-encode `Date`, bit-pack small ints) |
| SIMD vectorization | hard (values are scattered) | natural (column runs are contiguous of one type) |
| Late materialization | impossible | feasible — filter on Price first, materialize matching rows last |
| Point-lookup of one row | fast (one block) | slow (N reads, stitch by virtual ID) |

**Implicit virtual IDs (the trick that makes stitching cheap):**

Instead of storing `(rowID, value)` pairs explicitly — which would defeat the size win — column stores rely on **positional alignment**: the 5th entry of `Symbol` corresponds to the 5th entry of `Date` corresponds to the 5th entry of `Price`. Reconstructing row 5 is *N pointer reads*, not *N hash lookups*. Sorting requires keeping all columns in the same order, which is enforced at write time.

**The SIMD payoff, sketched:**

```
SUM(Price) over 1M rows:

ROW STORE:
  loop: load row → extract Price field → add to sum   (1M scalar ops, cache misses on row reads)

COLUMN STORE (with AVX2, 4 doubles per instruction):
  loop: VLOADPD ymm0, [price+i]                       (250k vector ops)
        VADDPD  ymm_acc, ymm_acc, ymm0
  horizontal-sum ymm_acc at end
```

Theoretical 4× speedup from SIMD alone, often 10–40× total after compression cuts I/O.

**Columnar vs Wide-Column — the naming trap:**

```
Column-oriented (Vertica, ClickHouse, Parquet):
  ↳ analytical, scan-heavy
  ↳ physical layout: one file per column

Wide-column (BigTable, HBase, Cassandra):
  ↳ key-value with row keys + nested column families
  ↳ physical layout: still row-major INSIDE a column family
  ↳ "wide" = sparse rows can have millions of columns,
            not "we store columns separately on disk"
```

The names collide because both descend from a 1960s "column-major" terminology, but the **physical layout decisions are different** — wide-column is row-oriented within each column family. Treating Cassandra as a columnar analytics engine is a classic architecture mistake.

### If you were the storage engine designer…
You're told: "build a store that handles both OLTP point reads and OLAP scans well." Pure columnar punishes point reads (N seeks per row); pure row punishes scans (read full rows to extract two columns). The **hybrid answer** is what modern systems converged on: **(a)** keep small recent writes in a row-oriented in-memory buffer ("delta store"), **(b)** periodically flush to a columnar on-disk format, **(c)** at query time, merge the two. Kudu, SingleStore, SAP HANA, and Snowflake all use variants of this. Alternatively: **PAX layout** (Partition Attributes Across) — a hybrid disk format where each page holds rows but groups same-column values within the page. Compromise on every axis; perfect at none.

### Where this shows up in real systems
- **Parquet files on S3 + Athena/Spark/Trino.** The de facto data-lake standard is Parquet (columnar, per-column compression, column statistics for predicate pushdown). Athena charges by *bytes scanned* — and because Parquet lets the engine read only relevant columns, switching a row-oriented JSON warehouse to Parquet routinely cuts query cost 10–100×.
- **ClickHouse for real-time analytics.** Cloudflare's HTTP analytics, Uber's metrics, GitLab's CI logs — all sit on ClickHouse's columnar engine because the workload is "millions of rows ingested per second, scanned for aggregates." The columnar layout + LZ4 + delta encoding routinely hits 10:1 compression on web logs.
- **The "Cassandra is columnar" misconception.** Engineering interviews still ask "is Cassandra columnar?" The chapter's distinction is the answer: Cassandra is a **wide-column** store — row-oriented per partition, just with sparse and very wide rows. It will lose badly to ClickHouse or Druid on analytical scans.
- **DuckDB.** An in-process columnar OLAP engine (2019–) is rapidly replacing pandas + SQLite for analytical Python workloads — same C-Store recipe, in a single file.

### Diagnostic questions
1. **Q:** A workload reads 2 of 50 columns and filters on 1 of them. Row store or column store?
   *Wrong-answer trap:* "Depends on size." Column. Even at small sizes, the I/O win from skipping 48 columns dominates; the per-row reconstruction cost is incurred only on the (filtered) survivors.
2. **Q:** A workload does `SELECT * FROM users WHERE id = ?` on a 100-column table. Row or column?
   *Wrong-answer trap:* "Column for compression." Row. Point lookup wants one block read; column store would do 100 file reads.
3. **Q:** Why does columnar compression *enable* faster queries rather than just save disk?
   *Wrong-answer trap:* "Cheaper storage." The deeper reason: less I/O *to do the same scan*. Disk and network are slower than CPU decompression — compressed columns are scanned faster end-to-end.
4. **Q:** Cassandra calls its layout "wide-column." Why is that not the same as Vertica's columnar?
   *Wrong-answer trap:* "It is, both store columns." Cassandra still stores **all of a row's columns in the same partition contiguously**, sorted by column name within the row. Vertica stores a column *across all rows* contiguously. The axis of contiguity is different.

### See also
- DDIA Ch.3 "Storage and Retrieval" — extends this discussion with **column-oriented compression** (bitmaps, run-length) and **sort keys** in much more detail.
- DBI later chapters on **B-Tree vs LSM** — orthogonal: B-Tree and LSM are about *how a column file is internally organized*, not about row vs column orientation.
- COD §6.4 (vectorization) — the SIMD instruction set that columnar engines exploit.

---

## [2026-05-14] Durability in In-Memory Databases — WAL + Backup + Checkpoint · pp.34–38 · Ch.1 § Memory- vs Disk-Based → § Column- vs Row-Oriented intro

### TL;DR
An "in-memory" database is not the same as "data lives only in RAM" — that would be a cache, not a database. Real in-memory systems (Redis with persistence, MemSQL/SingleStore, VoltDB, Tarantool) preserve durability via a **three-layer trick**: (1) every committed write is first appended to a **sequential write-ahead log** on disk, *before* the client gets the acknowledgement; (2) an on-disk **backup** holds a sorted snapshot of the database as of some past point; (3) **checkpointing** periodically applies the log up to a marker onto the backup, after which the prefix of the log can be discarded. On crash recovery, the database loads the backup and replays the log suffix from the last checkpoint. The chapter ends this section by hammering the point that an in-memory DB is **not just an on-disk DB with a giant page cache** — the storage *format* (pointer-rich, variable-size-friendly) is fundamentally different, and that format is what enables performance the page-cache approach can't match. The section then transitions into **row vs column orientation**, the other dimension along which storage engines split.

### History — "why does this exist?"
The WAL discipline was formalized by **C. Mohan's ARIES paper (IBM, 1992)** for traditional disk databases — its rule "log records describe physical page changes, and the log is forced to disk before the page" is the textbook durability primitive. The in-memory database design — log + backup + checkpoint, *minus* the page-cache layer — was pioneered by **IBM TPF** (transaction processing facility, 1979, airline reservations) and revived for the modern era by **H-Store / VoltDB** (Stonebraker, MIT/Brown/Yale, **2007**). Stonebraker's pitch was: DRAM is large enough for OLTP working sets (a TPC-C database fits in 64 GB), so the page cache, B-tree latches, and buffer manager — *all of which exist to manage disk slowness* — are dead weight. Strip them, keep only the durability log and a periodic backup, and you get 50× the throughput on the same hardware. Every modern in-memory DB (MemSQL/SingleStore 2013, Hekaton inside SQL Server 2014, Tarantool, Redis with `appendonly yes`) is a variation on this 2007 H-Store recipe. The "backup is *not* a page cache" caveat (the NOTE on p.35) is a direct response to the early-2010s pushback from disk-DB vendors who claimed "we already do this with a big buffer pool" — the chapter's point is that they don't, because their on-disk format wasn't *designed* for the in-memory case.

### Intuition — "this is like…"
A bookkeeper running a busy shop:
- The **ledger** (WAL) is a sequentially-numbered pad where every transaction is scribbled the instant it happens — fast, append-only, never rewritten.
- The **filing cabinet** (backup) is a tidy alphabetized archive of how the books looked at the end of last Tuesday.
- **Checkpointing** is the Friday-afternoon ritual where the bookkeeper takes Tuesday's cabinet, applies all the pad entries from Wednesday morning to Friday morning, and produces a new cabinet labeled "as of Friday morning" — after which Wednesday/Thursday's ledger pages can go in the recycling bin.
- If the shop burns down (crash), the bookkeeper grabs the cabinet and the *unburnt* ledger pages and reconstructs the books. **No transaction is lost** because every transaction was scribbled in the ledger *before* the client walked out the door.

The chapter's caveat: the cabinet (backup) is **not** "the same books, just cached." It's a *different physical organization* — designed for sequential bulk load, not for live updates.

### Mechanics

**The three artifacts and their roles:**

```
                  ┌──────────────────────────────┐
  client write ──►│ in-memory data structures    │  ◄── fast reads
                  │  (skip lists, hash tables,   │      from here
                  │   pointer-rich, variable-    │
                  │   size objects)              │
                  └──────────────┬───────────────┘
                                 │ (1) append + fsync
                                 ▼
                  ┌──────────────────────────────┐
                  │ sequential write-ahead log   │  ← durability lives here
                  │ /var/lib/db/wal.NNNN         │
                  └──────────────┬───────────────┘
                                 │ (2) batched, asynchronous
                                 ▼
                  ┌──────────────────────────────┐
                  │ on-disk sorted backup        │  ← recovery reads this first
                  │ /var/lib/db/snapshot.bin     │
                  └──────────────────────────────┘
```

**The commit path, step by step (p.35):**

1. Client sends `INSERT user(42, 'Ada')`.
2. Server applies it to the in-memory structures.
3. Server appends a log record to the WAL and *waits for the fsync*.
4. Server returns `OK` to the client.

Step 3 is the **load-bearing line** — durability is defined by what survives between steps 3 and 4. If the machine dies between 2 and 3, the client never got `OK`, so the in-memory write was never "real." If it dies between 3 and 4, the log has the record; on recovery, replay re-applies it; the client may retry but no committed data is lost.

**Checkpointing — turning log volume into bounded recovery time:**

```
Time ──►
  WAL:   |  L1  L2  L3  L4 ... L9999 |  L10000 L10001 ...
                                ▲
                            checkpoint
                            marker

  Backup:  snapshot @ L9999
                            (newer than backup @ L0)

  Recovery cost = load backup + replay (current_LSN − checkpoint_LSN)
                  ─────────────         ──────────────────────────
                  bounded by backup     bounded by checkpoint interval
                  size, ~minutes        ~seconds-minutes
```

Without checkpoints, recovery time grows with *total database age*. With checkpoints every N minutes, recovery time grows with *N*, regardless of how long the database has been running. The trade-off: checkpointing is I/O-expensive, so you space it out as much as your RTO budget allows.

**Why "in-memory DB ≠ disk DB with huge page cache" — the format argument (p.35–36 NOTE):**

| Aspect | Disk-based DB w/ huge buffer pool | In-memory DB |
|---|---|---|
| On-disk format | Fixed-size pages (4–16 KB) tuned for sequential disk reads | Whatever the *backup* format wants — often log-structured or sorted runs |
| In-memory format | The page format, cached | Pointer-rich native objects (skip lists, ART trees, hash maps with pointers) |
| Variable-size values | Slot pointer indirection + overflow pages | Native — just point to it |
| Random access | Goes through buffer manager, page latch, slot lookup | Plain pointer dereference |
| Wide-and-short tree depth | Optimized for ~5 disk reads | Doesn't matter; trees can be tall and narrow |

The in-memory DB *cannot* simply reuse the disk-DB code with bigger caches because the **format is the source of the speedup**, not the absence of disk I/O. Eliminating the buffer manager removes overhead — but the *real* win is using data structures (linked lists, pointer trees) that page-oriented storage forbids.

**The transition to column/row (p.37–38):**

Storage engines also split along a second axis — how multi-field records are laid out *within* the chunks the storage engine reads:
- **Row-oriented** (MySQL, Postgres) groups all fields of one record contiguously. Good for "fetch the whole user record." Block-aligned I/O reads all columns even if you only want one.
- **Column-oriented** (Vertica, ClickHouse, MonetDB) groups all values of one column contiguously across rows. Good for "average the price column across 100M rows." Bad for whole-record fetch.

The two axes (memory-vs-disk, row-vs-column) are *orthogonal* — you can have all four combinations (Postgres = disk+row; ClickHouse = disk+column; Redis = memory+row-ish; MemSQL columnstore = memory+column).

### If you were the storage engine…

A transaction commits 100 writes in a batch. How many fsyncs do you issue?

**One** — and that one is the entire reason batching exists. Each fsync is ~1–10ms on NVMe, ~10ms on spinning disk. If you fsync per write, your peak throughput is bounded by `1 / fsync_latency` ≈ 100–1000 commits/sec. If you batch — buffer the log records, fsync once at the batch boundary, then ack *all 100 clients* — your throughput jumps two orders of magnitude. The cost: each client's commit latency is now bounded by *the slowest write in the batch* plus the fsync. This is **group commit**, invented in System R (1979) and rediscovered everywhere from MySQL's `binlog_group_commit_sync_delay` to Kafka's `linger.ms`. The chapter doesn't name group commit yet — but the "applied in batches to reduce the number of I/O operations" line is foreshadowing exactly this technique, which Chapter 5 will unpack.

### Cross-language view
*(n/a — durability mechanics are language-independent. The same WAL+checkpoint pattern appears in C (Postgres, SQLite), C++ (RocksDB, ClickHouse), Java (Cassandra, HBase), Go (CockroachDB, BadgerDB), and Rust (TiKV, SurrealDB). What changes is the I/O abstraction — `fsync(2)` in C, `os.File.Sync()` in Go, `tokio::fs::File::sync_all` in Rust — not the algorithm.)*

### Where this shows up in real systems

- **Redis `appendonly yes` (AOF) + RDB snapshots.** This is *literally* the chapter's recipe: AOF is the WAL (every command appended, fsync per write/per-second/never depending on config), RDB is the periodic backup, `BGREWRITEAOF` is the checkpoint. The `appendfsync` settings (`always`/`everysec`/`no`) are the dial that trades durability for throughput.
- **PostgreSQL `pg_wal/` + base backup + `archive_command`.** Disk-based but the *durability mechanism is identical*. `pg_basebackup` produces the backup; WAL segments produce the log; PITR (point-in-time recovery) is checkpoint-aware replay of the WAL suffix. The fact that this entry's pattern fits Postgres exactly is the chapter's quiet point: **WAL+backup+checkpoint is universal, the in-memory case just removes the buffer pool from the picture.**
- **SQLite's WAL mode (since 3.7.0, 2010).** When SQLite operates in WAL mode, the database file is the "backup," `wal` and `wal-shm` are the log, and `PRAGMA wal_checkpoint` is the explicit checkpoint command. The fact that SQLite has *two* journaling modes (rollback journal and WAL) directly reflects whether you want to read while writing (WAL allows it) or not.
- **Kafka as a generalized WAL.** Kafka is *just* the log layer of this picture, extracted into its own service. "We use Kafka as our database's source of truth" = "we run a distributed WAL and reconstruct the database (your search index, your cache, your downstream service) by replaying from the log" — exactly the chapter's recovery procedure, but with consumers replacing the in-memory data structures.

### Diagnostic questions

1. **Q:** Why must the log fsync complete *before* the client gets the commit ack, but the backup write can be asynchronous?
   *Wrong-answer trap:* "Because the backup is just an optimization." It's not — the backup is required for *bounded recovery time*. The reason the *fsync order* differs: durability is defined by "what survives a crash given what the client was told." If the client was told `OK`, the log must contain it. The backup is just a checkpoint of past-already-committed data — losing the latest backup is fine, you just replay more log.

2. **Q:** Why can't an in-memory DB just reuse a disk DB's storage engine code with a 100 GB buffer cache?
   *Wrong-answer trap:* "It would be too slow because of latches/page management." Partly — but the deeper reason is **data-structure choice**: the disk DB's format is built around fixed-size pages because random disk I/O is expensive in page-size chunks. With RAM, you want pointer-rich structures (skip lists, ART) that the page format *forbids*. The format mismatch is the root, not the buffer manager overhead.

3. **Q:** What goes wrong if you set the checkpoint interval to "once a day"?
   *Wrong-answer trap:* "Recovery is slow." Yes — and the *log grows* until disk fills, because log records can't be discarded until checkpointed. A node crashing 23 hours into the day must replay 23 hours of log; if your write rate is 10K ops/sec, that's ~800M log records to apply. Most systems target ~30 sec to a few minutes between checkpoints for exactly this reason.

4. **Q:** Why does the chapter introduce row vs column *immediately after* memory vs disk?
   *Wrong-answer trap:* "They're related by storage layout." More precisely: **they're orthogonal axes** that together span the four corners of storage-engine design. Petrov is setting up the rest of Part I — Chapters 3–7 trace the row+disk axis (B-trees, LSM); Part II addresses column and memory variants. The pair sets the taxonomy.

5. **Q:** If RAM is volatile and disk is non-volatile, why don't we just store everything on disk?
   *Wrong-answer trap:* "Speed." The trade-off the chapter implies: disk RAS (random access speed) is ~10K× worse than RAM RAS. Modern OLTP working sets are *small* (~10s of GB) — they fit in RAM on commodity servers. The in-memory architecture is rational specifically when the working set fits; it stops being rational at analytics-warehouse scale where you have 100 TB of cold data per active GB.

### See also
- [dbi-notes.md](Notes/dbi-notes.md) 2026-05-13 — *DBMS Architecture: The Six-Box Decomposition* — the buffer manager + recovery manager boxes are exactly the components this entry discusses *removing* (buffer) and *keeping* (recovery).
- [ddia-notes.md](Notes/ddia-notes.md) — Kleppmann's Ch.3 (storage engines) covers the same WAL+SSTable mechanics from the disk-DB side; this entry is the in-memory complement.
- [ostep-notes.md](Notes/ostep-notes.md) — the chapter on file systems and journaling: `ext4`'s journal is the *exact same* WAL discipline applied to filesystem metadata. Same algorithm, different layer.
- [dsg-notes.md](Notes/dsg-notes.md) — distributed services that use Raft are running a replicated WAL; the leader's log is the WAL, followers are warm backups, snapshotting is checkpointing.

---

## [2026-05-13] DBMS Architecture — The Six-Box Decomposition · pp.29–33 · Ch.1 § DBMS Architecture

### TL;DR
Petrov draws the canonical DBMS block diagram: a **transport subsystem** in front, a **query processor / optimizer** in the middle, an **execution engine** below it, and a **storage engine** at the bottom that itself fans out into **four sub-components** — transaction manager, lock manager, access methods, buffer manager, recovery manager. The diagram is the **table of contents of the book in disguise**: every later chapter is "deep dive on one box of Figure 1-1." The same six-box decomposition applies whether the database is disk-based (Postgres, MySQL) or in-memory (Redis, MemSQL/SingleStore) — what changes is the *ratio of work* per box and the data structures each box uses, not the box layout.

### History — "why does this exist?"
The six-box decomposition crystallized in the **System R papers (IBM, 1974–1979)** — System R was the first DBMS to cleanly separate parsing, optimization, execution, and storage in code, and its descendants (DB2, Oracle, Postgres) inherited the architecture. **Hellerstein, Stonebraker & Hamilton's *Architecture of a Database System*** (Foundations and Trends in Databases, 2007) is the canonical retrospective and the source Petrov cites first; it codified what every DBMS textbook has copied since. The diagram has been stable for **~50 years** because the boundaries it draws (parse vs. plan vs. execute vs. persist) correspond to **real abstraction barriers** — you can swap optimizers (Postgres's vs. CockroachDB's) without rewriting the storage engine, and vice versa. The Stonebraker/Hellerstein paper opens by complaining that *most engineers building data systems in the cloud era have never seen this diagram*, which is why a fresh DBI-style book existed in the first place.

### Intuition — "this is like…"
A DBMS is a **factory floor with five workstations and a loading dock**. The loading dock is the transport subsystem — trucks arrive carrying queries. Workstation 1 (parser) verifies the work order is legal. Workstation 2 (optimizer) figures out the *cheapest production sequence*. Workstation 3 (executor) runs that sequence, fetching parts from the warehouse as needed. The warehouse itself (storage engine) has four staff: a foreman (transaction manager), a security guard (lock manager), a librarian (access methods), and a cache clerk (buffer manager), plus a black-box flight recorder (recovery manager) that lets the foreman reconstruct any half-finished job after a power outage.

### Mechanics

**Figure 1-1, redrawn in ASCII:**

```
                  ┌──────────────────────────────────────┐
   client ──────► │      TRANSPORT SUBSYSTEM             │ ◄──── peer node
                  │ (network I/O, cluster comms, auth)   │
                  └──────────────┬───────────────────────┘
                                 │  raw query
                                 ▼
                  ┌──────────────────────────────────────┐
                  │      QUERY PROCESSOR                 │
                  │   parse → validate → access control  │
                  └──────────────┬───────────────────────┘
                                 │  parsed AST
                                 ▼
                  ┌──────────────────────────────────────┐
                  │      QUERY OPTIMIZER                 │
                  │  eliminate redundancy; cost-based    │
                  │  plan search using stats + placement │
                  └──────────────┬───────────────────────┘
                                 │  execution plan
                                 ▼
                  ┌──────────────────────────────────────┐
                  │      EXECUTION ENGINE                │
                  │  drives plan; fans out to remote     │
                  │  nodes; aggregates results           │
                  └──────────────┬───────────────────────┘
                                 │  local ops
                                 ▼
       ╔═════════════════════════════════════════════════════════╗
       ║                  STORAGE ENGINE                         ║
       ║                                                         ║
       ║  ┌────────────────┐  ┌─────────────────────┐            ║
       ║  │ Transaction    │  │ Lock manager        │            ║
       ║  │ manager        │  │ (physical integrity)│            ║
       ║  │ (logical       │  │                     │            ║
       ║  │  integrity)    │  └─────────────────────┘            ║
       ║  └────────────────┘                                     ║
       ║                                                         ║
       ║  ┌────────────────┐  ┌─────────────────────┐            ║
       ║  │ Access methods │  │ Buffer manager      │            ║
       ║  │ (B-trees, LSM, │  │ (page cache in RAM) │            ║
       ║  │  heap files)   │  │                     │            ║
       ║  └────────────────┘  └─────────────────────┘            ║
       ║                                                         ║
       ║  ┌──────────────────────────────────────────┐           ║
       ║  │ Recovery manager  (WAL, ARIES, redo/undo)│           ║
       ║  └──────────────────────────────────────────┘           ║
       ╚═════════════════════════════════════════════════════════╝
```

**What each box owns, and the question it answers:**

| Box | Owns | The single question it answers |
|-----|------|-------------------------------|
| Transport | TCP, TLS, wire protocol, cluster gossip | "How do bytes get in and out?" |
| Query processor | Parser, validator, AuthZ check | "Is this a legal query for this user?" |
| Optimizer | Statistics, plan search, cost model | "Of the many ways to answer this, which is cheapest?" |
| Execution engine | Operator tree iteration, distributed fan-out | "Run the plan; collect results." |
| Transaction mgr | ACID's A, C, I (the *logical* invariants) | "Will this leave the DB in a legal state?" |
| Lock mgr | Latches, row/page/table locks, deadlock detection | "Are two concurrent ops trying to do incompatible things?" |
| Access methods | B-trees, LSM-trees, heap files, indexes | "Where on disk is the row I want?" |
| Buffer mgr | Page cache, eviction policy (LRU, clock) | "Is this page already in RAM?" |
| Recovery mgr | WAL writing, checkpointing, replay | "If we crash right now, can we get back to a consistent state?" |

**The two cross-cutting concerns the diagram hides:**

The boxes are not as independent as Figure 1-1 suggests. Two concerns cut horizontally:

1. **Concurrency control** = transaction manager + lock manager **together**. Neither is sufficient alone — the txn manager defines what "consistent" means; the lock manager enforces the no-conflict invariant that lets the txn manager reach consistency. Splitting them is more a textbook convenience than an implementation reality.
2. **Durability** = buffer manager + recovery manager **together**. The buffer manager decides *when* dirty pages flush; the recovery manager makes the flush safe (write-ahead log first, page later). Splitting them is again pedagogical — in code (e.g., Postgres's `bufmgr.c` and `xlog.c`) they're tightly coupled.

These two horizontal concerns are why DBI dedicates separate later chapters to **Concurrency Control** and **Recovery** — they don't fit cleanly into a single box.

**Memory- vs. disk-based DBMS — what does NOT change in the diagram (p.33):**

The same six boxes apply to Redis and Postgres. What differs:

| Aspect | Disk-based (Postgres) | In-memory (Redis, MemSQL) |
|--------|----------------------|---------------------------|
| Buffer manager | Dominates perf; LRU/clock matters | Trivial — everything is "in cache" |
| Access methods | B-tree-on-disk; designed for sequential I/O | Hash tables, skiplists; designed for random access |
| Recovery manager | WAL is the *primary* durability mechanism | WAL + AOF snapshot + maybe battery-backed RAM |
| Transaction mgr | Same conceptually; different perf cost | Same |
| Optimizer | Critical (disk reads dominate cost) | Less critical (everything's fast) |

The memory-vs-disk distinction is about **which box bottlenecks**, not which boxes exist. Read this section as Petrov telling you: "the architecture is invariant; the chapter weights aren't."

### If you were tracing a `SELECT` query in Postgres at 3 AM, which box would you instrument first?

The textbook's answer depends on the symptom, but the most common diagnostic flow is **optimizer first, then execution engine, then buffer manager**:

1. `EXPLAIN ANALYZE` — opens the optimizer's box. If the chosen plan is wrong (wrong join order, missing index use), you fix stats (`ANALYZE`) or hints.
2. If the plan is right but slow, you look at the execution engine's operator timings — which operator (hash join, seq scan, sort) is dominating?
3. If the operator is a seq scan touching disk, you look at the buffer manager — is the working set blowing out shared_buffers? `pg_stat_io` is your read.

You almost never need to instrument the transport subsystem, query processor, or transaction manager unless a specific symptom points there (connection storms → transport; AuthZ errors → processor; deadlocks → lock manager). This **descend-the-diagram debugging discipline** is exactly what the six-box decomposition gives you.

### Cross-language view
*(n/a — architectural-decomposition entry.)*

### Where this shows up in real systems

- **Postgres source-tree mirrors Figure 1-1 almost exactly.** `src/backend/parser/` = query processor; `src/backend/optimizer/` = optimizer; `src/backend/executor/` = execution engine; `src/backend/storage/` = the storage engine sub-boxes (further split into `buffer/`, `lmgr/`, `access/`, `transam/`). Reading the directory listing is itself a tour of the diagram.
- **CockroachDB / TiDB split the diagram across nodes.** SQL nodes hold the parser, optimizer, and executor; KV nodes hold the storage engine. The horizontal cut isn't between boxes inside a single process — it's between *layers* across the network, and the transport subsystem is doing far more work than in single-node DBs. The diagram still applies; the *implementation* of the arrows between boxes is gRPC instead of function calls.
- **The "no SQL" databases didn't actually skip the diagram, they just deleted boxes.** Early MongoDB had no query optimizer worth the name (cost-based optimization arrived in 3.x); early Redis had no transaction manager (MULTI/EXEC arrived later); Cassandra has no global lock manager (deliberately — it gets concurrency from quorums, not locks). Reading a NoSQL system as "Figure 1-1 with boxes X and Y removed" tells you exactly which problems that system traded away for what gain.

### Diagnostic questions

1. **Q:** Why is the lock manager separated from the transaction manager in Figure 1-1 if they're so tightly coupled in practice?
   *Wrong-answer trap:* "Because they're different code modules." It's because they enforce **different kinds of integrity**: the txn manager enforces *logical* consistency (the DB's invariants — your constraints, your foreign keys), while the lock manager enforces *physical* integrity (no two threads write the same byte). MVCC systems (Postgres) loosen lock-manager work in favor of txn-manager work; lock-heavy systems (MySQL ISAM, pre-MVCC) do the opposite. Separating them lets the textbook discuss the tradeoff.

2. **Q:** Does an in-memory DBMS need a recovery manager?
   *Wrong-answer trap:* "No — there's no disk to recover from." It does. The recovery manager exists not to recover *the disk*, but to recover *correct state after a crash*. Redis's AOF and RDB snapshots are recovery managers; MemSQL's redo log is a recovery manager. Even a pure-RAM system needs to answer "if I lose power, do I lose any acknowledged writes?" — and the answer is "no, because of recovery manager."

3. **Q:** Where in Figure 1-1 does index maintenance happen on an `INSERT`?
   *Wrong-answer trap:* "Access methods." Half-right. The *physical* index write is access methods. But the *decision* that an index needs updating, and the *ordering* of (heap write, WAL write, index write), is split across the execution engine (decides what to write), the access methods (does the B-tree split), the recovery manager (logs intent), and the lock manager (latches the page being split). A single INSERT touches **five of the nine boxes**. The diagram's clean boundaries don't survive a single mutation.

4. **Q:** If you were writing a from-scratch DBMS and had to drop one box from Figure 1-1, which is the least painful to drop?
   *Wrong-answer trap:* "Optimizer." Optimizer-less databases (early MongoDB, some embedded KV stores) work but force users to write physically-tuned queries — a real cost, but a tolerable one. The hardest to drop is **buffer manager**; without one, every query is an `mmap`-and-pray hack, eviction is at the OS's discretion, and you lose the ability to do explicit page-pinning during a B-tree descent. LMDB famously *does* drop it (delegates to OS mmap) and the cost is that LMDB performance is exquisitely sensitive to OS page-cache behavior in ways its authors document at length.

### See also

- [dbi-notes.md](Notes/dbi-notes.md) 2026-05-12 — *OLTP, OLAP, and HTAP* — that taxonomy shapes which box dominates: OLTP stresses the transaction + lock managers, OLAP stresses the optimizer + execution engine, HTAP tries to win both.
- Hellerstein, Stonebraker & Hamilton, *Architecture of a Database System* (2007) — the canonical paper Petrov is paraphrasing; reading it alongside this chapter is a free graduate seminar.
- DDIA Ch.3 (Storage and Retrieval) — same storage-engine sub-boxes, viewed from the data-engineering angle rather than the DBMS-internals angle.
- OSTEP Ch. on file systems — the buffer manager and access methods sit on top of the OS-level page cache and disk scheduler discussed there; reading them together clarifies why DBs often *fight* the OS for cache control.

---

## [2026-05-12] OLTP, OLAP, and HTAP — A DBMS Taxonomy · pp.24–28 · Part I intro + Ch.1 Introduction

### TL;DR
Database systems split into two long-standing camps — **OLTP** (many small, latency-sensitive transactions) and **OLAP** (few large, throughput-sensitive analytical queries) — because the storage layouts that win one workload lose the other (row-oriented vs column-oriented; B-trees vs LSM-trees). **HTAP** ("hybrid") is the post-2014 attempt to serve both from one system, usually by keeping two physical copies of the data internally and routing queries to the right one. The taxonomy isn't just trivia: every later chapter of DBI is implicitly answering "are we optimizing for OLTP, OLAP, or trying to do both?"

### History — "why does this exist?"
The split is older than the names. **Codd's 1970 relational model** assumed one database serving both transactions and reports, and through the 1980s mainframe DB2 / Oracle did exactly that — badly, because analytic queries locked tables and starved transactions. **Bill Inmon (1990) coined "data warehouse"** and made the case that analytics deserved its own physical store fed by nightly ETL — that's the OLTP/OLAP separation we still live with. The **C-Store paper (Stonebraker et al., 2005)** crystallized OLAP's preferred shape: columns, not rows; this became Vertica commercially and inspired Redshift, BigQuery, Snowflake, ClickHouse. **HTAP as a term (Gartner, 2014)** named the obvious next move: ETL pipelines are slow, expensive, and stale, so why not run both workloads against one logical database? The 2010s answer was "because the storage layouts are incompatible"; the 2020s answer is "we'll keep both layouts" — TiDB, SingleStore (formerly MemSQL), Snowflake Unistore, and PlanetScale's vitess-based analytics tier all do exactly that.

### Intuition — "this is like…"
**OLTP is a 7-Eleven; OLAP is a Costco; HTAP is the dream of one store doing both.** A 7-Eleven optimizes for many small, fast checkouts (you buy two items in 30 seconds, hundreds of times an hour). A Costco optimizes for occasional huge cart-fulls (one customer, 80 items, 20 minutes at checkout). Build a single store that does both well, and you have to physically split the space — pallet aisles in one half, grab-and-go shelves in the other. That's exactly what HTAP databases do internally: two storage layouts, one query layer.

### Mechanics

**The three categories DBI introduces:**

| Category | Workload shape | Storage preference | Examples |
|---|---|---|---|
| **OLTP** (transaction processing) | Many concurrent users; small reads/writes of single rows; tight latency budgets (ms); predefined queries | **Row-oriented** B-trees or LSM-trees; rows colocated for fast single-row access | PostgreSQL, MySQL, Oracle, CockroachDB, Spanner |
| **OLAP** (analytical processing) | Few users; large scans over millions of rows; throughput over latency; ad hoc SQL | **Column-oriented**; columns compressed; only touched columns read | BigQuery, Snowflake, Redshift, ClickHouse, DuckDB |
| **HTAP** (hybrid) | Both, sometimes on the same data within seconds | Two layouts internally: row store for writes, column replica for reads | TiDB, SingleStore, Snowflake Unistore, MongoDB Atlas (Search/Analytics nodes) |

**Why one storage layout can't win both:**

```
Row-oriented layout (good for OLTP):
   Disk block:  [id=1│name="Ada"│age=30│city="London"]
                [id=2│name="Bob"│age=25│city="Paris" ]
                [id=3│name="Cy" │age=40│city="Tokyo" ]
   ─ Fetching one user: 1 random I/O, all columns adjacent. ✔
   ─ "What is the average age across 100M rows?":
        Must read every block (every byte of every row) to touch one column. ✘

Column-oriented layout (good for OLAP):
   id   block:  [1, 2, 3, …]
   name block:  ["Ada", "Bob", "Cy", …]
   age  block:  [30, 25, 40, …]    ← compress this beautifully (run-length, delta)
   city block:  ["London", "Paris", "Tokyo", …]
   ─ "Average age": read only the age column. 5–50× less I/O. ✔
   ─ Fetching one full user row: 4 separate column blocks; random I/O on each. ✘
```

The 5–50× ratio is the *core economic argument* for column stores on analytics, and it's why the OLTP/OLAP split survived for 30 years — no amount of cleverness in indexing makes row stores fast at column-aggregate workloads.

**How HTAP cheats:**

```
                 ┌──────────────┐
   Writes ─────► │  Row store   │  (transactional, freshly written)
                 │  (e.g.       │
                 │   in-memory) │
                 └──────┬───────┘
                        │ async (or sync, in TiDB's TiFlash)
                        ▼
                 ┌──────────────┐
   OLAP ◄─────── │ Column store │  (analytical, columnar, compressed)
   queries       │  replica     │
                 └──────────────┘

   The query planner picks: row store for point reads + writes,
   column store for scans + aggregations.
```

The trick is that **the column replica isn't a separate database** — it's maintained by the same system, with the same SQL surface, and stays consistent with the row store within milliseconds to seconds (depending on configuration). The cost is roughly 2× storage and a non-trivial replication path; the win is no ETL, no staleness, no separate analytics warehouse.

**SLA — the term DBI introduces in this chunk (p.26 footnote).** A *service-level agreement* names quantitative commitments: latency (P50/P99/P99.9), throughput (queries/second), jitter (latency variance), failure rate. The OLTP vs OLAP split is largely an SLA split — OLTP SLAs are tight on P99 latency (10–100 ms); OLAP SLAs accept seconds-to-minutes per query but measure throughput in TB/hour scanned.

### If you were the storage-engine architect…

You can pick **one** primary representation. Which side do you give up?

- **Row-first (PostgreSQL's choice):** Optimize for OLTP, accept that analytics will need a replica or extension. Postgres later added columnar extensions (Citus, Hydra) but its core remains row-oriented.
- **Column-first (Snowflake's choice):** Optimize for OLAP, accept that single-row mutations are slow. Snowflake added Unistore (row-oriented hybrid tables) in 2022 precisely because customers wanted *some* OLTP without buying a second database.
- **Both-first (TiDB's choice):** Build two physical engines (TiKV row store + TiFlash column store) under one SQL layer. You pay 2× storage and a complex replication path, but you get one SQL endpoint for everything.

The right answer depends on your **bias hypothesis**: do you believe customers' OLTP workloads will eventually need analytics (so HTAP wins), or that the two workloads are too different to share one engine well (so specialized systems win)? The 2020s industry verdict is partially HTAP, partially federated query engines (Trino, DuckDB-against-Parquet), but the OLTP/OLAP separation is far from dead.

### Cross-language view
*(n/a — this is a system-architecture taxonomy entry. Cross-language treatment of the underlying storage structures — B-trees, LSM-trees — will appear in later DBI entries.)*

### Where this shows up in real systems

- **The "modern data stack" is the OLTP/OLAP split made into a vendor matrix.** Production runs on Postgres/MySQL (OLTP); CDC pipes data into Snowflake/BigQuery (OLAP); dbt transforms it; Looker/Tableau queries it. The whole stack exists *because* OLTP and OLAP couldn't share one engine for 30 years. HTAP systems are an explicit bet that this stack can collapse.
- **Real-time analytics services.** ClickHouse-as-a-service, Pinot at LinkedIn, Druid at Netflix — all built because OLAP latency requirements moved from "nightly batch" to "sub-second on streaming data." None of them are HTAP; they're specialized OLAP-with-low-latency, fed by separate OLTP systems.
- **Vector databases (Pinecone, Weaviate, pgvector) are a fourth category** the DBI taxonomy doesn't capture, but every choice in their design (HNSW indexes, quantization) is recognizable as a workload-specific storage trade-off — i.e. the same kind of decision DBI is teaching, applied to a workload the book predates.

### Diagnostic questions

1. **Q:** Why isn't "just add an index" enough to make a row store fast at analytics?
   *Wrong-answer trap:* "Indexes don't cover all columns." Even a covering index over (say) `(age, city)` only helps queries on those columns; the moment a query asks `SELECT AVG(salary)` you still scan rows. Columnar storage is fast because *every column is its own optimally-laid-out structure*, not because of indexing.

2. **Q:** Why does HTAP require ~2× storage instead of just being a smart query planner over one layout?
   *Wrong-answer trap:* "Because the engineers were lazy." It's information-theoretic: row layout and column layout encode the same data with *different access optimizations*. You can convert at query time, but the conversion cost (rearranging GB of data) defeats the latency win you're paying for. Materializing both layouts is the cheapest correct answer.

3. **Q:** A startup CTO says "we'll use Postgres for everything, no separate warehouse." When does that break?
   *Wrong-answer trap:* "At ~1TB of data." It breaks at the *query shape* boundary, not the size boundary — the day your dashboards run a 30-second analytical query that locks rows the OLTP path needs. Many teams hit this around 100GB, others survive at 5TB. The trigger is the analytical workload, not the row count.

4. **Q:** Why is "HTAP" sometimes criticized as a marketing term?
   *Wrong-answer trap:* "Because no system does both well." The sharper critique: every "HTAP" system internally has two physical stores — so an HTAP database is structurally an OLTP database + an OLAP database + a query router. The term obscures that the underlying separation persists; what HTAP buys is *operational simplicity* (one vendor, one SQL surface), not a fundamental new layout.

### See also

- DDIA Ch.3 (Storage and Retrieval) is the *long* version of this same taxonomy — Kleppmann walks through B-trees, LSM-trees, and column stores with worked examples; DBI is the deeper specialist version of the same material.
- DDIA Ch.10 (Batch Processing) and Ch.11 (Stream Processing) extend OLAP from "warehouse queries" to "data pipelines" — Spark/Flink are OLAP-style engines for derived data.
- COD Ch.5 (Memory Hierarchy) — the column-store performance argument is fundamentally a *cache locality* argument: column blocks compress better and stream through cache predictably, where row blocks waste cache lines on columns you didn't ask for.
- DSG Ch.6 (later) on log-structured storage — the LSM-tree machinery that powers OLTP write throughput is conceptually similar to the Kafka-style logs DSG builds.

---

## [2026-05-11] The Storage Engine: A Database's Pluggable Core · pp.19–23 · Part I · Storage Engines

### TL;DR
A database management system (DBMS) is a layered architecture; the **storage engine** is the bottom layer — the component responsible for persisting bytes to disk and serving them back, exposing only a minimal key/value API of `get`/`put`/`delete`/`scan`. Above it sit query processing, execution, transport. The crucial design insight is that storage engines are *separable* from the rest of the DBMS: MySQL alone supports InnoDB, MyISAM, and RocksDB; MongoDB supports WiredTiger, In-Memory, and MMAPv1. This decoupling lets database vendors compose new systems quickly and lets operators pick the engine whose performance profile fits their workload.

### History — "why does this exist?"
The idea of a pluggable storage layer goes back at least to **Berkeley DB (1991, Margo Seltzer et al. at Berkeley)** — a key/value embedded library that became the storage substrate of many higher-level systems (OpenLDAP, early Subversion, the original Bitcoin client). Google's **LevelDB (Jeff Dean & Sanjay Ghemawat, 2011)** and Facebook's **RocksDB (2013)** popularized log-structured merge-tree engines as a reusable component, and they now power most modern systems' write paths (CockroachDB, TiKV, Kafka Streams' state stores, Ceph's BlueStore). The split also runs the other direction: when **MySQL 5.1 (2008)** introduced pluggable storage engines as a first-class API, it institutionalized the idea that the DBMS-above-the-engine and the engine itself are separable products with separable release cycles — which is why Percona, MariaDB, and MyRocks could exist as variants of the same nominal "MySQL." Reed's 1978 thesis (cited in the chapter as **[REED78]**) on naming and protection in distributed systems provided the conceptual frame: persistent state is a *protected resource* with a uniform access protocol.

### Intuition — "this is like…"
Think of a restaurant. The **storage engine** is the walk-in cooler and the pantry — it knows how to put food in a labeled location, retrieve it intact, and tell you how much is left. The **query processor** is the head chef, who decides which ingredients to combine and in what order. The **execution engine** is the line cook, who actually moves food from cooler to pan. The **transport layer** is the waiter, taking orders and delivering plates. The genius of the modern restaurant — and the modern database — is that you can swap the cooler (LevelDB → RocksDB) without retraining the chef, the cooks, or the waiters. As long as the cooler responds to "give me 2 kg of carrots" in the agreed protocol, the rest of the restaurant doesn't care whether it's a walk-in fridge or a hydroponic farm in the basement.

### Mechanics

**The four-layer DBMS architecture (textbook decomposition):**

```
       Client app
           │
   ┌───────▼────────┐
   │ Transport      │  ← TCP/TLS, wire protocol (Postgres FE/BE, MySQL X protocol, …)
   ├────────────────┤
   │ Query          │  ← SQL parser, planner, optimizer
   │  Processor     │
   ├────────────────┤
   │ Execution      │  ← physical operators: scans, joins, aggregates
   │  Engine        │
   ├────────────────┤
   │ Storage        │  ← key/value primitives, page cache, WAL, compaction
   │  Engine        │  ← THIS is what Part I of DBI is about
   ├────────────────┤
   │ Disk / NVMe / S3 │
   └────────────────┘
```

**The storage engine's contract — a minimal API.** From the chapter:

```
storage.put(key: bytes, value: bytes)        // create or update
storage.get(key: bytes) -> bytes | None      // point lookup
storage.delete(key: bytes)
storage.scan(start: bytes, end: bytes) -> iterator<(bytes, bytes)>  // range
```

Notice keys and values are **opaque byte sequences** — the storage engine doesn't know `int32` from `string`. Type interpretation, sort orders for strings vs numbers, indexing strategies — all of that lives above the storage engine, in the query/execution layers.

**Two big families of storage engines (the chapter's organizing axis for later sections):**

| Family | Examples | Strength | Weakness |
|--------|----------|----------|----------|
| **B-tree based** | InnoDB, WiredTiger, LMDB | Fast point reads, in-place updates | Write amplification at high write rates |
| **LSM-tree based** | LevelDB, RocksDB, HBase, Cassandra | Sequential write throughput | Compaction overhead, read amplification |

**Pluggability in practice — same engine, different DBMSes:**

```
                   RocksDB (LSM-tree storage engine)
                   ┌──────┬─────────┬──────────┬───────────┐
                   ▼      ▼         ▼          ▼           ▼
                MyRocks  TiKV   CockroachDB  Kafka      Yugabyte
                (MySQL)  (KV)    (NewSQL)    Streams    (Postgres-compat)
```

One LSM-tree implementation, five very different products. This is why understanding the storage layer pays off disproportionately: it's where the *real* performance budget lives, and the same engine appears under many brand names.

**Comparing databases — the chapter's process:** Don't compare on (a) which engine is used, (b) DB-Engines rank, or (c) implementation language. Instead:

1. Define the workload: schema, record sizes, client count, query mix, read/write ratio.
2. Run a long, realistic stress test on a production-shaped cluster.
3. Watch for problems that appear only at scale or after time (compaction stalls, GC pauses, replica drift).
4. Use **YCSB (Yahoo! Cloud Serving Benchmark)** and **TPC-C** as starting points — but treat them as scaffolds, not verdicts.

**TPC-C in one paragraph.** The Transaction Processing Performance Council's OLTP benchmark — concurrent transactions over a warehouses/stock/customers/orders schema, measured in transactions-per-minute (tpmC). Transactions must satisfy ACID and conform to TPC-C's specific transaction mix. It's the canonical OLTP benchmark and has been the public scoreboard for relational systems since 1992. Modern systems often report ~hundreds of thousands to millions of tpmC; the absolute number is less interesting than the trend across vendor releases.

### If you were the database architect…

You're building a new system that needs to store ~1 TB of frequently-updated user records with low-latency point reads. Do you write your own storage engine?

**No** — the textbook's argument is structural. The storage engine is the most heavily-engineered layer of any DBMS: page cache strategy, write-ahead log, crash recovery, compaction, checksumming, bloom filters, block compression. RocksDB has ~400k LOC and 15+ years of production hardening; reproducing it would take a small team 3+ years and would never reach feature parity. **Pick an existing storage engine — RocksDB for write-heavy, LMDB for read-heavy, SQLite for embedded — and spend your engineering budget on the layers above** (query language, distribution, replication). The history of databases since 2010 is the story of teams that took this advice (TiKV, CockroachDB, FoundationDB-on-SQLite-style design) outpacing teams that didn't.

The exception that proves the rule: **FoundationDB** wrote their own storage engine (Redwood) — but only after running on SQLite for years and only when they hit specific deterministic-simulation requirements that no existing engine met.

### Cross-language view
*(n/a — storage-engine concepts manifest at the architecture level, not the language level. Implementation languages vary: RocksDB is C++, LMDB is C, BoltDB/badger are Go, Sled is Rust. Each language's RAII/GC story shapes resource management — e.g., LMDB's mmap model is natural in C, awkward in GC languages where the buffer's lifetime is unclear.)*

### Where this shows up in real systems

- **MyRocks at Facebook (2017).** Facebook replaced InnoDB with RocksDB under MySQL for their UDB (user database) tier, halving their flash footprint by exploiting LSM compression. The wire protocol stayed MySQL; only the storage engine changed. This is the cleanest possible demonstration of the textbook's "engines are pluggable" claim — Facebook's application code didn't move.
- **TiKV → TiDB.** TiKV (the storage layer) is a distributed RocksDB; TiDB (the SQL layer) is a separate process that speaks MySQL wire protocol on top. PingCAP can — and does — sell TiKV by itself as a key/value store, *and* TiDB as a relational database. Same engine, two products.
- **Kafka's "log as the storage engine."** Kafka famously chose a simpler-than-LSM design — an immutable append-only log — as its storage engine. The trade-off: no point lookups, only sequential reads from offsets. But because the storage primitive is so simple, Kafka can do *millions* of writes/sec where a B-tree-based engine couldn't. The lesson generalizes: when the storage engine matches the workload, the upper layers get simpler too.
- **S3 as the new "storage engine."** A 2024–2026 trend: systems like SlateDB, WarpStream, and Tigris are building storage engines whose persistent layer is S3 (or any object store) rather than local disk. The DBMS API stays the same; the storage engine is now distributed, durable-by-default, and infinitely scalable, at the cost of ~50ms-per-op latency floors. This is the *next* round of "engines are pluggable," played at cloud scale.

### Diagnostic questions

1. **Q:** Why is the storage engine's API just `get`/`put`/`delete`/`scan` — no `JOIN`, no `SELECT WHERE`, no transactions?
   *Wrong-answer trap:* "Because the storage engine is dumb." It's deliberately narrow: a narrow API means many implementations can satisfy it, which is what makes pluggability possible. Any richer API would couple the storage engine to assumptions about schema and query language, which would break the abstraction.

2. **Q:** Why is "which storage engine" a *bad* dimension for comparing databases?
   *Wrong-answer trap:* "Because it doesn't matter." It matters enormously for performance, but it tells you nothing about correctness, query expressivity, distribution model, or operability. Two systems on the same engine (MyRocks and TiKV both use RocksDB) have wildly different operational profiles.

3. **Q:** If keys and values are opaque bytes, how does the database know to sort `int32(10)` before `int32(100)` and not lexicographically?
   *Wrong-answer trap:* "The storage engine handles type-aware sort." It doesn't — the query/execution layer above serializes numbers in a sort-preserving encoding (e.g., big-endian for unsigned ints, sign-flipped MSB for signed) before handing the bytes to the storage engine. The engine then sorts byte-lexicographically and the higher layer gets the right order back.

4. **Q:** TPC-C measures transactions-per-minute. Why is that better than measuring queries-per-second?
   *Wrong-answer trap:* "Minutes are more convenient." A transaction is a unit of *correctness* (ACID-bounded), not just a unit of work. Measuring tpmC counts the work the system did *while preserving guarantees*; QPS can be gamed by relaxing isolation. The benchmark deliberately ties throughput to correctness.

5. **Q:** Why does the chapter warn against benchmarks on dimensions like implementation language?
   *Wrong-answer trap:* "Languages don't matter for performance." They matter — Rust avoids GC pauses, C++ permits zero-copy paths — but they're a leading indicator at best. A well-engineered Java DBMS will outperform a poorly-engineered Rust one for almost any realistic workload. Language is a coarse proxy for engineering quality, not a substitute for measuring it.

### See also

- [ddia-notes.md](Notes/ddia-notes.md) — Chapter 3 (Storage and Retrieval) covers exactly the B-tree vs LSM-tree split this chapter previews; read DBI's later chapters as the *implementation* of DDIA Ch.3's *concepts*.
- [ostep-notes.md](Notes/ostep-notes.md) — Persistence chapters (file systems, journaling) explain the layer *below* the storage engine: how the OS guarantees the bytes the engine writes actually survive a crash.
- [dsg-notes.md](Notes/dsg-notes.md) — Distributed Services with Go builds a commit-log–based service that is itself a kind of degenerate storage engine; the design choices map onto this taxonomy.
- [sahp-notes.md](Notes/sahp-notes.md) — Software Architecture: The Hard Parts has the macro version of this argument: every layered architecture lives or dies by whether the contracts between layers stay narrow enough to remain pluggable.

---
