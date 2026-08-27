package retry

import (
	"context"
	"errors"
	"testing"
	"time"
)

type recordingBackoff struct {
	attempts []int
}

func (b *recordingBackoff) Next(attempt int) time.Duration {
	b.attempts = append(b.attempts, attempt)
	return 0
}

func TestDoReturnsResultOnSuccess(t *testing.T) {
	calls := 0

	got, err := Do(
		context.Background(),
		Config{
			Attempts: 3,
		},
		func() (string, error) {
			calls++
			return "ok", nil
		},
	)

	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}

	if got != "ok" {
		t.Fatalf("expected %q, got %q", "ok", got)
	}

	if calls != 1 {
		t.Fatalf("expected 1 call, got %d", calls)
	}
}

func TestDoRetriesUntilSuccess(t *testing.T) {
	calls := 0
	transient := errors.New("transient failure")

	got, err := Do(
		context.Background(),
		Config{
			Attempts: 3,
		},
		func() (string, error) {
			calls++

			if calls < 3 {
				return "", RetryableError(transient)
			}

			return "ok", nil
		},
	)

	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if got != "ok" {
		t.Fatalf("got %q, want %q", got, "ok")
	}

	if calls != 3 {
		t.Fatalf("got %d calls, want 3", calls)
	}
}

func TestDoReturnsErrorAfterAttemptsExhausted(t *testing.T) {
	calls := 0
	wantErr := errors.New("persistent failure")

	_, err := Do(
		context.Background(),
		Config{
			Attempts: 3,
		},
		func() (string, error) {
			calls++
			return "", RetryableError(wantErr)
		},
	)

	if !errors.Is(err, wantErr) {
		t.Fatalf("got %v, want %v", err, wantErr)
	}

	if calls != 3 {
		t.Fatalf("got %d calls, want 3", calls)
	}
}

func TestDoStopsOnTerminalError(t *testing.T) {
	calls := 0
	permanent := errors.New("validation failed")

	_, err := Do(
		context.Background(),
		Config{
			Attempts: 3,
		},
		func() (string, error) {
			calls++
			return "", TerminalError(permanent)
		},
	)

	if !errors.Is(err, permanent) {
		t.Fatalf("got %v, want %v", err, permanent)
	}

	if calls != 1 {
		t.Fatalf("got %d calls, want 1", calls)
	}
}

func TestDoRequestsBackoffBetweenRetries(t *testing.T) {
	calls := 0
	backoff := &recordingBackoff{}
	temporary := errors.New("temporary failure")

	_, err := Do(
		context.Background(),
		Config{
			Attempts: 3,
			Backoff:  backoff,
		},
		func() (string, error) {
			calls++

			if calls < 3 {
				return "", RetryableError(temporary)
			}

			return "ok", nil
		},
	)

	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if len(backoff.attempts) != 2 {
		t.Fatalf("got %d backoff calls, want 2", len(backoff.attempts))
	}

	if backoff.attempts[0] != 1 || backoff.attempts[1] != 2 {
		t.Fatalf("got backoff attempts %v, want [1 2]", backoff.attempts)
	}
}

func TestFixedBackoffReturnsConstantDelay(t *testing.T) {
	backoff := FixedBackoff{
		Delay: 250 * time.Millisecond,
	}

	for attempt := 1; attempt <= 4; attempt++ {
		if got := backoff.Next(attempt); got != 250*time.Millisecond {
			t.Fatalf("attempt %d: got %v, want %v", attempt, got, 250*time.Millisecond)
		}
	}
}

func TestExponentialBackoffDoublesUntilCap(t *testing.T) {
	backoff := ExponentialBackoff{
		Base: 100 * time.Millisecond,
		Max:  500 * time.Millisecond,
	}

	want := []time.Duration{
		100 * time.Millisecond,
		200 * time.Millisecond,
		400 * time.Millisecond,
		500 * time.Millisecond,
		500 * time.Millisecond,
	}

	for attempt, wantDelay := range want {
		got := backoff.Next(attempt + 1)
		if got != wantDelay {
			t.Fatalf("attempt %d: got %v, want %v", attempt+1, got, wantDelay)
		}
	}
}

func TestJitteredBackoffStaysWithinHalfToFullDelay(t *testing.T) {
	backoff := JitteredBackoff{
		Inner: FixedBackoff{
			Delay: 100 * time.Millisecond,
		},
	}

	for attempt := 1; attempt <= 20; attempt++ {
		got := backoff.Next(attempt)

		if got < 50*time.Millisecond || got >= 100*time.Millisecond {
			t.Fatalf("attempt %d: got %v, want [50ms, 100ms)", attempt, got)
		}
	}
}
