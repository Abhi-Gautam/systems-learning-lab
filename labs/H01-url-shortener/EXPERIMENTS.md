# H01 Lab — Experiments

Four experiments. Each one switches one knob, makes ONE thing in the H01 note observable, and takes <5 minutes to run.

Each experiment ends with **"What the note says"** — a one-paragraph link back to the relevant section of `notes/hld-notes.md` entry H01.

---

## Setup (do once)

```bash
cd labs/H01-url-shortener
go build ./...                    # confirm everything compiles
```

For experiments 3 and 4 you'll also need:

```bash
docker compose up -d              # starts redis on :6379
```

---

## Experiment 1: feel the counter bottleneck

**The claim**: a single global counter behind a mutex is the bottleneck the H01 design exists to remove. Sharding alone helps. Block pre-allocation kills it.

**Run** (3 separate runs, killing the server between each):

```bash
# 1a. NAIVE — one global counter
go run ./cmd/shortener -id-mode=naive

# In another terminal:
go run ./bench/loadgen -seed=5000                            # populate (uses POST)
# the seed phase itself is the POST workload — note its duration
```

```bash
# 1b. SHARDED — 4 independent counters
go run ./cmd/shortener -id-mode=sharded -shards=4
go run ./bench/loadgen -seed=5000
```

```bash
# 1c. BLOCK — sharded + 1000-ID block pre-allocation per goroutine
go run ./cmd/shortener -id-mode=block -shards=4 -block-size=1000
go run ./bench/loadgen -seed=5000
```

**What to watch**: how long the seed phase takes in each mode. The naive run will be 5-10× slower than block at the same concurrency.

**Why**: in `naive`, every POST grabs the same mutex. With ~64 concurrent workers, ~63 of them are always waiting. In `block`, each worker holds a private block of 1000 IDs and only contends every 1000th request.

**What the note says**: this is "Deep dive 1: ID generation — base62 counter vs hash vs Snowflake." Specifically the "Counter row hot-spot" failure mode and its mitigation. You just observed the spike and the fix.

---

## Experiment 2: watch the birthday paradox bite (hash IDs)

**The claim**: hashing the URL to derive an ID seems clean, but at any sane ID width collisions become a meaningful fraction of writes at scale.

The birthday paradox says: in a space of N slots, you expect the first collision around √N draws. Tabulated for base62:

| Width | Slot space | Expected first collision | After 100k writes |
|---|---|---|---|
| 7 chars | 3.5 × 10¹² | ~1.9 million writes | negligible |
| 5 chars | 9.2 × 10⁸ | ~30k writes | ~1% collision rate |
| 4 chars | 1.5 × 10⁷ | ~3900 writes | ~25% collision rate |
| **3 chars** | 2.4 × 10⁵ | ~490 writes | nearly all writes collide |

**Run**:

```bash
# Start hash mode with a deliberately tight slot space
go run ./cmd/shortener -id-mode=hash -hash-width=4

# Pound it with POSTs
go run ./bench/loadgen -seed=50000

# Read the collision rate
curl -s http://localhost:8080/stats | jq
```

**What to watch**: `hash_collision_rate` climbs from ~0% at the first few hundred URLs to >20% by 50k URLs. The server retries with a salt, but each retry is variable extra latency on the write path — exactly what the note warns about.

**Try `-hash-width=3`** if you want to see the cliff: collision rate hits 50%+ within a few thousand POSTs.

**What the note says**: this is the "Hash collision under load" row of the failure-modes table in Deep dive 1, and the "Why not just hash the URL" diagnostic question. You just watched the failure mode actually happen.

---

## Experiment 3: read path — no cache vs in-process LRU vs Redis

**The claim**: the H01 architecture's "95% CDN / 4% Redis / 1% DB" split rests on caches doing nearly all the work. Even on one laptop, you can see the latency tier-by-tier.

**Run**:

```bash
# Start docker
docker compose up -d

# 3a. No cache, mem store — best case for "no cache"
go run ./cmd/shortener -cache=none
# In another terminal:
go run ./bench/loadgen -seed=10000
go run ./bench/loadgen -dist=uniform -qps=2000 -duration=10s

# 3b. With LRU
# Kill server, restart with cache:
go run ./cmd/shortener -cache=lru -cache-size=2000
go run ./bench/loadgen -dist=uniform -qps=2000 -duration=10s

# 3c. Redis-backed store (network RTT per miss)
go run ./cmd/shortener -store=redis -redis=localhost:6379 -cache=none
go run ./bench/loadgen -seed=10000
go run ./bench/loadgen -dist=uniform -qps=2000 -duration=10s
```

**What to watch**:

- `3a` p99 is dominated by Go's HTTP stack — ~1-3ms — because the mem store is in-process.
- `3b` p99 is similar (or slightly better — LRU is smaller than the full map), but the *cache_hit_rate* in `/stats` is interesting: under uniform load with 10k seeds and a 2k-entry cache, hit rate hovers around 20%. **That's the uniform-distribution failure mode in action.**
- `3c` p99 should jump to ~2-5ms — each miss is a Redis round-trip across the Docker network. This is the "Redis intra-DC ~1ms" tier from the note, made tangible.

**What the note says**: this is the architecture section's "3-tier cache topology" claim and Deep dive 2 ("Cache topology — 3-tier"). The note assumes the cache is doing the work; here you measured what happens when traffic is uniform and the cache *can't* do the work. Which is the lead-in to experiment 4.

---

## Experiment 4: Pareto/Zipf — why the cache architecture only works under skew

**The claim** — restated for clarity since the primer mentioned it abstractly:

The H01 architecture absorbs 95% of reads at the CDN. That's only possible because **a small fraction of URLs accounts for most traffic** (Zipfian / Pareto). If users redirected to URLs uniformly at random, the same cache would hit only a tiny fraction of the time and the architecture would collapse.

This experiment uses the same server, the same seed set, the same bench tool — only the *distribution* of which URLs get requested changes.

**Run**:

```bash
# Server with a small cache so the distribution matters
go run ./cmd/shortener -cache=lru -cache-size=500

# Seed 10000 URLs (so the cache is 5% of the working set — like the note's 1M-of-365B claim, scaled)
go run ./bench/loadgen -seed=10000

# 4a. UNIFORM — every URL equally likely
go run ./bench/loadgen -dist=uniform -qps=2000 -duration=10s
curl -s http://localhost:8080/stats | jq

# 4b. ZIPF (s=1.2 → moderate skew, like real web traffic)
# Kill server, restart fresh so cache counters reset:
go run ./cmd/shortener -cache=lru -cache-size=500
go run ./bench/loadgen -seed=10000
go run ./bench/loadgen -dist=zipf -zipf-s=1.2 -qps=2000 -duration=10s
curl -s http://localhost:8080/stats | jq

# 4c. ZIPF heavily skewed (s=2.0 — closer to viral URLs)
# Restart again:
go run ./cmd/shortener -cache=lru -cache-size=500
go run ./bench/loadgen -seed=10000
go run ./bench/loadgen -dist=zipf -zipf-s=2.0 -qps=2000 -duration=10s
curl -s http://localhost:8080/stats | jq
```

**What to watch — the headline numbers**:

| Distribution | Cache hit rate (cap=500, ids=10000) |
|---|---|
| Uniform | ~5% (close to capacity / id-count ratio) |
| Zipf s=1.2 | ~70-85% |
| Zipf s=2.0 | ~95%+ |

**Why this is the most important experiment in the lab**:

Look at the gap between 5% and 95%. That gap is the difference between "the CDN-first architecture works" and "the architecture is a 100× waste of money." The world *happens* to be Zipfian for URL traffic, news consumption, video views, etc. Power-law distributions are the load-bearing assumption underneath every CDN-shaped design.

**What the note says**: this is the assumption the note states in one line — *"hot 5% of URLs serve 95% of redirects, Pareto"* — and never re-examines. You just verified the assumption holds (with Zipf) and showed what happens when it doesn't (uniform). When an interviewer asks "what if traffic were uniform?" you now have an answer that doesn't sound recited.

---

## Extensions (optional — these map to the note's "Common follow-ups")

1. **Hot-key Redis under viral load**: bench with `dist=zipf -zipf-s=3.0` (extreme skew) against `-store=redis`. Watch one Redis key absorb most traffic. Maps to the "Hot key" failure mode in Deep dive 2.
2. **Counter recovery after restart**: kill the block-counter server mid-run, restart it. Some IDs are lost (the unused portion of each pod's block). The note says "ID space has 10× headroom; accept it." Confirm with a stats diff.
3. **Implement TTL expiry**: add a `-ttl=1h` flag that sets a Redis TTL on each write. Walk through how a CDN POP's stale-after-takedown window maps to TTL — exactly the "Stale URL after takedown" failure in Deep dive 2.

---

## When you finish

You should now be able to answer all 5 of the H01 note's "Diagnostic questions" without re-reading. If any feel shaky, the experiment that addressed it is the one to re-run — not the note paragraph to re-read.

The note is the *reference*. The lab is where you build the intuition the reference assumes.
