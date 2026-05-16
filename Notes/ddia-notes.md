# DDIA Notes

_Entries follow the template at `Notes/TEMPLATE.md`. Append-only. **Newest entry at top**, immediately after this header._

---

## [2026-05-19] Relational vs Document vs Network — The Three Data Models, Their History, and Why The Debate Keeps Repeating · pp.49–62 · Ch.2 Opening → Relational vs Document Model → Many-to-Many → Repeating History → Schema-on-Read vs Schema-on-Write

### TL;DR
This chunk is **Chapter 2's opening salvo** and the most historically informed argument in the book. Kleppmann's load-bearing claim is that the data model is *the* design decision with the deepest reach — it shapes not just how software is written but **how programmers think about the problem** (the Wittgenstein epigraph is not decorative). The chapter then walks the **relational/document/CODASYL triangle** three times: (1) as a present-day choice between SQL and MongoDB-style stores, framed around the **object-relational impedance mismatch**; (2) as a historical recurrence — document databases are, structurally, *the hierarchical IMS model of 1968 returning in JSON clothing*, with the same many-to-many limitations CODASYL tried (and failed) to solve in the 1970s; (3) as the **schema-on-write vs schema-on-read** dichotomy, the dynamic-vs-static-typing debate ported into databases. The deepest insight: **the question is not "which data model is better" but "which kind of relationships dominate your data."** Tree-shaped data (a résumé, a product page) wants documents; web-of-references data (a social graph, a recommendation network, anything that *grows new associations as features are added*) wants relational joins or graphs; analytics-style append-only event streams want neither. By the end of the chunk you should be able to look at any feature request and predict, before writing a line of code, whether the data model will start fighting you in six months.

### History — "why does this exist?"
The chapter is a **layered history of database thinking** and is worth treating as such because the layers are still load-bearing in 2026 architecture decisions. The **hierarchical model** was crystallized in **IBM's IMS (1968)**, originally written by IBM and Rockwell for tracking parts in the **Apollo space program** — the same project that demanded Margaret Hamilton's flight software. IMS is still in production on z/OS mainframes in 2026, running the back ends of major US banks and airlines. The **CODASYL network model** (Conference on Data Systems Languages, 1969 spec via the DBTG report) generalized IMS to allow multiple parents per record; **Charles Bachman won the 1973 Turing Award** for it. The **relational model** is **E. F. Codd's 1970 CACM paper** "A Relational Model of Data for Large Shared Data Banks" — itself a reaction to CODASYL's "navigation-by-pointer" complexity. Codd won the **1981 Turing Award**; his model dominated by the mid-1980s with **System R (IBM)** and **Ingres (Berkeley)** as the academic prototypes and **Oracle (1979)**, **DB2 (1983)**, **Sybase/MS SQL (1987)** as the commercial scaffolding. Object databases — **GemStone (1986), ObjectStore (1988), Versant (1988)** — were the *first* "the relational model is wrong" wave, peaking around 1991 and collapsing by 1995 because the impedance mismatch they solved was less painful than the loss of ad-hoc queries. XML databases were the *second* wave (around 2001, with eXist-db and MarkLogic), niche by 2005. **NoSQL** is the *third* wave: the name comes from a **2009 meetup hashtag** organized by Johan Oskarsson in San Francisco to discuss open-source distributed nonrelational stores; it was retroactively softened to "Not Only SQL." The major NoSQL families that emerged were document (MongoDB 2009, CouchDB 2005), column-family (Bigtable 2006, Cassandra 2008, HBase 2008), key-value (DynamoDB 2007 internally, Redis 2009, Riak 2009), and graph (Neo4j 2007). The **impedance mismatch** term is borrowed from electronics (literally maximum-power-transfer theorem) and was popularized for OO/DB by **Scott Ambler's writing in the late 1990s**. The chapter is being written in 2015–2017; by 2026 the **polyglot persistence** prediction Kleppmann hedges on (p.50) has fully materialized — every nontrivial system uses 3–5 storage technologies.

### Intuition — "this is like…"
Choosing a data model is like **choosing the shape of the filing cabinet for a business you do not yet fully understand**. A *document* database is a stack of folders, each folder self-contained: pulling one folder gets you the whole person/order/product in one motion (great **locality**), but if two folders need to point at the same entity ("Joe's home address"), you either duplicate the address into every folder (a nightmare when Joe moves) or invent a cross-reference system that the cabinet does not natively understand. A *relational* database is a wall of indexed drawers, each drawer holding one *kind* of fact (people, addresses, orders, line items); answering a question typically means opening several drawers and matching cards together (a **join**) — slower per question, but the cabinet itself takes responsibility for keeping cross-references consistent. A *CODASYL* database is a maze of folders connected by literal physical strings: every fact has a strict navigation path from the entrance of the building, and if you forget the path you cannot find your data. The relational model "won" not because joins were faster but because **the burden of remembering paths moved from the programmer to the query optimizer**, freeing programmers to ask new questions without rewriting old code.

### Mechanics

**1. The four-layer abstraction stack (pp.49–50) — the chapter's framing diagram.** Every running application is **four data-model layers** stacked on top of each other, and the chapter is explicitly about layer 2:

```
   ┌───────────────────────────────────────────┐
   │  L1 — App developer's mental model        │   "people, orders, money flows"
   │       (domain objects, business rules)    │
   ├───────────────────────────────────────────┤
   │  L2 — General-purpose data model          │   ← this chapter
   │       (relational tables, JSON docs,      │
   │        graph triples)                     │
   ├───────────────────────────────────────────┤
   │  L3 — Storage-engine representation       │   ← Chapter 3
   │       (B-trees, LSM-trees, heap files,    │
   │        column stores)                     │
   ├───────────────────────────────────────────┤
   │  L4 — Bytes on physical media             │
   │       (NAND flash blocks, magnetic        │
   │        domains, optical pulses)           │
   └───────────────────────────────────────────┘
```

**Why this stack matters:** every layer **forces vocabulary on the layer above**. Choose JSON at L2 and your L1 vocabulary will drift toward nested objects; choose tables at L2 and L1 will drift toward normalized entities. The chapter's deeper claim, anchored in the Wittgenstein quote, is that **L2 doesn't just store your model — it shapes your model**.

**2. The Object-Relational Impedance Mismatch (pp.51–53).** The single most quoted phrase in the chapter and the genuine pain that drove the NoSQL movement. The mismatch is concrete and worth seeing as a diagram:

```
   APPLICATION SIDE                          DATABASE SIDE
   ────────────────                          ─────────────
   class User {                              users table
     String firstName                          ┌────┬───────────┬──────────┐
     String lastName                           │ id │ firstName │ lastName │
     List<Position> positions  ──┐             ├────┼───────────┼──────────┤
     List<Education> education ──┤             │  1 │  Bill     │  Gates   │
     Map<String, String>         │             └────┴───────────┴──────────┘
       contactInfo               │
   }                             │           positions table
                                 ├──FK──────►┌────┬─────────┬───────────────┐
                                 │           │ id │ user_id │ job_title     │
                                 │           ├────┼─────────┼───────────────┤
                                 │           │ 12 │   1     │ Co-founder    │
                                 │           │ 13 │   1     │ Co-chair      │
                                 │           └────┴─────────┴───────────────┘
                                 │
                                 │           education table  (similar shape)
                                 └──FK──────►(...)

   Loading one User =  4 separate queries OR 1 messy 4-way join
   Saving one User  =  4 inserts (or 1 INSERT with returning + cascades)
   Schema migration =  ALTER TABLE × N tables
```

ORM frameworks — **Hibernate (Java, 2001), ActiveRecord (Rails, 2005), SQLAlchemy (Python, 2006), Entity Framework (.NET, 2008), GORM (Go), Diesel (Rust)** — exist *entirely* to manage this mismatch and **never fully hide it**. The "N+1 query problem" everyone discovers their first year is the impedance mismatch leaking through Hibernate. **JSON documents collapse the mismatch by storing the tree directly**:

```json
{ "user_id": 251, "first_name": "Bill", "last_name": "Gates",
  "positions":    [ {"job_title": "Co-chair", "organization": "Bill & Melinda Gates Foundation"},
                    {"job_title": "Co-founder, Chairman", "organization": "Microsoft"} ],
  "education":    [ {"school_name": "Harvard University", "start": 1973, "end": 1975} ],
  "contact_info": { "blog": "http://thegatesnotes.com", "twitter": "..." } }
```

One document = one user = one read. **The locality win is real**: a profile fetch is a single B-tree lookup on `_id` instead of a 4-way join with 4 random I/Os.

**3. The four NoSQL drivers (p.51).** Kleppmann names them flatly, and they remain the four reasons people leave PostgreSQL in 2026:

| Driver | Concrete symptom | Where it leads |
|---|---|---|
| **Scalability** beyond a single node | "we sharded MySQL by hand and can't add features" | Cassandra, Dynamo, Bigtable family |
| **Open-source preference** | "we don't want to pay Oracle $40K/CPU" | Postgres, MySQL, then any open-source NoSQL |
| **Specialized queries** | "our queries are graph traversals / full-text / time-series" | Neo4j, Elasticsearch, InfluxDB |
| **Schema rigidity friction** | "ALTER TABLE on a 500M-row table is a 4-hour outage" | MongoDB, schema-on-read JSON |

**Polyglot persistence** (Fowler's term, 2011, which Kleppmann nods to on p.51) is now the *default* — a typical 2026 microservices stack has Postgres (transactional), Redis (cache + ephemeral), Kafka (event log), Elasticsearch (search), ClickHouse or Snowflake (analytics), and S3 + Parquet (cold archive). **The relational database did not lose; it stopped being the only thing in the room.**

**4. One-to-many vs many-to-one vs many-to-many — the chapter's central distinction (pp.54–57).** This is where the chapter's argument turns from history to engineering. The résumé example walks the staircase deliberately:

| Relationship | What it looks like | Document model | Relational model |
|---|---|---|---|
| **One-to-many tree** (user → positions, education) | Strict containment hierarchy | ✓ **Natural** — nested arrays in the same doc | ⚠️ Requires extra tables + FK + joins |
| **Many-to-one** (user → region_id, user → industry_id) | Many records reference one canonical value | ⚠️ Awkward — either denormalize the string or simulate joins in app code | ✓ **Natural** — FK is the whole point |
| **Many-to-many** (recommendations between users; companies as entities) | Web of cross-references | ❌ **Painful** — app-side join, document-reference resolution, denormalization drift | ✓ **Natural** — join tables, queries optimized by planner |

**The chapter's predictive claim (p.55):** "data has a tendency of becoming more interconnected as features are added." You ship v1 with a flat document; v2 adds "users can recommend other users"; v3 adds "companies have their own pages and posts"; v4 adds "users can endorse skills on each other's profiles." Each feature drags one more many-to-many relationship into the model. **The document model gets uglier with every feature in this trajectory; the relational model stays roughly the same shape.**

**The normalization argument (p.55, ii. footnote):** the working definition Kleppmann gives is *"if you're duplicating values that could be stored in just one place, the schema is not normalized."* That definition skips the formal 1NF/2NF/3NF/BCNF ladder (which he flatly calls "of little practical interest"); the test you actually run is *"if this string changes, how many rows must I update?"* — answer >1 means you have a normalization problem and an inconsistency time bomb.

**5. The repeating-history argument (pp.58–59).** Document databases are **structurally the IMS hierarchical model returning in JSON clothing**. Both:
- represent data as **nested records inside parent records**
- are **excellent for one-to-many trees** (which is why both work great for "load a profile / load a product / load an order")
- are **bad at many-to-many** (which is why both force the application to either denormalize or do app-side joins)

The 1970s solved this with two competing answers. **CODASYL/network** said: let records have multiple parents, and let programmers traverse "access paths" by pointer-chasing. The access path was *manually chosen by the application programmer* — fast on 1970s tape drives but **brittle when the data model needed to change**. If your access path didn't fit a new query, you rewrote the query code by hand.

**The relational model's masterstroke (pp.59–60):** *flat tables + query optimizer + indexes*. The optimizer became the **single, reusable, amortizable** piece of infrastructure that decided access paths automatically. Codd's deep insight was an *economics* insight: you write the optimizer once, and every application that uses the database benefits forever. Hand-coded access paths win for one query; a general optimizer wins for *all queries over time*. **This is the same logic that justifies operating systems, compilers, and the JVM JIT** — invest in a general-purpose layer, and an entire ecosystem above it gets faster cheaper.

Modern Postgres's query planner is the *direct descendant of System R's optimizer (1974, Selinger et al., the "Selinger optimizer" paper)* — same cost-based join-ordering, same index selection logic, 50 years of refinement. When you write `SELECT * FROM users JOIN positions USING (user_id) WHERE region_id = 5`, the planner is doing Selinger's algorithm with modern statistics. You are riding on the back of a 50-year investment.

**Why CODASYL lost and document DBs (so far) haven't (p.60):** CODASYL's access paths were *required* — you couldn't avoid the path-traversal mental model. Document DBs' references are *optional* — most uses are tree-shaped and don't need cross-references, and for those that do, the resolution happens at *read time* via a follow-up query, not at *insert time* via a path commitment. The reversibility difference is decisive.

**6. The "which model leads to simpler app code" verdict (pp.60–61).** Kleppmann is unusually direct here:

- **Document-shaped data** (a tree of one-to-many loaded as a unit) → **document model wins**. Shredding it into 5 tables is *cumbersome schemas + complicated app code*.
- **Highly interconnected data** (web of many-to-many) → **document model becomes painful**, relational is *acceptable*, **graph models are most natural**.
- **In between** → it depends on how the relationships evolve over time. Bet on growth toward interconnection.

The honest tool for highly-interconnected data is **graph**: Neo4j, JanusGraph, Amazon Neptune, or the increasingly popular SQL extension via **CTEs + recursive queries** in Postgres / SQL Server. The chapter previews graph models for later (§"Graph-Like Data Models" on p.49 in the book's pagination).

**7. Schema-on-read vs schema-on-write (pp.61–62) — the typing debate in data clothes.** This is the chapter's most operationally useful distinction. The mapping is exact:

| Database concept | Programming-language analogue |
|---|---|
| **Schema-on-write** (RDBMS) — the DB enforces structure at INSERT/UPDATE | **Static typing** — compiler enforces structure at compile time |
| **Schema-on-read** (Mongo, dynamic JSON columns) — the *reader* interprets structure | **Dynamic typing** — runtime checks structure at use time |

The "currently full_name, want to split into first_name + last_name" example (p.62) is the cleanest comparison:

```js
// Schema-on-read (Mongo): no migration. Reader handles old + new shapes.
if (user && user.name && !user.first_name) {
  user.first_name = user.name.split(" ")[0];
}
```

```sql
-- Schema-on-write (Postgres): explicit migration.
ALTER TABLE users ADD COLUMN first_name text;
UPDATE users SET first_name = split_part(name, ' ', 1);  -- locks rows
```

**The trade-off Kleppmann names (p.62):** `ALTER TABLE` is fast on most modern relational DBs (Postgres rewrites in milliseconds for the metadata-only path); **`UPDATE` on millions of rows is slow on any DB**, and that is the actual cost. Document DBs avoid the up-front migration cost by **paying it on every read forever** in the form of branching code. If your writes outnumber your reads (rare) the trade is good; if your reads outnumber your writes (almost always) the trade compounds badly over time — you accumulate `if old_format` branches that nobody dares delete.

**Schema-on-read shines when:** the data really *is* heterogeneous (many different object types, external data you don't control). Schema-on-write shines when: the data shape is stable, the team is large enough that humans cannot remember the implicit shape, or compliance demands a verifiable schema.

**The pragmatic 2026 answer:** Postgres now has both — `JSONB` columns let you mix schema-on-write columns with schema-on-read JSON in the same table, with B-tree and GIN indexing over JSON keys. This is the **dominant pattern in modern OLTP**: structured shape for the load-bearing fields, JSONB for the long tail. MongoDB has moved the other direction, adding optional JSON Schema validation in 3.6 (2017) and richer validation in 5.0+. The two camps are **converging in the middle**, which is the boring true ending of the debate.

### If you were the architect designing a new product…
Kleppmann's three-question screen, made explicit. **(1) What is the shape of your domain's most common entity load?** If users typically load a self-contained tree (a profile, a product, an order with line items, a document), favor the document model — the locality + simplicity wins are real. **(2) What other entities will need to *refer back* to this entity?** If the answer is "many, and we don't yet know which" — comments referencing users, recommendations between users, events tagged with arbitrary objects — then you have many-to-many in your future and the relational model's optimizer-driven join will save you. **(3) How will the data shape evolve?** If you genuinely expect heterogeneity (many object subtypes, external schemas you don't control), schema-on-read is the realistic answer; if the shape is stable and the team is large, schema-on-write keeps every contributor honest. **In doubt, choose Postgres with JSONB** — it's the rare technology where the boring default is also the sophisticated answer, because you keep both regimes at once and can migrate either direction without rewriting the data.

### Cross-language view — ORM and document-store idioms

```python
# Python — SQLAlchemy (relational)
class User(Base):
    __tablename__ = 'users'
    id           = Column(Integer, primary_key=True)
    first_name   = Column(String, nullable=False)
    region_id    = Column(Integer, ForeignKey('regions.id'))   # many-to-one
    positions    = relationship('Position', back_populates='user')  # one-to-many

# Python — PyMongo (document)
db.users.insert_one({
    "_id": 251,
    "first_name": "Bill",
    "region_id": "us:91",
    "positions": [{"job_title": "Co-chair", "org": "Gates Foundation"}],
})
```

```go
// Go — sqlx (relational, struct tags map columns)
type User struct {
    ID         int    `db:"id"`
    FirstName  string `db:"first_name"`
    RegionID   int    `db:"region_id"`
}
// Positions loaded via a second query or LEFT JOIN scan.

// Go — mongo-driver (document)
type User struct {
    ID         int        `bson:"_id"`
    FirstName  string     `bson:"first_name"`
    Positions  []Position `bson:"positions"`
}
```

```rust
// Rust — Diesel (relational, fully type-checked schemas at compile time)
#[derive(Queryable, Identifiable)]
#[diesel(table_name = users)]
struct User { id: i32, first_name: String, region_id: Option<i32> }
// Diesel generates SQL at compile time; ALTER TABLE without regenerating bindings = build error.

// Rust — mongodb crate (document, BSON-typed)
#[derive(Serialize, Deserialize)]
struct User { _id: i32, first_name: String, positions: Vec<Position> }
```

**What the stdlib / ecosystem actually does:** Postgres's `JSONB` operators (`->`, `->>`, `@>`, `?`, `jsonb_path_query`) and **GIN indexes** let you have schema-on-read columns inside a schema-on-write table — the *single biggest reason* most projects no longer need MongoDB. SQLite added JSON1 extension in 2015 and made it built-in in 3.38 (2022). MySQL 5.7+ has a JSON type with similar operators. **The "NoSQL vs SQL" battle has ended in a hybrid truce** at the engine level.

### Where this shows up in real systems
- **MongoDB → Postgres+JSONB migrations** are the canonical 2020s engineering blog post (Heap, Wave, Theory, dozens of mid-size companies). The pattern is always the same: *we started with documents for "schema flexibility," accumulated many-to-many relationships, hit join pain, and the relational + JSONB hybrid was the right destination.*
- **DynamoDB single-table design** (Rick Houlihan's canonical talks at re:Invent 2018–2020) is the *aggressively-document* extreme: store every entity in one table, denormalize ruthlessly, do all joins in the app. It works at FAANG scale but punishes feature evolution — many AWS-internal teams have publicly reverted to multi-table designs after years of single-table debt.
- **GraphQL's resolver model** is the *application-side* response to the impedance mismatch: let the client describe the shape it wants, and resolve fields via independent backend calls. It is the modern descendant of Hibernate's `@OneToMany`-driven N+1 problem, complete with the same DataLoader-batching gymnastics required to make it fast.
- **Linear, Notion, Figma** all store *user-perceived documents* as **JSONB blobs with relational metadata indexes** — the exact pattern this chunk argues toward. They get document locality for the user load and relational integrity for cross-entity links (Linear's "blocks this issue," Notion's "linked databases").
- **Datomic and XTDB** are explicit revivals of the *immutable + relational + graph* approach, storing time-versioned tuples and letting you query with Datalog. Niche but illuminating: they're the closest the 2020s have come to taking the CODASYL-relational debate back to first principles.
- **Snowflake / BigQuery / DuckDB** have a `VARIANT` / `JSON` column type that is schema-on-read inside an otherwise columnar analytics store — same hybrid resolution at the OLAP end of the spectrum.

### Diagnostic questions
1. *"Our team picked Mongo two years ago. We now have user-to-user recommendations, user-to-company endorsements, and a feed that aggregates across both. Should we migrate?"* — Probably yes, eventually. Walk the chapter's many-to-many checklist; if you have ≥3 distinct many-to-many relationships, the relational model's optimizer is now doing meaningful work that your app code is currently faking poorly. Postgres + JSONB is the destination; the migration is incremental, table-by-table.
2. *"Why does Hibernate do N+1 queries by default?"* — Because the impedance mismatch's *easy* mode is "load parent, then lazily load each child." Fixing it requires *explicit* join fetching (`JOIN FETCH` / `@BatchSize`) which couples your ORM config to your access patterns — exactly the CODASYL trap, in modern clothes.
3. *"My DBA says ALTER TABLE on a 500M-row table is a 4-hour outage. Is that still true in 2026?"* — Not on Postgres for *additive* changes (`ADD COLUMN ... DEFAULT NULL` is metadata-only since PG 11); it *is* still true on MySQL/InnoDB for column-rewriting changes without `pt-online-schema-change` or `gh-ost`. The schema-on-read pitch is weaker in 2026 than when this chapter was written because relational DBs have caught up on the *write-time* migration cost.
4. *"Is graph database actually the right answer for our 'recommendations' feature?"* — Only if traversal queries dominate ("friends of friends of friends who like jazz"). For "recommendations by user X for user Y" (1 hop), a relational join table is faster and simpler. Use graph when **the depth of traversal is variable** and matters per query.
5. *"Why did CODASYL lose so completely that nobody remembers it?"* — Three reasons: (a) hand-coded access paths didn't survive frequent data-model changes; (b) the relational model's query optimizer turned out to be a hugely amortizable investment; (c) ad-hoc queries (the user typing a new SELECT without pre-declaring a path) were impossible in CODASYL and trivial in SQL. The third reason is the deepest — **the relational model unlocked an audience of analysts that CODASYL excluded**.
6. *"Should I use JSONB columns everywhere?"* — No. The rule of thumb: **columns for fields you query or constrain; JSONB for the long tail of "we'll figure it out later" attributes**. JSONB is wonderful but you lose the type-system safety net for any field you put in there — every reader must handle missing/wrong types.
7. *"What's the schema-flexibility argument actually buying me?"* — Three things: (a) zero-downtime additive shape changes; (b) heterogeneous records with optional fields (e.g., one event log with 50 event types); (c) faster early-stage iteration. All three are real, all three are also achievable with `JSONB` + nullable columns in Postgres. Pick Mongo if the *write throughput / horizontal-shard* story matters more than the schema story alone.

### See also
- **DDIA Ch.3** (Storage and Retrieval) — the L3 layer below this one: B-trees vs LSM-trees are the physical reason document and relational stores have different write/read profiles.
- **DDIA Ch.4** (Encoding and Evolution) — schema-on-read becomes schema-on-the-wire when services exchange messages; Avro/Protobuf/Thrift are this chapter's argument at the network layer.
- **DDIA Ch.10** (Batch) and **Ch.11** (Stream) — analytics workloads change which data model wins; columnar + schema-on-read dominates at the warehouse end.
- **DBI Ch.1–2** — Petrov's *Database Internals* covers Selinger optimizer mechanics in depth; this chapter motivates *why* the optimizer matters, DBI shows *how* it works.
- **LDDD** (Learning Domain-Driven Design) — Khononov's aggregate-vs-entity boundaries map directly to "this is one document" vs "these are separate entities with references"; the strategic DDD layer answers Kleppmann's "what shape is your domain?" question.
- **TPP Ch.8 Orthogonality** (entry [2026-05-18]) — abstracting your DB behind a Repository is *literally* Hunt & Thomas's reversibility principle applied to this chapter's choice; you get to *defer* the schema-on-read/write decision per entity.
- **DSG Ch.4–5** — distributed Go services typically pick per-service storage; this chapter is the menu that decision is choosing from.

---

## [2026-05-18] Describing Performance, Coping with Load, and Maintainability — The Full Scalability/Maintainability Spine · pp.35–48 · Ch.1 §§ Scalability → Maintainability → Summary

### TL;DR
Kleppmann closes Chapter 1 by handing you the **vocabulary every backend conversation about scaling and maintenance is actually using under the hood**: throughput vs response time vs latency (different things, casually conflated), percentiles (p50/p95/p99/p999) instead of means, **tail-latency amplification** (why fanout services rot at the tail), the load-generator gotcha (open-loop vs closed-loop testing), and the false dichotomy between scaling up and scaling out. The chapter then pivots to maintainability and frames it as **three engineering postures, not one virtue**: *operability* (make life easy for ops), *simplicity* (kill accidental complexity à la Moseley & Marks), and *evolvability* (cheap change). Together this chunk is the implicit grammar of every postmortem, capacity plan, and SLA you will ever read.

### History — "why does this exist?"
The percentile vocabulary is industry, not academia: **Amazon's 2007 Dynamo paper (DeCandia et al., SOSP)** put p99.9 on the map by stating it as a hard internal SLO and showing the tail mattered more to revenue than the mean. The "**100 ms costs Amazon 1% in sales**" number is from Greg Linden's 2006 Stanford talk recounting an internal A/B test — it is the most quoted single statistic in latency engineering. **Tail latency amplification** was named and quantified in **Dean & Barroso's 2013 CACM article "The Tail at Scale"**, which showed that for a service fanning out to 100 backends, a 1-in-100 slow backend means roughly *every* user request waits on a slow tail. The percentile-approximation algorithms Kleppmann lists — **t-digest** (Dunning, 2014), **HdrHistogram** (Tene, 2012), **forward decay** (Cormode et al., 2009) — are what actually runs inside Prometheus, Datadog, and DynamoDB telemetry today; their existence is *why* you can afford to publish p99.9 on every endpoint. The maintainability triad has older roots: **Fred Brooks's 1986 "No Silver Bullet"** introduced the *essential vs accidental* complexity distinction Moseley & Marks ("Out of the Tar Pit," 2006) sharpened into the operational definition Kleppmann cites.

### Intuition — "this is like…"
Latency vs response time is the difference between **how long you waited in the restaurant queue (latency)** and **how long until your food was on the table (response time)** — the latter includes the queue, the kitchen's actual cook time, and the waiter walking it back. The mean-vs-percentile point is the difference between **"the average customer waits 8 minutes"** and **"1 in 20 customers waits 25 minutes and never comes back"** — it is the second number that decides whether the restaurant survives, and it is the second number the mean *cannot tell you*.

### Mechanics

**1. The three latency words — get these wrong in production and you will misdiagnose every incident:**

| Word | What it measures | Who sees it |
|---|---|---|
| **Service time** | The pure time the server spends processing the request (CPU + I/O actually doing the work) | The server's profiler |
| **Latency** | Time the request was *latent*, awaiting service (queueing) | The internal scheduler / queue |
| **Response time** | What the client sees: service time + latency + network RTT + retransmits + GC pause + page fault + … | The user |

The book footnotes a definition pulled from queueing theory: **latency is the wait, response time is the wait + the work**. Most monitoring stacks call their `request_duration_ms` "latency" — it is actually response time. The distinction matters because optimizing the service time (faster handler) and optimizing the latency (deeper thread pool, smaller queue) are **different fixes** and the wrong fix wastes a sprint.

**2. Why percentiles, not means.** Worked numbers from p.37 (Amazon's stated SLO):

```
Service has 1000 requests in last minute. Sorted ascending:
  p50  =  request #500   →  e.g.   80 ms   ("median user")
  p95  =  request #950   →  e.g.  240 ms   ("frustrated user")
  p99  =  request #990   →  e.g.  900 ms   ("angry user")
  p999 =  request #999   →  e.g.  4200 ms  ("Amazon's biggest customer")

A single 60-second GC pause in the same minute changes the mean by ~60 ms.
It changes the p999 by ~60,000 ms.
The mean buried the customer who was about to leave you.
```

**The Amazon argument (p.37):** the p99.9 customer is *correlated with revenue*, because customers with the most data (most account history, biggest carts) take the longest to render, so the tail = your whales. Optimizing the median while letting the tail rot is **revenue-negative even when the dashboard looks green**.

**3. Tail-latency amplification (the diagram on p.39, redrawn).** The single most counter-intuitive result in the chapter:

```
End-user request
    │
    ▼
┌──────────────────── fan-out to 10 backends, in parallel ────────────────────┐
│   B1   B2   B3   B4   B5   B6   B7   B8   B9   B10                          │
│   ●────●────●────●────●────●────●────●────●────●─SLOW─────────────►         │
└─────────────────────────────────────────────────────────────────────────────┘
                                                         ▲
   end-user response time = max(B1..B10), not mean ──────┘

If each backend has p99 = 1 s (1% slow), then probability that
  all 10 calls are fast  =  0.99^10  ≈  0.904
  ⇒  ~9.6% of end-user requests see a slow tail
  ⇒  effectively every user's p90 is now their backends' p99
```

**Rule of thumb (Dean & Barroso):** in a service that fans out to N backends, the user's p_X latency tracks each backend's p_(X^(1/N)). At N=100 backends, your user p99 is each backend's **p99.99**. This is why microservice architectures have a tail-latency problem that monoliths did not: more hops = more chances to hit the slow side of every distribution.

**4. The load-generator gotcha (p.38) — closed-loop vs open-loop.** A subtle bug that invalidates almost every load test written by an engineer who has not been burned by it before:

```
WRONG (closed loop, "coordinated omission" — the famous Tene bug):
    while running:
        send_request()
        wait_for_response()      ← this is the bug
        record_latency()

If the server stalls for 1 s, you send 0 requests during that second.
Your histogram has no entries for the period when latency was worst.
You measure the server's *good* moments and miss its *bad* moments.

RIGHT (open loop):
    while running:
        send_request_async()     ← do not wait
        sleep(1 / target_rps)
        record_latency_when_response_arrives()
```

Closed-loop generators *artificially keep queues short* by throttling themselves whenever the server is slow — exactly the moments you need to be measuring. Use **wrk2**, **k6 with constant-arrival-rate**, or **vegeta** in rate mode — never `ab` for tail measurement.

**5. Approaches for coping with load — the up/out/elastic decision tree (p.39–40):**

| Approach | What it is | When it wins | When it loses |
|---|---|---|---|
| **Scale up (vertical)** | One bigger machine | Workload fits on one box; stateful DB; latency-sensitive (no network hop) | High-end hardware has super-linear pricing; single point of failure |
| **Scale out (horizontal, "shared-nothing")** | Many smaller machines | Stateless services; embarrassingly parallel work; commodity-priced fleet | Stateful systems hit the *distributed* tax (consensus, replication, partitioning) |
| **Elastic** | Auto-scales on detected load | Bursty / unpredictable load (Black Friday, news spikes) | Adds operational surprises; "scaled at 3am because of a metric blip" |

Kleppmann's editorial is explicit: **"good architectures usually involve a pragmatic mixture"** — a few beefy machines + a fleet of small ones beats either extreme. And on stateful systems: **"common wisdom until recently was to keep your database on a single node until forced to distribute it,"** with the hedge that this *common wisdom may be changing* as distributed-system tooling matures (a thesis the rest of the book argues).

**6. The "no magic scaling sauce" principle (p.40).** Two systems with identical aggregate throughput look architecturally nothing alike:

```
System A: 100,000 req/sec × 1 KB each   = 100 MB/s
System B:  3 req/min      × 2 GB each   = 100 MB/s

Same throughput. Completely different bottlenecks:
  A → connection pool, request batching, cache hit rate, p99 tail
  B → bulk transfer, checkpointing, resume-on-failure, memory ceiling
```

Architecture is built around *which operations are common* (the **load parameters**). Get the load parameters wrong and the architecture is at best wasted, at worst counterproductive. Hence the chapter's startup advice: in an unproven product, **iterate-ability beats scalability** — you do not yet know which load parameters you are optimizing for.

**7. Maintainability — the three postures (pp.41–44):**

| Posture | One-line definition | Concrete things it asks of the system |
|---|---|---|
| **Operability** | Make routine ops tasks easy | Good monitoring; automation hooks; no per-machine state; predictable, documented behavior; sensible defaults with override knobs; self-healing where appropriate |
| **Simplicity** | Manage *accidental* complexity (Moseley & Marks) | Good abstractions (SQL hides on-disk B-trees, HLLs hide CPU registers); explicit state machines; consistent naming; kill special-cases |
| **Evolvability** (a.k.a. extensibility, modifiability, plasticity) | Cheap change as requirements drift | Tested code; refactoring discipline; small modules; the *system-level* version of TDD/refactoring (p.43) — "how would you refactor Twitter from approach 1 to approach 2?" |

The deep point on simplicity: **complexity is accidental if it is not inherent in the user-facing problem**. A SQL query planner is *essential* complexity (joins are hard); a 14-step deploy ritual that exists because nobody wrote a script is *accidental* complexity. The first you accept; the second you eliminate. The tool for the job is **abstraction** — and the chapter is honest that *finding good abstractions is very hard*, especially for distributed systems where "we have many good algorithms but it's much less clear how to package them."

### If you were the SRE on call when p99 spikes…
Kleppmann's chapter is the textbook for what to *check first*. Start by asking which percentile spiked: **mean up, p99 flat** = workload shift (cheap requests displaced by expensive ones); **p99 up, mean flat** = tail event (GC, queue depth, slow disk on one node, head-of-line blocking from one expensive request stalling the line). If you only watch the mean, you will misread the second case as "no problem" while users churn. Second, ask the **fanout question**: how many backends does this endpoint touch? If N is large and any one backend's p99 is misbehaving, the user's p90 is now the laggard's p99 — fix the slowest backend, not the average one. Third, before believing your load test, check whether the generator is open-loop; if it isn't, the worst minute of the incident is **literally absent from your data**.

### Where this shows up in real systems
- **Google SRE Book Ch.6** ("Monitoring Distributed Systems") explicitly tells you to alert on percentiles, never means — same reasoning Kleppmann gives, codified into Google's SLO discipline (and now the SRE Workbook's "burn-rate alerting").
- **Prometheus' `histogram_quantile` and `_bucket` design** exists because of the *do not average percentiles* footnote on p.38. Histograms are aggregable across replicas (you sum the bucket counts, then compute the quantile); means and pre-computed percentiles are not. Every Grafana p99 panel implicitly relies on this.
- **AWS DynamoDB's published "single-digit-millisecond p99"** is a marketing translation of exactly this chapter — the SLO is stated at p99, not mean, because that is what the calling fanout services need to bound their *own* tails.
- **Netflix's Hystrix / resilience4j circuit breakers** exist to defend against tail-latency amplification: if backend B9 is slow, fail fast and degrade rather than hold the whole user request hostage.
- **Cloudflare's load testing infra** publicly switched from `ab` to `wrk2` in a 2017 blog post specifically because of coordinated omission — same bug Kleppmann calls out in one paragraph.

### Diagnostic questions
1. *"My monitoring shows p99 is flat but users are complaining about slowness — what's wrong?"* — Either you are averaging percentiles across replicas (mathematically meaningless, see p.38), or you are measuring server-side and missing client-side queueing/network effects, or your load generator is closed-loop.
2. *"Why is moving from monolith to 30 microservices making p99 worse, even though each service is fast?"* — Tail-latency amplification: fan-out turns each backend's p99 into the user's p_(0.99^N). Fix the slowest backend, not the average.
3. *"We auto-scale on CPU. Why are we still missing SLO?"* — CPU-based autoscaling can't see queue depth. Scale on request-queue length or request rate per replica instead; otherwise you scale *after* the tail has already gone bad.
4. *"Should we shard the database to handle 10× growth?"* — First test if a bigger box gets you to 10× cheaper than the operational cost of going distributed; Kleppmann's prior is "single-node until forced," and the book spends Chs. 5–6 explaining why distributing state is qualitatively harder than distributing stateless services.
5. *"Why does the load test say we can do 50k RPS but production tops out at 30k?"* — Closed-loop coordinated omission, almost certainly. The test was throttling itself when the server slowed down.

### See also
- **DDIA Ch.5–6** (Replication, Partitioning) — the entire mid-book is "what becomes hard when you scale out."
- **DDIA earlier entry [2026-05-17]** (Reliability — Faults vs Failures) — the maintainability triad here is the *operational* sibling of the fault/failure framework.
- **OSTEP Ch.7–10** (Scheduling) — head-of-line blocking is a scheduling problem; understanding MLFQ explains why one slow request can poison a queue.
- **SAHP** — the entire "Hard Parts" book is about the trade-offs of the scale-out architectures Kleppmann is sketching here.
- **REF** (Refactoring) — Kleppmann's evolvability is the system-level version of Fowler's local-level refactoring discipline.

---

## [2026-05-17] Reliability — Faults vs Failures, and Tolerating Unreliable Components · pp.23–34 · Ch.1 § Reliability

### TL;DR
Kleppmann's opening move is to **separate the words we use carelessly** — a *fault* is one component deviating from spec, a *failure* is the whole system stopping its required service to the user. The probability of any individual fault cannot be driven to zero (disks die, processes hang, ops type the wrong thing), so the engineering goal is not fault prevention but **preventing faults from cascading into failures**. The chapter then sorts faults into three lanes — hardware, software, human — each with a fundamentally different correlation structure, which dictates a fundamentally different tolerance strategy.

### History — "why does this exist?"
The fault/failure distinction comes from **Jim Gray's 1985 Tandem Technical Report TR-85.7** ("Why Do Computers Stop and What Can Be Done About It?"), which catalogued real Tandem outages and found that ~42% were operator/configuration errors, ~25% software, ~18% hardware — overturning the then-dominant assumption that hardware was the main villain. **Avižienis et al.'s 2004 "Basic Concepts and Taxonomy of Dependable and Secure Computing"** crystallized the vocabulary the field still uses (fault → error → failure chain). The book's worked numbers — "10,000 disks → 1 dies/day" — come from **Pinheiro/Weber/Barroso's 2007 Google FAST paper** on disk failure rates in the wild, which was the first large-scale public dataset that contradicted vendor MTBF claims by 2–5×.

### Intuition — "this is like…"
A **commercial airliner.** A single sensor failing is a *fault*, not a *failure*: there are three of them voting, the autopilot tolerates one being wrong, the plane keeps flying. The failure mode the FAA actually fears is the *correlated* one — all three sensors freezing identically because of the same icing condition. That is exactly Kleppmann's distinction: independent faults are tolerable through redundancy; **correlated faults are catastrophic** because redundancy doesn't help if every copy fails together.

### Mechanics

**The three fault lanes — what correlates with what:**

| Lane | Correlation | Tolerance technique | The real failure mode |
|---|---|---|---|
| Hardware | Mostly **independent** (one disk dying ≠ another dying) | Redundancy (RAID, dual PSU, hot-swap CPU) | "We had 10K disks, we lost the rack's PDU" — correlated by *shared cause* |
| Software | **Heavily correlated** (every replica runs the same buggy binary) | Process isolation, crash-restart, monitoring, slow rollouts | Leap-second 2012: every Linux box on Earth hung the same minute |
| Human | **Bursty + adversarial** (one ops error fans out everywhere) | Sandboxes, fast rollback, telemetry, blameless postmortems | 2017 S3 outage: one operator typo took down half the internet |

**The fault-tolerance loop (the chapter's implicit algorithm):**

```
                    ┌──────────────────────┐
                    │ design assumes fault │
                    │   can & will occur   │
                    └─────────┬────────────┘
                              │
            ┌─────────────────┴─────────────────┐
            ▼                                   ▼
   ┌────────────────┐                ┌─────────────────────┐
   │   redundancy   │                │ deliberately inject │
   │ (mask the fault)│               │   faults (Chaos     │
   │                │                │   Monkey, gameday)  │
   └────────┬───────┘                └─────────┬───────────┘
            └────────────┬─────────────────────┘
                         ▼
              ┌─────────────────────┐
              │ tolerance machinery │
              │  stays exercised &  │
              │     trustworthy     │
              └─────────────────────┘
```

The Chaos Monkey insight (Netflix, 2010) is counterintuitive: **deliberately increase your fault rate** so the recovery code runs in daylight, not at 3 AM in production for the first time. Code paths that aren't exercised will rot; the cure is to exercise them on purpose.

**Twitter fan-out — load is not one number (pp.32–34).** The chapter's only worked example in this chunk. Twitter's *write* rate (4.6k tweets/sec average, 12k peak) is trivially handleable. The system breaks on **fan-out**: each tweet → ~75 followers → 345k home-timeline writes/sec on average, with celebrity tail at >30M writes per tweet. Two designs:

| | Approach 1 (read fan-out) | Approach 2 (write fan-out) |
|---|---|---|
| **Post** | O(1) — append to global tweet table | O(followers) — push to each follower's timeline cache |
| **Read** | O(followees) — JOIN + merge sorted | O(1) — read precomputed timeline |
| **Wins when** | Reads cheap, writes hot | Reads >> writes (Twitter: 300k reads/s vs 4.6k writes/s) |

Twitter started with #1, switched to #2, and now runs a **hybrid** — celebrities bypass write fan-out (their tweets are pulled at read time), normal users use write fan-out. The lesson DDIA wants you to take: **a single "RPS" number is meaningless** without knowing the fan-out distribution behind it. The right load parameter is the one whose tail dominates the cost.

### If you were the SRE…
You're paged because your database cluster's p99 latency tripled. The dashboards show one disk has elevated error rates but hasn't fully died. **Replace it now, or wait until off-hours?** Kleppmann's framing tells you to think correlation, not just severity: a single disk is the *independent* fault lane (low blast radius); your replacement procedure is what could create a *correlated* fault (you've never tested it under load, the kubelet might rebalance other replicas onto already-hot nodes). The right answer is rarely about the disk — it's about whether your recovery code has been *exercised* recently. If it has (gameday last month), replace now. If it hasn't, you're flying blind and the cure may be worse than the disease.

### Where this shows up in real systems
- **AWS S3 February 2017 outage.** Operator running a runbook command typed an extra argument and removed too many capacity servers. The failure wasn't the typo (a human fault — inevitable) but the absence of a confirmation step on a destructive command (a design fault — the system let one keystroke fan out). AWS's postmortem rewrote the tool to require a soft-fail dry-run first — exactly the "decouple mistakes from failures" tip.
- **Google's Borg / Kubernetes liveness probes.** Direct application of "allow processes to crash and restart." Borg doesn't try to prove your binary is correct; it assumes it will misbehave and kills + restarts it on a schedule. This is software-fault tolerance built into the platform layer.
- **Cassandra hinted handoff & read repair.** Designed assuming nodes will be temporarily down (independent hardware faults) and that writes mid-outage must reconcile later. The whole protocol exists because Cassandra *refuses* to assume nodes are reliable, which is the chapter's thesis made physical.

### Diagnostic questions
1. **Q:** Your service has 99.99% uptime per node and runs on 10 nodes. What's the system's uptime?
   *Wrong-answer trap:* "99.99% × 10 = 99.9%." That assumes independence and that all 10 must be up. The real answer depends on the architecture: if any one can serve, uptime is ~1 − (10⁻⁴)¹⁰ ≈ 100%; if all 10 must be up, ~99.9%. The math is meaningless without the topology.
2. **Q:** You add a third replica to a 2-replica system. Does reliability triple?
   *Wrong-answer trap:* "Yes." Not if all three replicas share a cause of failure — same rack, same OS image, same buggy library version. Independence has to be *engineered*, not assumed.
3. **Q:** Why is Chaos Monkey net-positive even though it intentionally causes outages?
   *Wrong-answer trap:* "It finds bugs." Closer, but the deeper reason is that recovery code *only works if it runs.* Dormant recovery paths bit-rot — Chaos Monkey is a forcing function that keeps them executable.
4. **Q:** A bug only manifests once a leap-second occurs. Is that a hardware fault or a software fault?
   *Wrong-answer trap:* "Software." Right answer with a twist: it's a software fault, but its correlation structure is the killer — every machine running that code fails at the same instant. The category isn't just "where the bug lives" but "how it correlates."
5. **Q:** Twitter switched from approach 1 (read fan-out) to approach 2 (write fan-out). Under what load profile would the reverse switch — back to approach 1 — be correct?
   *Wrong-answer trap:* "Never, approach 2 is just better." Wrong — approach 1 wins when *reads << writes* (e.g., a logging system where 99% of records are written and never queried). The right answer is workload-dependent.

### See also
- OSTEP Ch. on processes — "allow processes to crash and restart" is the OS-level form of this chapter's software-fault tip.
- COD 2026-05-15 (CPU performance equation) — yield/redundancy is *hardware* fault tolerance; this chapter generalizes the idea up the stack.
- TPP 2026-05-17 (DRY) — "every piece of knowledge has a single representation" is the *prevention* dual of this chapter's *tolerance* mindset: prevent inconsistency by single-sourcing, tolerate failure by replicating. The two principles seem to contradict; they apply at different layers.

---

## [2026-05-16] The Book's Thesis — Data-Intensive vs Compute-Intensive, and the Three Pillars · pp.9–22 · Front Matter (TOC + Preface)

### TL;DR
Kleppmann's Preface sets two coordinate systems the rest of the book moves through. **First**, it draws a line between **data-intensive** applications (data volume, complexity, or change-rate is the bottleneck) and **compute-intensive** applications (CPU cycles are the bottleneck) — and declares this book is about the first. **Second**, it names the **three pillars** every data system is judged on: **reliability** (works correctly even with faults), **scalability** (handles growth in data or load), and **maintainability** (different people, different times, can keep evolving it). Every chapter that follows is an attack on one of these axes — Chapter 1 is the formal definition, Chapters 2–4 give you single-node tools to achieve them, Chapters 5–9 do the same for distributed systems, Chapters 10–12 cover derived data. The Preface's hidden second message: **principles outlast tools**. Buzzwords ("NoSQL," "Big Data," "Web-scale") rot in five years; the principles behind why a B-tree is range-friendly and an LSM is write-friendly do not.

### History — "why does this exist?"
The "data-intensive" framing is a deliberate **2010s response to a 2000s vocabulary crisis**. By the time Kleppmann started writing (~2014), the term **"Big Data"** had been so thoroughly hyped by McKinsey reports and conference keynotes that engineers couldn't use it in a technical conversation — it was a marketing word, not an engineering one. The phrase **"data-intensive scientific computing"** was coined earlier by **Jim Gray** in his "Fourth Paradigm" lecture (Microsoft Research, 2007), arguing that data exploration had become the fourth scientific paradigm alongside theory, experiment, and simulation. Kleppmann generalized it from science to software systems. The **three-pillar framing** (reliability, scalability, maintainability) is older still — it descends from **Saltzer & Schroeder's "The Protection of Information in Computer Systems" (1975)** and **Anita Borg's reliability metrics work at DEC (1980s)**. But Kleppmann's specific *triad* — bundling them as the primary evaluation axes for data systems — became the de-facto vocabulary in distributed-systems interviews from 2017 onward. If you have ever been asked "tell me about reliability vs availability" in a system design interview, that question crystallized after this book.

### Intuition — "this is like…"
Think of a data system as a **municipal water utility**, not a fancy espresso machine. An espresso machine is compute-intensive — the beans are small, the work per shot is in the grinder and pump. A water utility is data-intensive — the volume is enormous, the pipes are the bottleneck, and the operator's challenge is keeping water flowing reliably, scaling treatment plants for population growth, and maintaining infrastructure that outlives the engineers who built it. Kleppmann's three pillars map cleanly: **reliability** is "no E. coli in the water," **scalability** is "handle the new housing development," **maintainability** is "the 1950s pipes can be replaced without a citywide shutdown." When you read DDIA, every algorithm and trade-off is in service of one of those three jobs.

### Mechanics

**The two-axis frame the book sets up:**

```
                  Compute-intensive               Data-intensive
                  ───────────────────             ──────────────
Bottleneck        CPU cycles, FLOPs               Bytes moved, bytes stored,
                                                  bytes changed per second
Examples          Ray tracing, simulation,        OLTP databases, search engines,
                  weather modeling, ML            event streaming, analytics
                  training
Typical scale     1 node × big CPU/GPU            N nodes × commodity hardware
Failure mode      "Slow"                          "Lost data" / "Stale data" /
                                                  "Inconsistent answer"
Tools the         BLAS, CUDA, MPI, SLURM          Postgres, Kafka, Cassandra,
book covers       (NOT this book)                 Spark, Elasticsearch
```

**The three pillars and what they actually mean in this book:**

| Pillar | Working definition | Chapters that attack it | The trap |
|---|---|---|---|
| **Reliability** | "System continues working *correctly* (right answer at right performance) even in the face of adversity (faults)." | Ch.1, Ch.5 (replication), Ch.7 (transactions), Ch.8 (distributed faults) | Confusing reliability with availability — a system can be available *and* serve wrong answers |
| **Scalability** | "Reasonable ways of dealing with growth — in data volume, traffic, complexity." | Ch.1, Ch.5–6 (replication, partitioning), Ch.10–11 (batch, stream) | Treating scalability as a single number ("scale to N users") instead of a function of load parameters |
| **Maintainability** | "Many different people will work on the system, productively." | Ch.1, Ch.4 (encoding/evolution), Ch.12 (futures) | Optimizing only for first-author productivity; ignoring operability, simplicity, evolvability |

**Three sub-principles of maintainability** (foreshadowed on p.10, formalized in Ch.1):
- **Operability**: easy for ops teams to keep running smoothly
- **Simplicity**: new engineers can understand the system (manage accidental complexity)
- **Evolvability**: easy to make future changes (also called *extensibility*, *modifiability*, *plasticity*)

**The Preface's structural argument about *principles vs tools*** (p.16):

```
        ┌────────────────────────────────────────┐
        │ TOOLS layer (decays in 5-10 years):    │
        │   PostgreSQL 13, Cassandra 4, Kafka,   │
        │   Redis, Spark, DynamoDB, ...          │
        └────────────────────────────────────────┘
                          ▲
                          │ "where does this tool fit in?"
                          │
        ┌────────────────────────────────────────┐
        │ PRINCIPLES layer (decades-stable):     │
        │   • Replication (leader/leaderless)    │
        │   • Partitioning (hash / range)        │
        │   • Storage (B-tree / LSM)             │
        │   • Consistency models (linearizable,  │
        │     causal, eventual)                  │
        │   • Encoding & schema evolution        │
        │   • Batch vs stream                    │
        └────────────────────────────────────────┘
```

The book is structured so that every principle gets unpacked once and then mapped onto multiple tools. That's why the chapter on "replication" doesn't say "this is the MySQL chapter" — it covers leader-based replication abstractly, then names MySQL, PostgreSQL, MongoDB, Cassandra as concrete instances.

### Where this shows up in real systems
- **System design interviews at FAANG.** The "design Twitter / Uber / WhatsApp" prompt is graded on exactly Kleppmann's three pillars. Candidates who don't structure their answer around reliability/scalability/maintainability tend to ramble; those who do, sound like staff engineers.
- **AWS Well-Architected Framework (2015→).** Its pillars (operational excellence, security, reliability, performance efficiency, cost optimization) overlap heavily with DDIA's — different industry, same concerns. Reading AWS docs is easier once the DDIA vocabulary clicks.
- **The "Hyrum's Law" of distributed systems.** Hyrum Wright's famous quip (Google) — "With a sufficient number of users of an API, all observable behaviors of your system will be depended on by somebody" — is a maintainability hazard the rest of the book quietly works around in many places (e.g., Ch.4's schema-evolution rules exist precisely because *every* JSON field becomes someone's contract).

### Diagnostic questions
1. **Q:** A pure CNN training pipeline running on a single GPU — does DDIA's framework apply?
   *Wrong-answer trap:* "Yes, all software." This is compute-intensive (GPU FLOPs dominate). DDIA's pillars apply only loosely — the more substantive guidance for that workload lives in HPC and ML-systems literature, not here.
2. **Q:** A system has 100% uptime SLA but returns stale reads sometimes. Reliable?
   *Wrong-answer trap:* "Yes — it's available." Available, yes; reliable, no. Reliability requires *correctness* under faults, not just liveness. This distinction sets up the entire CAP/PACELC discussion in Ch.9.
3. **Q:** Why does Kleppmann *not* define "Big Data"?
   *Wrong-answer trap:* "Oversight." Deliberate — the term is so overloaded it costs more clarity than it gives. The book uses precise replacements ("single-node vs distributed," "online vs batch") instead.
4. **Q:** Maintainability has three sub-principles. Which is *most* about *future* engineers, not current ones?
   *Wrong-answer trap:* "Operability." That's about ops teams *now*. **Evolvability** is the future-engineer answer — making the system easy to *change* later when requirements move.

### See also
- TPP Ch.1 (read today's other entry): the same "principles outlast tools" argument, generalized to all of software.
- SAHP Ch.1 — "Architecture" defined as the trade-off space; DDIA's pillars are SAHP's "non-functional requirements" in concrete form.
- DBI Ch.1 (already noted): the **storage engine internals** layer Kleppmann describes from 30,000 ft, Petrov goes deep on.
- COD Ch.1 §1.6 (already noted): same response-time/throughput tension, at the CPU level instead of the system level.

---
