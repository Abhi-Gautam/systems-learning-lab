package ratelimiter

import (
	"sync"
	"time"
)

type RateLimiter struct {
	capacity   float64
	refillRate float64
	mu         sync.Mutex
	buckets    map[string]*bucket
	now        func() time.Time
}

type bucket struct {
	tokens     float64
	lastRefill time.Time
}

func NewRateLimiter(capacity, refillRate float64) *RateLimiter {
	return newRateLimiter(capacity, refillRate, time.Now)
}

func newRateLimiter(capacity, refillRate float64, now func() time.Time) *RateLimiter {
	return &RateLimiter{
		capacity:   capacity,
		refillRate: refillRate,
		buckets:    make(map[string]*bucket),
		now:        now,
	}
}

func (r *RateLimiter) Allow(userID string) bool {
	r.mu.Lock()
	defer r.mu.Unlock()
	b, ok := r.buckets[userID]
	if !ok {
		r.buckets[userID] = &bucket{
			tokens:     r.capacity - 1,
			lastRefill: r.now(),
		}
		return true
	}

	now := r.now()
	elapsed := now.Sub(b.lastRefill).Seconds()
	b.tokens += elapsed * r.refillRate

	if b.tokens > r.capacity {
		b.tokens = r.capacity
	}

	b.lastRefill = now

	if b.tokens < 1 {
		return false
	}
	b.tokens--
	return true
}
