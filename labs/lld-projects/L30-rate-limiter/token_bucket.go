package ratelimiter

import (
	"math"
	"time"
)

type TokenBucketLimiter struct {
	core *rateLimiter
}

// NewTokenBucketLimiter creates a token-bucket rate limiter.
func NewTokenBucketLimiter(
	capacity, refillRate float64,
) *TokenBucketLimiter {
	return newTokenBucketLimiter(capacity, refillRate, time.Now)
}

func newTokenBucketLimiter(
	capacity, refillRate float64,
	now func() time.Time,
) *TokenBucketLimiter {
	return &TokenBucketLimiter{
		core: newRateLimiter(
			func(createdAt time.Time) userState {
				return newTokenBucket(capacity, refillRate, createdAt)
			},
			now,
		),
	}
}

func (r *TokenBucketLimiter) Allow(userID string) Decision {
	return r.core.Allow(userID)
}

// tokenBucket contains the state and behavior for token-bucket limiting.
// Its implementation will move here during the architecture cleanup.

type tokenBucket struct {
	capacity   float64
	refillRate float64
	tokens     float64
	lastRefill time.Time
}

func newTokenBucket(capacity, refillRate float64, now time.Time) *tokenBucket {
	return &tokenBucket{
		capacity:   capacity,
		refillRate: refillRate,
		tokens:     capacity,
		lastRefill: now,
	}
}

func (b *tokenBucket) allow(now time.Time) Decision {
	elapsed := now.Sub(b.lastRefill).Seconds()
	b.tokens += elapsed * b.refillRate

	if b.tokens > b.capacity {
		b.tokens = b.capacity
	}

	b.lastRefill = now

	if b.tokens < 1 {
		missingTokens := 1 - b.tokens
		secondsToWait := missingTokens / b.refillRate

		retryAfter := time.Duration(
			math.Ceil(secondsToWait * float64(time.Second)),
		)
		return Decision{Allowed: false, RetryAfter: retryAfter}
	}
	b.tokens--
	return Decision{Allowed: true}
}
