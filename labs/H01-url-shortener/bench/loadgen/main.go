// loadgen — issues GET requests against the shortener with either
// uniform or Zipfian (Pareto-like) ID selection. Reports latency
// distribution and the server's cache hit rate at the end.
//
// Usage:
//
//   # 1. Pre-seed the server with N URLs (so there's something to GET)
//   go run ./bench/loadgen --seed=10000
//
//   # 2. Hammer it
//   go run ./bench/loadgen --dist=zipf    --duration=15s --qps=5000
//   go run ./bench/loadgen --dist=uniform --duration=15s --qps=5000
//
// The same N=10000 set is used both runs. The only thing that changes
// is the SHAPE of the request mix.
package main

import (
	"bytes"
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"log"
	"math/rand"
	"net/http"
	"os"
	"sort"
	"sync"
	"sync/atomic"
	"time"
)

var (
	target    = flag.String("target", "http://localhost:8080", "shortener address")
	dist      = flag.String("dist", "zipf", "request distribution: uniform | zipf")
	zipfS     = flag.Float64("zipf-s", 1.2, "Zipf 's' parameter (>1, higher = more skewed)")
	zipfV     = flag.Float64("zipf-v", 1.0, "Zipf 'v' parameter (≥1)")
	duration  = flag.Duration("duration", 10*time.Second, "bench duration")
	qps       = flag.Int("qps", 1000, "target requests/sec")
	workers   = flag.Int("workers", 64, "concurrent workers")
	seedCount = flag.Int("seed", 0, "if >0: seed this many URLs then exit (no bench)")
	idFile    = flag.String("ids", "/tmp/h01-ids.txt", "file storing the IDs from seed; reused for bench")
)

func main() {
	flag.Parse()
	rand.Seed(time.Now().UnixNano())

	if *seedCount > 0 {
		seed()
		return
	}
	bench()
}

// ---------- seed phase ----------

func seed() {
	log.Printf("seeding %d URLs into %s", *seedCount, *target)
	f, err := os.Create(*idFile)
	if err != nil {
		log.Fatal(err)
	}
	defer f.Close()

	client := &http.Client{Timeout: 5 * time.Second}
	jobs := make(chan int, *workers*2)
	results := make(chan string, *workers*2)
	var wg sync.WaitGroup
	for w := 0; w < *workers; w++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for i := range jobs {
				url := fmt.Sprintf("https://example.com/seed-%d", i)
				body, _ := json.Marshal(map[string]string{"url": url})
				resp, err := client.Post(*target+"/shorten", "application/json", bytes.NewReader(body))
				if err != nil {
					continue
				}
				var out struct{ ID string }
				_ = json.NewDecoder(resp.Body).Decode(&out)
				resp.Body.Close()
				if out.ID != "" {
					results <- out.ID
				}
			}
		}()
	}
	go func() {
		for i := 0; i < *seedCount; i++ {
			jobs <- i
		}
		close(jobs)
		wg.Wait()
		close(results)
	}()
	n := 0
	for id := range results {
		fmt.Fprintln(f, id)
		n++
	}
	log.Printf("seeded %d ids → %s", n, *idFile)
}

// ---------- bench phase ----------

func loadIDs() []string {
	data, err := os.ReadFile(*idFile)
	if err != nil {
		log.Fatalf("read %s (did you run --seed first?): %v", *idFile, err)
	}
	ids := []string{}
	for _, line := range bytes.Split(data, []byte("\n")) {
		s := string(bytes.TrimSpace(line))
		if s != "" {
			ids = append(ids, s)
		}
	}
	if len(ids) == 0 {
		log.Fatal("no ids loaded")
	}
	return ids
}

type sampler func() int

func newSampler(n int) sampler {
	switch *dist {
	case "uniform":
		return func() int { return rand.Intn(n) }
	case "zipf":
		// math/rand Zipf: P(k) ∝ (v+k)^-s for k=0..imax.
		z := rand.NewZipf(rand.New(rand.NewSource(time.Now().UnixNano())), *zipfS, *zipfV, uint64(n-1))
		var mu sync.Mutex
		return func() int {
			mu.Lock()
			defer mu.Unlock()
			return int(z.Uint64())
		}
	default:
		log.Fatalf("unknown dist: %s", *dist)
		return nil
	}
}

type result struct {
	latency time.Duration
	status  int
}

func bench() {
	ids := loadIDs()
	pick := newSampler(len(ids))

	log.Printf("bench: dist=%s qps=%d duration=%s ids=%d workers=%d",
		*dist, *qps, *duration, len(ids), *workers)

	ctx, cancel := context.WithTimeout(context.Background(), *duration)
	defer cancel()

	// Token-bucket-ish: emit a tick per request at the desired rate.
	interval := time.Second / time.Duration(*qps)
	ticks := make(chan struct{}, *workers*4)
	go func() {
		t := time.NewTicker(interval)
		defer t.Stop()
		for {
			select {
			case <-ctx.Done():
				close(ticks)
				return
			case <-t.C:
				select {
				case ticks <- struct{}{}:
				default:
					// drop on overflow → backpressure
				}
			}
		}
	}()

	results := make(chan result, *workers*16)
	var sent atomic.Uint64
	var wg sync.WaitGroup

	for w := 0; w < *workers; w++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			client := &http.Client{
				Timeout: 2 * time.Second,
				CheckRedirect: func(req *http.Request, via []*http.Request) error {
					return http.ErrUseLastResponse
				},
				Transport: &http.Transport{MaxIdleConnsPerHost: 200, MaxConnsPerHost: 200},
			}
			for range ticks {
				id := ids[pick()]
				start := time.Now()
				resp, err := client.Get(*target + "/" + id)
				lat := time.Since(start)
				if err != nil {
					results <- result{lat, 0}
					continue
				}
				_, _ = io.Copy(io.Discard, resp.Body)
				resp.Body.Close()
				results <- result{lat, resp.StatusCode}
				sent.Add(1)
			}
		}()
	}

	go func() {
		wg.Wait()
		close(results)
	}()

	lats := []time.Duration{}
	statusCount := map[int]int{}
	for r := range results {
		lats = append(lats, r.latency)
		statusCount[r.status]++
	}

	report(lats, statusCount)
	fetchServerStats()
}

func report(lats []time.Duration, statusCount map[int]int) {
	if len(lats) == 0 {
		log.Println("no results")
		return
	}
	sort.Slice(lats, func(i, j int) bool { return lats[i] < lats[j] })
	p := func(q float64) time.Duration { return lats[int(float64(len(lats)-1)*q)] }
	fmt.Println()
	fmt.Println("=== latency ===")
	fmt.Printf("count=%d\n", len(lats))
	fmt.Printf("p50 =%8s\n", p(0.50))
	fmt.Printf("p90 =%8s\n", p(0.90))
	fmt.Printf("p99 =%8s\n", p(0.99))
	fmt.Printf("p999=%8s\n", p(0.999))
	fmt.Printf("max =%8s\n", lats[len(lats)-1])
	fmt.Println("=== status ===")
	for s, c := range statusCount {
		fmt.Printf("  %d: %d\n", s, c)
	}
}

func fetchServerStats() {
	resp, err := http.Get(*target + "/stats")
	if err != nil {
		return
	}
	defer resp.Body.Close()
	var out map[string]any
	if err := json.NewDecoder(resp.Body).Decode(&out); err != nil {
		return
	}
	b, _ := json.MarshalIndent(out, "", "  ")
	fmt.Println("=== server stats ===")
	fmt.Println(string(b))
}
