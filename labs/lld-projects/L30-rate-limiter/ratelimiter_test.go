package ratelimiter

import (
	"sync"
	"testing"
	"time"
)

func TestPublicLimitersExposeCommonContract(t *testing.T) {
	var tokenBucket Limiter = NewTokenBucketLimiter(2, 1)
	var fixedWindow Limiter = NewFixedWindowLimiter(2, time.Second)

	if tokenBucket == nil {
		t.Fatal("token-bucket limiter should not be nil")
	}

	if fixedWindow == nil {
		t.Fatal("fixed-window limiter should not be nil")
	}
}

func TestRateLimiterAllowsRequestUpToCapacity(t *testing.T) {
	limiter := newTokenBucketLimiter(2, 1, time.Now)
	if !limiter.Allow("user-1").Allowed {
		t.Errorf("expected request to be allowed")
	}
	if !limiter.Allow("user-1").Allowed {
		t.Errorf("expected request to be allowed")
	}
	if limiter.Allow("user-1").Allowed {
		t.Errorf("expected request to be denied")
	}
}

func TestRateLimiterRefillsTokens(t *testing.T) {
	currentTime := time.Now()

	limiter := newTokenBucketLimiter(2, 1, func() time.Time {
		return currentTime
	})

	if !limiter.Allow("user-1").Allowed {
		t.Fatal("first request should be allowed")
	}

	if !limiter.Allow("user-1").Allowed {
		t.Fatal("second request should be allowed")
	}

	if limiter.Allow("user-1").Allowed {
		t.Fatal("third immediate request should be rejected")
	}

	currentTime = currentTime.Add(time.Second)

	if !limiter.Allow("user-1").Allowed {
		t.Fatal("request should be allowed after one token refills")
	}
}

func TestRateLimiterRejectsPartialToken(t *testing.T) {
	currentTime := time.Now()

	limiter := newTokenBucketLimiter(2, 1, func() time.Time {
		return currentTime
	})

	if !limiter.Allow("user-1").Allowed {
		t.Fatal("first request should be allowed")
	}

	if !limiter.Allow("user-1").Allowed {
		t.Fatal("second request should be allowed")
	}

	currentTime = currentTime.Add(500 * time.Millisecond)

	if limiter.Allow("user-1").Allowed {
		t.Fatal("request should be rejected with only a partial token")
	}

	currentTime = currentTime.Add(500 * time.Millisecond)

	if !limiter.Allow("user-1").Allowed {
		t.Fatal("request should be allowed after one full token refills")
	}
}

func TestRateLimiterDoesNotExceedCapacity(t *testing.T) {
	currentTime := time.Now()

	limiter := newTokenBucketLimiter(2, 1, func() time.Time {
		return currentTime
	})

	if !limiter.Allow("user-1").Allowed {
		t.Fatal("first request should be allowed")
	}

	if !limiter.Allow("user-1").Allowed {
		t.Fatal("second request should be allowed")
	}

	currentTime = currentTime.Add(10 * time.Second)

	if !limiter.Allow("user-1").Allowed {
		t.Fatal("request should be allowed after refill")
	}

	if !limiter.Allow("user-1").Allowed {
		t.Fatal("request should be allowed after refill")
	}

	if limiter.Allow("user-1").Allowed {
		t.Fatal("bucket should not contain more than its capacity")
	}
}

func TestRateLimiterTracksUsersIndependently(t *testing.T) {
	currentTime := time.Now()

	limiter := newTokenBucketLimiter(2, 1, func() time.Time {
		return currentTime
	})

	if !limiter.Allow("user-1").Allowed {
		t.Fatal("user-1 first request should be allowed")
	}

	if !limiter.Allow("user-1").Allowed {
		t.Fatal("user-1 second request should be allowed")
	}

	if limiter.Allow("user-1").Allowed {
		t.Fatal("user-1 third request should be rejected")
	}

	if !limiter.Allow("user-2").Allowed {
		t.Fatal("user-2 first request should be allowed")
	}

	if !limiter.Allow("user-2").Allowed {
		t.Fatal("user-2 second request should be allowed")
	}

	if limiter.Allow("user-2").Allowed {
		t.Fatal("user-2 third request should be rejected")
	}
}

func TestRateLimiterAllowsAtMostCapacityConcurrently(t *testing.T) {
	currentTime := time.Now()

	limiter := newTokenBucketLimiter(10, 1, func() time.Time {
		return currentTime
	})

	const totalRequests = 100

	var wg sync.WaitGroup
	results := make(chan Decision, totalRequests)

	wg.Add(totalRequests)

	for i := 0; i < totalRequests; i++ {
		go func() {
			defer wg.Done()
			results <- limiter.Allow("user-1")
		}()
	}

	wg.Wait()
	close(results)

	allowed := 0
	for result := range results {
		if result.Allowed {
			allowed++
		}
	}

	if allowed != 10 {
		t.Fatalf("allowed %d requests, want 10", allowed)
	}
}

func TestRateLimiterReturnsRetryAfter(t *testing.T) {
	currentTime := time.Now()

	limiter := newTokenBucketLimiter(2, 1, func() time.Time {
		return currentTime
	})

	limiter.Allow("user-1")
	limiter.Allow("user-1")

	decision := limiter.Allow("user-1")

	if decision.Allowed {
		t.Fatal("third request should be rejected")
	}

	if decision.RetryAfter != time.Second {
		t.Fatalf("got retry after %v, want %v", decision.RetryAfter, time.Second)
	}
}

func TestFixedWindowLimiterResetsAtWindowBoundary(t *testing.T) {
	currentTime := time.Now()

	limiter := newFixedWindowLimiter(
		2,
		time.Second,
		func() time.Time {
			return currentTime
		},
	)

	if !limiter.Allow("user-1").Allowed {
		t.Fatal("first request should be allowed")
	}

	if !limiter.Allow("user-1").Allowed {
		t.Fatal("second request should be allowed")
	}

	decision := limiter.Allow("user-1")

	if decision.Allowed {
		t.Fatal("third request in the same window should be rejected")
	}

	if decision.RetryAfter != time.Second {
		t.Fatalf(
			"got retry after %v, want %v",
			decision.RetryAfter,
			time.Second,
		)
	}

	currentTime = currentTime.Add(time.Second)

	if !limiter.Allow("user-1").Allowed {
		t.Fatal("request should be allowed in the next window")
	}
}

func TestRateLimiterCreatesOneStatePerUser(t *testing.T) {
	currentTime := time.Now()
	created := 0

	limiter := newRateLimiter(
		func(now time.Time) userState {
			created++
			return newTokenBucket(1, 1, now)
		},
		func() time.Time {
			return currentTime
		},
	)

	limiter.Allow("user-1")
	limiter.Allow("user-1")
	limiter.Allow("user-2")

	if created != 2 {
		t.Fatalf("created %d states, want 2", created)
	}

	if len(limiter.states) != 2 {
		t.Fatalf("stored %d user states, want 2", len(limiter.states))
	}

	if _, ok := limiter.states["user-1"].(*tokenBucket); !ok {
		t.Fatalf("user-1 state has type %T, want *tokenBucket", limiter.states["user-1"])
	}
}
