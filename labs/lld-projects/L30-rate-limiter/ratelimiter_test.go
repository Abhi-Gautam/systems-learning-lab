package ratelimiter

import (
	"sync"
	"testing"
	"time"
)

func TestRateLimiterAllowsRequestUpToCapacity(t *testing.T) {
	limiter := NewRateLimiter(2, 1)
	if !limiter.Allow("user-1") {
		t.Errorf("expected request to be allowed")
	}
	if !limiter.Allow("user-1") {
		t.Errorf("expected request to be allowed")
	}
	if limiter.Allow("user-1") {
		t.Errorf("expected request to be denied")
	}
}

func TestRateLimiterRefillsTokens(t *testing.T) {
	currentTime := time.Now()

	limiter := newRateLimiter(2, 1, func() time.Time {
		return currentTime
	})

	if !limiter.Allow("user-1") {
		t.Fatal("first request should be allowed")
	}

	if !limiter.Allow("user-1") {
		t.Fatal("second request should be allowed")
	}

	if limiter.Allow("user-1") {
		t.Fatal("third immediate request should be rejected")
	}

	currentTime = currentTime.Add(time.Second)

	if !limiter.Allow("user-1") {
		t.Fatal("request should be allowed after one token refills")
	}
}

func TestRateLimiterRejectsPartialToken(t *testing.T) {
	currentTime := time.Now()

	limiter := newRateLimiter(2, 1, func() time.Time {
		return currentTime
	})

	if !limiter.Allow("user-1") {
		t.Fatal("first request should be allowed")
	}

	if !limiter.Allow("user-1") {
		t.Fatal("second request should be allowed")
	}

	currentTime = currentTime.Add(500 * time.Millisecond)

	if limiter.Allow("user-1") {
		t.Fatal("request should be rejected with only a partial token")
	}

	currentTime = currentTime.Add(500 * time.Millisecond)

	if !limiter.Allow("user-1") {
		t.Fatal("request should be allowed after one full token refills")
	}
}

func TestRateLimiterDoesNotExceedCapacity(t *testing.T) {
	currentTime := time.Now()

	limiter := newRateLimiter(2, 1, func() time.Time {
		return currentTime
	})

	if !limiter.Allow("user-1") {
		t.Fatal("first request should be allowed")
	}

	if !limiter.Allow("user-1") {
		t.Fatal("second request should be allowed")
	}

	currentTime = currentTime.Add(10 * time.Second)

	if !limiter.Allow("user-1") {
		t.Fatal("request should be allowed after refill")
	}

	if !limiter.Allow("user-1") {
		t.Fatal("request should be allowed after refill")
	}

	if limiter.Allow("user-1") {
		t.Fatal("bucket should not contain more than its capacity")
	}
}

func TestRateLimiterTracksUsersIndependently(t *testing.T) {
	currentTime := time.Now()

	limiter := newRateLimiter(2, 1, func() time.Time {
		return currentTime
	})

	if !limiter.Allow("user-1") {
		t.Fatal("user-1 first request should be allowed")
	}

	if !limiter.Allow("user-1") {
		t.Fatal("user-1 second request should be allowed")
	}

	if limiter.Allow("user-1") {
		t.Fatal("user-1 third request should be rejected")
	}

	if !limiter.Allow("user-2") {
		t.Fatal("user-2 first request should be allowed")
	}

	if !limiter.Allow("user-2") {
		t.Fatal("user-2 second request should be allowed")
	}

	if limiter.Allow("user-2") {
		t.Fatal("user-2 third request should be rejected")
	}
}
func TestRateLimiterAllowsAtMostCapacityConcurrently(t *testing.T) {
	currentTime := time.Now()

	limiter := newRateLimiter(10, 1, func() time.Time {
		return currentTime
	})

	const totalRequests = 100

	var wg sync.WaitGroup
	results := make(chan bool, totalRequests)

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
		if result {
			allowed++
		}
	}
	if allowed != 10 {
		t.Fatalf("allowed %d requests, want 10", allowed)
	}
}
