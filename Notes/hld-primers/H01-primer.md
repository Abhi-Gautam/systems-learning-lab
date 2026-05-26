# H01 Primer — URL shortener

_Read this BEFORE the `notes/hld-notes.md` entry for H01. Then run the lab at `labs/H01-url-shortener/`. The note assumes you've done both._

---

## What this problem assumes you already know

| Concept | Where it shows up in the H01 note | One-line definition |
|---|---|---|
| **base62 encoding** | "ID generation", everywhere `aB3xK9z` appears | Writing an integer in a base where digits are `0-9a-zA-Z` (62 symbols) |
| **Sharding** | "Postgres × 32 shards", "counter per shard" | Splitting one logical table across N physical databases by a key range/hash |
| **Pareto distribution** | "5% of URLs serve 95% of redirects" | A statistical pattern where a small fraction of items account for most of the activity |
| **CDN edge / POP** | "CDN absorbs 95%", "200+ POPs" | A small server geographically close to the user that stores recent HTTP responses |
| **p99 latency** | "p99 < 100ms" — the headline NFR | The latency value that 99% of requests come in under (the slow-tail metric) |

If all 5 rows feel obvious — skip ahead to the lab. If 2+ feel fuzzy — read those sections below.

---

## Concept 1: base62 encoding

You already know base10 (digits `0-9`) and base16 (digits `0-9a-f`). Base62 just extends the alphabet: digits `0-9`, lowercase `a-z`, uppercase `A-Z` → 62 symbols total.

To convert an integer to base62: divide-and-remainder, same as you'd do for base10 → binary.

```
Convert 14_276_804_201 to base62:

  14_276_804_201 ÷ 62 = 230_271_035  remainder 31  → digit 'V'  (62-symbol alphabet 0..9 a..z A..Z; 31 = 'V')
       230_271_035 ÷ 62 =   3_714_048  remainder  59  → digit 'x'
         3_714_048 ÷ 62 =      59_904  remainder   0  → digit '0'
            59_904 ÷ 62 =         966  remainder  12  → digit 'c'
               966 ÷ 62 =          15  remainder  36  → digit 'A'
                15 ÷ 62 =           0  remainder  15  → digit 'f'

Reading remainders bottom-to-top:  "fAc0xV"  → 6 chars for ~14 billion.
```
> **Why it matters for H01**: 62⁷ = ~3.5 trillion. That's enough unique IDs for 365 billion URLs (10 years × 100M/day) with 10× headroom — at **7 characters**. Base10 of the same integer is 9 digits; hex is 9 digits. Base62 is the densest URL-safe encoding (URL-safe = no `+ / ? #` chars that need escaping).

```go
// 10-line base62 encoder
const alphabet = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"

func base62(n uint64) string {
    if n == 0 { return "0" }
    var out []byte
    for n > 0 {
        out = append([]byte{alphabet[n%62]}, out...)
        n /= 62
    }
    return string(out)
}
```

---

## Concept 2: sharding

One database can hold maybe 10 TB and handle maybe 10k writes/sec. H01 needs 183 TB and 3k writes/sec peak — fits writes, blows storage. So you **shard**: split the logical `links` table across N physical databases.

Two common ways to split:

| Strategy | How keys are routed | Pros | Cons |
|---|---|---|---|
| **Hash sharding** | `shard = hash(id) % N` | Even distribution | Range queries explode (scan all shards) |
| **Range sharding** | `id 0..1B → shard 0; 1B..2B → shard 1; ...` | Range scans cheap | Hot spots if writes cluster in one range |

H01 uses range sharding by `id`: shard 0 owns IDs `0..N`, shard 1 owns `N..2N`, etc. Each shard has its own counter row that hands out IDs only within its own range, so **two shards can never mint the same ID** by construction. No coordination needed.

```
links table — logically one table
       │
       ▼ shard router (in API code, not in DB)
┌──────────────┬──────────────┬─────┬──────────────┐
│ Postgres #0  │ Postgres #1  │ ... │ Postgres #31 │
│ IDs 0..N     │ IDs N..2N    │     │ IDs 31N..32N │
│ counter row  │ counter row  │     │ counter row  │
└──────────────┴──────────────┴─────┴──────────────┘
```

**The thing to internalize**: a shard is just another database. Nothing magical. "32 shards" means "32 separate Postgres instances and the application code knows which one to ask."

---

## Concept 3: Pareto distribution (the 80/20 rule, formally)

In a Pareto distribution, a small fraction of items account for most of the observations. The most-quoted form is "80% of effects come from 20% of causes." URL traffic is way more extreme: **about 5% of URLs serve 95% of the redirects.**

```
URL popularity (Pareto α≈1, log-log scale)

QPS │█
    │█
    │██
    │███
    │███████
    │██████████████████████████████████████████████
    └────────────────────────────────────────────────►
       ^                                              ^
     viral hits                              long tail of obscure URLs
     (5% of URLs)                            (95% of URLs)
```

**Why it matters for H01**: this distribution is what makes a *small* cache do *huge* work. A CDN that caches just the top 1M URLs (out of billions) absorbs 95%+ of all redirects. Without Pareto, the cache would have to be 100× bigger to get the same hit rate — and the whole CDN-first architecture would collapse.

The note's "95/4/1 split" (95% CDN, 4% Redis, 1% DB) is **a direct claim about Pareto holding in URL traffic**. If users redirected to random URLs uniformly, the cache wouldn't help and the design would fail.

---

## Concept 4: what a CDN edge actually stores

A common confusion: "the CDN caches the URL." It doesn't. It caches the **HTTP response**.

Walk through one redirect:

```
Browser    ────GET /aB3xK9z────►    CDN POP (Cloudflare, in Mumbai)
                                    │
                                    │ checks its disk cache, key = "/aB3xK9z"
                                    │
                       ┌────────────┴───────────┐
                       │                        │
                  HIT (95%)                 MISS (5%)
                       │                        │
                       ▼                        ▼
        respond with cached bytes:    forward to origin in US
        HTTP/1.1 301                  origin returns 301 + Location
        Location: https://...         POP stores those bytes
        Cache-Control: max-age=1y     POP returns to browser
        (Latency: 5-20ms)             (Latency: 80-150ms — cross-region)
```

What's on the POP's disk: a key (`/aB3xK9z`) → a small blob of bytes (the full HTTP response: status line + headers). When the next user in Mumbai hits `/aB3xK9z`, the POP serves those exact bytes back without touching the origin.

**Implications baked into H01**:
- A new short URL is **slow on first hit** in every region until that POP has seen it (cache warmup).
- A taken-down URL **stays alive in the cache** until TTL expires OR until you call the CDN's purge API. That's the "stale URL after takedown" failure mode in the note.

---

## Concept 5: p99 latency

If 1000 requests are served and you sort their response times, the **p99** is the response time at position 990 (the 99th percentile). 99% of requests finish at or below that time; 1% take longer.

```
sorted latencies (ms):  3 4 4 5 5 6 ... 7 8 9 12 47 89 210
                                            ▲        ▲
                                           p50     p99
                                          (~7ms)  (~89ms)
```

**Why use p99 instead of average?** Averages hide the slow tail. A service with avg=10ms but p99=500ms is a bad service — 1% of requests is a LOT at scale. At 10B redirects/day, p99=500ms means **100 million slow requests per day**. p99 captures user-visible slowness; average hides it.

**For H01**: the NFR is `p99 < 100ms over the network`. That budget has to cover network RTT + TLS + processing + DB. A cross-region DB call alone burns 80ms. **The 3-tier cache exists specifically so that ≤1% of requests ever take the slow DB path** — keeping p99 inside the budget.

This is why the note says "p99 of the *mix*" — 95% of requests at 10ms (CDN), 4% at 30ms (Redis), 1% at 90ms (DB), gives a mixture whose 99th percentile is dominated by the DB path. If the DB path were 200ms, p99 would blow.

---

## Checkpoint — answer these without re-reading

1. Why is `srt.ly/aB3xK9z` exactly 7 characters and not 6 or 8?
2. If shard 5's Postgres dies, can shard 6 mint IDs that shard 5 should have minted? Why or why not?
3. A user in Tokyo requests a freshly-created URL. The closest CDN POP has never seen it. What happens, in order?
4. If URL popularity were *uniform* (every URL equally likely) instead of Pareto, which tier of the cache would still work?
5. Avg latency = 20ms, p99 = 800ms. Service is healthy or sick? Why?

If you can answer all 5 → open `labs/H01-url-shortener/EXPERIMENTS.md`.
If not → re-read the concept you stumbled on. The H01 note will collapse into nonsense without these five.
