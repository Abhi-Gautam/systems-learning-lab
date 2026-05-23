# Engineering Blogs (source of production numbers + war stories)

Tier-S engineering blogs cited by `/design-today` notes. Used to ground "Where this shows up in production" sections with **named systems and named numbers** — not "many companies do this."

These blogs are URLs the user opens in a browser if they want the raw post. The composer cites the company + post title + specific number/decision; the URL is for verification.

---

## Storage / KV / messaging

### Discord — How Discord stores billions of messages
**URL**: https://discord.com/blog/how-discord-stores-billions-of-messages
**Powers**: H03 WhatsApp · H17 DynamoDB · H22 Slack
**Key numbers**: 8B messages stored; ~120M new/day; chose Cassandra in 2016.
**Key insight**: Compound partition key `(channel_id, time_bucket)` to bound partition size.

### Discord — How Discord migrated from Cassandra to ScyllaDB (2022)
**URL**: https://discord.com/blog/how-discord-stores-trillions-of-messages
**Powers**: H17 DynamoDB · H03 WhatsApp
**Key numbers**: 177 Cassandra nodes → 72 ScyllaDB nodes; trillions of messages; JVM GC root cause for migration.
**Key insight**: Hint queue depth during failover caused GC pauses; per-shard hint queues in ScyllaDB fixed it.

### Stripe — Designing robust and predictable APIs with idempotency
**URL**: https://stripe.com/blog/idempotency
**Powers**: H14 Stripe payment · L36 idempotency-key middleware
**Key insight**: Idempotency-key middleware sits at API edge; stores full request fingerprint + response; 24h TTL.

### Stripe — Online migrations at scale
**URL**: https://stripe.com/blog/online-migrations
**Powers**: H14 Stripe payment · DDIA-adjacent discussions
**Key insight**: 4-step migration: dual-write → backfill → dual-read → cutover; verification framework runs in shadow.

### Werner Vogels — 10 lessons from a decade of DynamoDB
**URL**: https://www.allthingsdistributed.com/2017/01/amazon-dynamodb-10-years.html
**Powers**: H17 DynamoDB
**Key insight**: Adaptive capacity rebalances partition heat live; isolation of hot keys is auto-magic in DDB but not in self-hosted Dynamo derivatives.

---

## Caching / CDN

### Netflix — EVCache, distributed cache at Netflix
**URL**: https://netflixtechblog.com/caching-for-a-global-netflix-1471f6e4bd8d
**Powers**: H09 distributed cache · H08 Instagram feed
**Key numbers**: 30+ trillion ops/day; multi-region replication via Kafka.
**Key insight**: Multi-region cache replication is a *replicated stream of invalidations*, not a replicated store.

### Cloudflare — How we built rate limiting capable of scaling to millions of domains
**URL**: https://blog.cloudflare.com/counting-things-a-lot-of-different-things/
**Powers**: H10 rate limiter
**Key insight**: Sliding window approximation = (current_window_count * (1 - overlap_fraction)) + (prev_window_count * overlap_fraction); avoids storing per-request timestamps.

### Cloudflare — R2 object storage
**URL**: https://blog.cloudflare.com/r2-open-beta/
**Powers**: H07 Dropbox · H18 GFS · H05 Netflix CDN
**Key insight**: Egress-free object storage; zero-trust between datacenters via per-bucket signing.

---

## ML / LLM serving

### Anthropic — Engineering challenges of serving Claude
**URL**: https://www.anthropic.com/news (search for serving infrastructure posts)
**Powers**: H26 LLM serving · H27 RAG · H28 prompt cache
**Key insight**: Long-context prompts (100k+ tokens) require fundamentally different routing than short prompts — separate fleets.

### Anthropic — Contextual retrieval
**URL**: https://www.anthropic.com/news/contextual-retrieval
**Powers**: H27 RAG
**Key insight**: Prepending chunk-context (the surrounding section's gist) to each chunk before embedding lifts retrieval accuracy 35-49%.

### Character.AI — Optimizing inference
**URL**: https://research.character.ai/optimizing-inference/
**Powers**: H26 LLM serving
**Key insight**: Multi-Query Attention (MQA) + custom KV cache eviction = 33% cost reduction; query batching across users.

---

## Observability / monitoring

### Datadog — Husky, a TSDB for event observability
**URL**: https://www.datadoghq.com/blog/engineering/introducing-husky/
**Powers**: H31 TSDB · H24 log analytics
**Key numbers**: PB/day ingestion; Parquet-on-S3 storage.
**Key insight**: Decouple ingestion from query — ingestion writes Parquet, query engine reads Parquet; metadata layer pins the schema.

### Honeycomb — Refinery and tail sampling
**URL**: https://docs.honeycomb.io/get-started/best-practices/sampling/
**Powers**: H32 distributed tracing
**Key insight**: Tail sampling requires holding entire trace in memory until decision; refinery is a proxy that aggregates spans before the sampling decision.

### Honeycomb — Engineering high-cardinality observability
**URL**: https://www.honeycomb.io/blog/treating-data-events-not-metrics-will-change-your-world
**Powers**: H31 TSDB · H32 tracing
**Key insight**: Wide events > pre-aggregated metrics; the cardinality problem is solved at storage by columnar Parquet, not by reducing label dimensions.

---

## Real-time / messaging

### LinkedIn — Feed Mixer architecture
**URL**: https://engineering.linkedin.com/blog/2016/04/architecture-of-linkedin-s-real-time-distributed-graph
**Powers**: H02 Twitter timeline · H08 Instagram feed
**Key insight**: Hybrid fan-out — push for normal users, pull for celebrities (>1M followers); decision made per-post.

### WhatsApp — 1 million concurrent connections per server (FreeBSD + Erlang)
**URL**: archived; Rick Reed's High Scalability post
**Powers**: H03 WhatsApp
**Key insight**: BEAM VM's lightweight processes (each = ~2KB) make millions of idle TCP connections per server feasible.

---

## Databases / SQL engines

### Databricks — How we built Delta Lake
**URL**: https://www.databricks.com/blog/2019/04/24/delta-lake-genesis.html
**Powers**: H36 Delta Lake · H35 query optimizer
**Key insight**: ACID on top of object storage via a transaction log (JSON files in `_delta_log/`); checkpoints every 10 commits compact the log.

### CockroachDB — Living without atomic clocks
**URL**: https://www.cockroachlabs.com/blog/living-without-atomic-clocks/
**Powers**: H39 Spanner
**Key insight**: HLC (hybrid logical clock) is a software substitute for TrueTime; uncertainty bound is larger and varies, but no GPS+atomic clock hardware needed.

---

## Adding a blog

1. Add an entry under the relevant category.
2. Include: company name, post title, URL, `**Powers**:` (which problems), `**Key numbers**:` (if any), `**Key insight**:` (the one thing this blog teaches that papers don't).
3. Prefer blogs ≥3 years old (still relevant) or very recent (last 12 months) for current state.
