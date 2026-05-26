package main

import (
	"context"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	"h01/internal"
)

// CLI flags — toggling these is the whole point of the lab.
var (
	addr        = flag.String("addr", ":8080", "listen address")
	idMode      = flag.String("id-mode", "block", "id generation: naive | sharded | block | hash")
	shards      = flag.Int("shards", 4, "shard count for sharded/block modes")
	blockSize   = flag.Uint64("block-size", 1000, "ID block size for block mode")
	hashWidth   = flag.Uint("hash-width", 4, "base62 chars to keep for hash mode (smaller = more collisions)")
	storeMode   = flag.String("store", "mem", "store backend: mem | redis")
	redisAddr   = flag.String("redis", "localhost:6379", "redis address (when store=redis)")
	cacheMode   = flag.String("cache", "none", "read cache: none | lru | redis")
	cacheSize   = flag.Int("cache-size", 10000, "LRU cache size when cache=lru")
	statsEvery  = flag.Duration("stats-every", 5*time.Second, "print stats every interval (0 = off)")
	verbose     = flag.Bool("verbose", false, "trace each request step-by-step through counter/store/cache")
)

type server struct {
	counter   internal.Counter   // for naive/sharded/block
	hash      *internal.HashIDGen // for hash mode
	store     internal.Store
	cache     *internal.LRUCache // optional read-path cache (nil = no L1)
	rcache    *internal.RedisStore // optional Redis L2 (nil if not used)
	postCount uint64
	getCount  uint64
}

type shortenRequest struct {
	URL string `json:"url"`
}

type shortenResponse struct {
	ID       string `json:"id"`
	ShortURL string `json:"short_url"`
}

func (s *server) handleShorten(w http.ResponseWriter, r *http.Request) {
	var req shortenRequest
	body, _ := io.ReadAll(r.Body)
	if err := json.Unmarshal(body, &req); err != nil || req.URL == "" {
		http.Error(w, "bad body", http.StatusBadRequest)
		return
	}
	if *verbose {
		log.Printf("=== POST /shorten received: url=%q (mode=%s) ===", req.URL, *idMode)
	}

	var id string
	var err error

	if *idMode == "hash" {
		// Hash mode: same URL → same ID (idempotent by construction).
		// Collisions happen when DIFFERENT URLs hash into the same slot.
		// We retry once with a salt; in real life you'd loop with backoff.
		for attempt := 0; attempt < 3; attempt++ {
			salted := req.URL
			if attempt > 0 {
				salted = fmt.Sprintf("%s#salt=%d", req.URL, attempt)
			}
			n := s.hash.NextFor(salted)
			id = internal.Base62(n)
			// Left-pad with '0' so all IDs in hash mode have the same width.
			for len(id) < int(*hashWidth) {
				id = "0" + id
			}
			err = s.store.PutIfAbsent(r.Context(), id, req.URL)
			if err == nil {
				break
			}
			if errors.Is(err, internal.ErrCollision) {
				s.hash.RecordCollision()
				continue
			}
			break
		}
	} else {
		if *verbose {
			log.Printf("STEP 1/4: calling %s counter.Next()...", s.counter.Name())
		}
		n := s.counter.Next()
		if *verbose {
			log.Printf("STEP 2/4: counter returned uint64=%d. base62-encoding...", n)
		}
		id = internal.Base62(n)
		if *verbose {
			log.Printf("STEP 2/4 done: id=%q (length %d)", id, len(id))
			log.Printf("STEP 3/4: store.PutIfAbsent(%q -> %q)...", id, req.URL)
		}
		err = s.store.PutIfAbsent(r.Context(), id, req.URL)
		if *verbose {
			if err == nil {
				log.Printf("STEP 3/4 done: stored OK.")
			} else {
				log.Printf("STEP 3/4 FAILED: %v", err)
			}
		}
	}

	if err != nil {
		http.Error(w, err.Error(), http.StatusConflict)
		return
	}
	if s.cache != nil {
		if *verbose {
			log.Printf("STEP 4/4: cache.Put(%q -> %q) (warming the LRU)", id, req.URL)
		}
		s.cache.Put(id, req.URL)
	} else if *verbose {
		log.Printf("STEP 4/4: no cache configured, skipping warm.")
	}
	if *verbose {
		log.Printf("=== POST /shorten done: short_url=http://srt.ly/%s ===", id)
	}
	resp := shortenResponse{ID: id, ShortURL: "http://srt.ly/" + id}
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(resp)
}

func (s *server) handleRedirect(w http.ResponseWriter, r *http.Request) {
	id := strings.TrimPrefix(r.URL.Path, "/")
	if id == "" || id == "shorten" {
		http.NotFound(w, r)
		return
	}
	if *verbose {
		log.Printf("=== GET /%s received ===", id)
	}

	// L1: in-process LRU
	if s.cache != nil {
		if *verbose {
			log.Printf("STEP 1/2: cache.Get(%q)...", id)
		}
		if url, ok := s.cache.Get(id); ok {
			if *verbose {
				log.Printf("STEP 1/2: CACHE HIT — skipping store entirely. redirecting to %q.", url)
				log.Printf("=== GET /%s done (served from cache) ===", id)
			}
			redirect(w, url)
			return
		}
		if *verbose {
			log.Printf("STEP 1/2: cache MISS — falling through to store.")
		}
	}
	// L2: origin store (mem or redis)
	if *verbose {
		log.Printf("STEP 2/2: store.Get(%q)...", id)
	}
	url, err := s.store.Get(r.Context(), id)
	if err != nil {
		if *verbose {
			log.Printf("STEP 2/2: store miss — 404. err=%v", err)
		}
		http.NotFound(w, r)
		return
	}
	if *verbose {
		log.Printf("STEP 2/2: store HIT — %q. warming cache, then redirecting.", url)
	}
	if s.cache != nil {
		s.cache.Put(id, url)
	}
	if *verbose {
		log.Printf("=== GET /%s done (served from store, cache now warmed) ===", id)
	}
	redirect(w, url)
}

func redirect(w http.ResponseWriter, url string) {
	w.Header().Set("Location", url)
	w.Header().Set("Cache-Control", "public, max-age=31536000")
	w.WriteHeader(http.StatusMovedPermanently)
}

func (s *server) handleStats(w http.ResponseWriter, _ *http.Request) {
	out := map[string]any{
		"id_mode":    *idMode,
		"store":      *storeMode,
		"cache_mode": *cacheMode,
	}
	if s.cache != nil {
		h, m := s.cache.Stats()
		total := h + m
		rate := 0.0
		if total > 0 {
			rate = float64(h) / float64(total)
		}
		out["cache_hits"] = h
		out["cache_misses"] = m
		out["cache_hit_rate"] = rate
	}
	if s.hash != nil {
		req, col := s.hash.Stats()
		out["hash_requests"] = req
		out["hash_collisions"] = col
		if req > 0 {
			out["hash_collision_rate"] = float64(col) / float64(req)
		}
	}
	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(out)
}

func build() *server {
	s := &server{}

	switch *idMode {
	case "naive":
		s.counter = internal.NewNaiveCounter(1_000_000)
	case "sharded":
		s.counter = internal.NewShardedCounter(*shards, 1_000_000)
	case "block":
		s.counter = internal.NewBlockCounter(*shards, 1_000_000, *blockSize)
	case "hash":
		s.hash = internal.NewHashIDGen(*hashWidth)
	default:
		log.Fatalf("unknown id-mode: %s", *idMode)
	}

	switch *storeMode {
	case "mem":
		s.store = internal.NewMemStore()
	case "redis":
		s.store = internal.NewRedisStore(*redisAddr)
	default:
		log.Fatalf("unknown store: %s", *storeMode)
	}

	switch *cacheMode {
	case "none":
		// no L1
	case "lru":
		s.cache = internal.NewLRUCache(*cacheSize)
	case "redis":
		// Treat Redis itself as the cache tier; nothing to do beyond store.
	default:
		log.Fatalf("unknown cache: %s", *cacheMode)
	}

	return s
}

func main() {
	flag.Parse()
	internal.Verbose = *verbose
	s := build()

	mux := http.NewServeMux()
	mux.HandleFunc("POST /shorten", s.handleShorten)
	mux.HandleFunc("GET /stats", s.handleStats)
	mux.HandleFunc("GET /", s.handleRedirect)

	srv := &http.Server{Addr: *addr, Handler: mux, ReadTimeout: 5 * time.Second}

	log.Printf("h01 listening on %s · id-mode=%s shards=%d block=%d store=%s cache=%s",
		*addr, *idMode, *shards, *blockSize, *storeMode, *cacheMode)

	if *statsEvery > 0 {
		go func() {
			t := time.NewTicker(*statsEvery)
			defer t.Stop()
			for range t.C {
				printStats(s)
			}
		}()
	}

	go func() {
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatal(err)
		}
	}()

	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt, syscall.SIGTERM)
	<-stop
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	_ = srv.Shutdown(ctx)
	printStats(s)
}

func printStats(s *server) {
	parts := []string{fmt.Sprintf("id=%s", *idMode)}
	if s.cache != nil {
		h, m := s.cache.Stats()
		total := h + m
		rate := 0.0
		if total > 0 {
			rate = float64(h) / float64(total)
		}
		parts = append(parts, fmt.Sprintf("cache=%.1f%% (%d/%d)", rate*100, h, total))
	}
	if s.hash != nil {
		req, col := s.hash.Stats()
		rate := 0.0
		if req > 0 {
			rate = float64(col) / float64(req)
		}
		parts = append(parts, fmt.Sprintf("collisions=%.2f%% (%d/%d)", rate*100, col, req))
	}
	log.Println("stats · " + strings.Join(parts, " · "))
}
