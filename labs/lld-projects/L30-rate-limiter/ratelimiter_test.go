package ratelimiter

import "testing"

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
