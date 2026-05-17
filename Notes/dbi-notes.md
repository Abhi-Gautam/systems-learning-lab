# DBI Notes

_Entries follow the template at `Notes/TEMPLATE.md`. Append-only. **Newest entry at top**, immediately after this header._

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
