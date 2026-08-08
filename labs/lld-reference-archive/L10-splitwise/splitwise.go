package splitwise

import (
	"context"
	"time"
)

type (
	UserID    string
	GroupID   string
	ExpenseID string
)

type Pair struct{ From, To UserID }

type SplitStrategy interface {
	Shares(amount int64, participants []UserID) (map[UserID]int64, error)
}

type EqualSplit struct{}

func (EqualSplit) Shares(amt int64, ps []UserID) (map[UserID]int64, error) {
	out := make(map[UserID]int64, len(ps))
	per := amt / int64(len(ps))
	res := amt - per*int64(len(ps))
	for _, p := range ps {
		out[p] = per
	}
	out[ps[0]] += res
	return out, nil
}

type PercentageSplit struct{ Bps map[UserID]int64 }

func (p PercentageSplit) Shares(amt int64, _ []UserID) (map[UserID]int64, error) {
	return nil, nil
}

type ExactSplit struct{ Cents map[UserID]int64 }

func (e ExactSplit) Shares(amt int64, _ []UserID) (map[UserID]int64, error) {
	return nil, nil
}

type Expense struct {
	ID           ExpenseID
	GroupID      GroupID
	Payer        UserID
	Participants []UserID
	AmountCents  int64
	Currency     string
	Split        SplitStrategy
	ReversalOf   *ExpenseID
	At           time.Time
}

type Transfer struct {
	From, To UserID
	Cents    int64
}

type Simplifier interface {
	Reduce(net map[Pair]int64) []Transfer
}

type LedgerStore interface {
	Append(ctx context.Context, e Expense) error
	Stream(ctx context.Context, g GroupID) (<-chan Expense, error)
}

type Group struct {
	ID      GroupID
	Members []UserID
	store   LedgerStore
	simp    Simplifier
	net     map[Pair]int64
	writeCh chan writeOp
}

type writeOp struct {
	exp  Expense
	done chan error
}

func (g *Group) AddExpense(ctx context.Context, e Expense) error {
	done := make(chan error, 1)
	g.writeCh <- writeOp{exp: e, done: done}
	return <-done
}

func (g *Group) Simplify() []Transfer { return g.simp.Reduce(g.net) }
