# H01 Lab — URL Shortener

The lab for H01. Read `notes/hld-primers/H01-primer.md` first, then work through `EXPERIMENTS.md` (in this directory), then read `notes/hld-notes.md` entry H01.

The point of this lab is **not** to build a production URL shortener. It is to make the *failures* that motivate the H01 design **observable on your laptop**. Each experiment switches one knob, you run it, you see the failure or the fix, and a section of the note stops being abstract.

---

## Layout

```
labs/H01-url-shortener/
├── cmd/shortener/main.go     # the HTTP server — POST /shorten + GET /:id + GET /stats
├── internal/
│   ├── base62.go             # the encoding from the primer
│   ├── counter.go            # naive · sharded · block — three ID strategies
│   ├── id_hash.go            # hash-of-URL ID strategy (the rejected alternative)
│   ├── store.go              # MemStore + RedisStore — the persistent map id→url
│   └── cache.go              # in-process LRU with hit/miss counters
├── bench/
│   └── loadgen/main.go       # GET-side bench with uniform | zipf distributions
├── docker-compose.yml        # Postgres + Redis (Postgres unused in v1; reserved)
├── EXPERIMENTS.md            # the four experiments — read this next
└── README.md                 # you are here
```

---

## Prereqs

- Go 1.26+
- Docker + docker-compose (only for experiments that use Redis)

That's it. No global tool installs required.

---

## Running

### Start the server (the simplest config — pure in-process)

```bash
go run ./cmd/shortener
# id-mode=block (the prod variant) · store=mem · cache=none
```

### Switch ID strategy

```bash
go run ./cmd/shortener -id-mode=naive          # one global counter + mutex
go run ./cmd/shortener -id-mode=sharded        # N independent counters
go run ./cmd/shortener -id-mode=block          # sharded + per-goroutine pre-allocation
go run ./cmd/shortener -id-mode=hash -hash-width=4   # SHA256-based, tunable collisions
```

### Turn on caches

```bash
go run ./cmd/shortener -cache=lru -cache-size=1000
go run ./cmd/shortener -store=redis -redis=localhost:6379    # requires docker-compose up
```

### Start Redis (and Postgres, reserved for future)

```bash
docker compose up -d
docker compose ps          # confirm both are healthy
docker compose down        # stop when done
```

---

## Manual smoke test

```bash
# In one terminal:
go run ./cmd/shortener -cache=lru

# In another:
ID=$(curl -s -X POST http://localhost:8080/shorten \
        -d '{"url":"https://example.com"}' \
        -H 'content-type: application/json' | jq -r .id)

echo "got id: $ID"
curl -i http://localhost:8080/$ID       # 301 + Location
curl -s http://localhost:8080/stats     # hit/miss counters
```

---

## What each flag teaches

| Flag | Default | What changing it teaches |
|---|---|---|
| `-id-mode=naive` | (off) | The single-counter bottleneck the note's Deep dive 1 fixes |
| `-id-mode=sharded` | (off) | Sharding removes the bottleneck but doesn't pre-allocate |
| `-id-mode=block` | ✅ | Why the H01 note says "1k-block pre-allocation cuts contention 1000×" |
| `-id-mode=hash -hash-width=3` | (off) | The birthday paradox the note rejects hash IDs for |
| `-cache=lru` | (off) | The hit-rate curve under Zipf vs uniform — the Pareto assumption made concrete |
| `-store=redis` | (off) | What network RTT costs you on every cache miss |

---

## Where to go next

After running the experiments in `EXPERIMENTS.md`:

1. Re-read `notes/hld-notes.md` entry H01. Lines should now click — you've seen the contention, you've seen the collision rate, you've seen the cache hit rate curve.
2. Try the "extensions" at the bottom of `EXPERIMENTS.md` — they map to the note's *Common follow-ups* section.
3. On Sunday revisit, you'll re-derive the design from memory; the lab is the muscle memory that makes that derivation cheap.
