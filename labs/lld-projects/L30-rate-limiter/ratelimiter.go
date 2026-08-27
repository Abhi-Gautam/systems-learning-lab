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
}

type bucket struct {
	tokens     float64
	lastRefill time.Time
}

func NewRateLimiter(capacity, refillRate float64) *RateLimiter {
	return &RateLimiter{
		capacity:   capacity,
		refillRate: refillRate,
		buckets:    make(map[string]*bucket),
	}
}

func (r *RateLimiter) Allow(userID string) bool {
	r.mu.Lock()
	defer r.mu.Unlock()
	b, ok := r.buckets[userID]
	if !ok {
		r.buckets[userID] = &bucket{
			tokens:     r.capacity - 1,
			lastRefill: time.Now(),
		}
		return true
	}
	if b.tokens <= 0 {
		return false
	}
	b.tokens--
	return true
}
