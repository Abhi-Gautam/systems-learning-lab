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
