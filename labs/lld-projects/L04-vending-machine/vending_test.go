package vendingmachine

import (
	"errors"
	"maps"
	"testing"
)

// --- helpers ---

func testMachine(t *testing.T) *Machine {
	t.Helper()
	m := NewMachine(
		[]Product{
			{Code: "A1", Price: 65, Stock: 2},
			{Code: "B2", Price: 100, Stock: 1},
			{Code: "C3", Price: 30, Stock: 1},
		},
		Cash{1: 10, 5: 10, 10: 10, 20: 5, 50: 2, 100: 1},
	)
	if m == nil {
		t.Fatal("NewMachine returned nil")
	}
	return m
}

func insertAll(t *testing.T, m *Machine, denoms ...int) {
	t.Helper()
	for _, d := range denoms {
		if err := m.Insert(d); err != nil {
			t.Fatalf("Insert(%d): %v", d, err)
		}
	}
}

func sameCash(a, b Cash) bool {
	ac, bc := Cash{}, Cash{}
	if a != nil {
		ac = maps.Clone(a)
	}
	if b != nil {
		bc = maps.Clone(b)
	}
	for k, v := range ac {
		if v == 0 {
			delete(ac, k)
		}
	}
	for k, v := range bc {
		if v == 0 {
			delete(bc, k)
		}
	}
	return maps.Equal(ac, bc)
}

func assertCredit(t *testing.T, m *Machine, want int) {
	t.Helper()
	if got := m.Credit(); got != want {
		t.Fatalf("Credit()=%d, want %d", got, want)
	}
}

func assertStock(t *testing.T, m *Machine, code string, want int) {
	t.Helper()
	got, ok := m.Stock(code)
	if !ok {
		t.Fatalf("Stock(%q): unknown product", code)
	}
	if got != want {
		t.Fatalf("Stock(%q)=%d, want %d", code, got, want)
	}
}

// --- construction ---

func TestNewMachineSeedsCatalogAndDrawer(t *testing.T) {
	m := NewMachine(
		[]Product{{Code: "A1", Price: 65, Stock: 3}},
		Cash{10: 2, 5: 1},
	)
	assertStock(t, m, "A1", 3)
	if !sameCash(m.Drawer(), Cash{10: 2, 5: 1}) {
		t.Fatalf("Drawer()=%v, want {10:2, 5:1}", m.Drawer())
	}
	assertCredit(t, m, 0)
}

// --- insert ---

func TestInsertAcceptsDenomAndRaisesCredit(t *testing.T) {
	m := testMachine(t)
	insertAll(t, m, 20, 10)
	assertCredit(t, m, 30)
}

func TestInsertRejectsBadDenomWithoutChangingCredit(t *testing.T) {
	m := testMachine(t)
	insertAll(t, m, 10)
	err := m.Insert(3)
	if !errors.Is(err, ErrBadDenom) {
		t.Fatalf("Insert(3) err=%v, want ErrBadDenom", err)
	}
	assertCredit(t, m, 10)
}

func TestInsertAfterSuccessfulVendStartsNewSession(t *testing.T) {
	m := testMachine(t)
	insertAll(t, m, 50, 20) // 70 >= 65
	if err := m.Select("A1"); err != nil {
		t.Fatal(err)
	}
	if _, err := m.Dispense(); err != nil {
		t.Fatal(err)
	}
	assertCredit(t, m, 0)

	if err := m.Insert(100); err != nil {
		t.Fatalf("Insert after vend: %v", err)
	}
	assertCredit(t, m, 100)
}

// --- select ---

func TestSelectValidProduct(t *testing.T) {
	m := testMachine(t)
	if err := m.Select("A1"); err != nil {
		t.Fatal(err)
	}
}

func TestSelectUnknownProduct(t *testing.T) {
	m := testMachine(t)
	err := m.Select("Z9")
	if !errors.Is(err, ErrUnknownProduct) {
		t.Fatalf("err=%v, want ErrUnknownProduct", err)
	}
}

func TestSelectOutOfStock(t *testing.T) {
	m := NewMachine(
		[]Product{{Code: "A1", Price: 65, Stock: 0}},
		Cash{1: 50},
	)
	insertAll(t, m, 100)
	err := m.Select("A1")
	if !errors.Is(err, ErrOutOfStock) {
		t.Fatalf("err=%v, want ErrOutOfStock", err)
	}
	assertCredit(t, m, 100)
	assertStock(t, m, "A1", 0)
}

func TestSelectReplacesPreviousSelection(t *testing.T) {
	m := testMachine(t)
	insertAll(t, m, 100)
	if err := m.Select("A1"); err != nil {
		t.Fatal(err)
	}
	if err := m.Select("B2"); err != nil {
		t.Fatal(err)
	}
	res, err := m.Dispense()
	if err != nil {
		t.Fatal(err)
	}
	if res.Code != "B2" {
		t.Fatalf("dispensed %q, want B2", res.Code)
	}
	assertStock(t, m, "A1", 2)
	assertStock(t, m, "B2", 0)
}

// --- dispense happy paths ---

func TestDispenseExactPayment(t *testing.T) {
	// A1=65: 50+10+5
	m := testMachine(t)
	insertAll(t, m, 50, 10, 5)
	if err := m.Select("A1"); err != nil {
		t.Fatal(err)
	}
	res, err := m.Dispense()
	if err != nil {
		t.Fatal(err)
	}
	if res.Code != "A1" {
		t.Fatalf("code=%q", res.Code)
	}
	if res.Change.Total() != 0 {
		t.Fatalf("change total=%d, want 0 (%v)", res.Change.Total(), res.Change)
	}
	assertStock(t, m, "A1", 1)
	assertCredit(t, m, 0)
}

func TestDispenseWithChange(t *testing.T) {
	// B2=100; insert 125 → change 25
	m := testMachine(t)
	insertAll(t, m, 100, 20, 5)
	if err := m.Select("B2"); err != nil {
		t.Fatal(err)
	}
	res, err := m.Dispense()
	if err != nil {
		t.Fatal(err)
	}
	if res.Code != "B2" {
		t.Fatalf("code=%q", res.Code)
	}
	if res.Change.Total() != 25 {
		t.Fatalf("change total=%d, want 25 (%v)", res.Change.Total(), res.Change)
	}
	assertStock(t, m, "B2", 0)
	assertCredit(t, m, 0)
}

func TestDispenseSequentialSessions(t *testing.T) {
	m := testMachine(t)

	insertAll(t, m, 50, 10, 5)
	if err := m.Select("A1"); err != nil {
		t.Fatal(err)
	}
	if _, err := m.Dispense(); err != nil {
		t.Fatal(err)
	}
	assertStock(t, m, "A1", 1)

	insertAll(t, m, 50, 10, 5)
	if err := m.Select("A1"); err != nil {
		t.Fatal(err)
	}
	if _, err := m.Dispense(); err != nil {
		t.Fatal(err)
	}
	assertStock(t, m, "A1", 0)
}

// --- dispense failures (no mutation) ---

func TestDispenseInsufficientFundsLeavesStockAndCredit(t *testing.T) {
	m := testMachine(t)
	insertAll(t, m, 10)
	if err := m.Select("A1"); err != nil {
		t.Fatal(err)
	}
	_, err := m.Dispense()
	if !errors.Is(err, ErrInsufficientFunds) {
		t.Fatalf("err=%v, want ErrInsufficientFunds", err)
	}
	assertStock(t, m, "A1", 2)
	assertCredit(t, m, 10)
}

func TestDispenseWithoutSelection(t *testing.T) {
	m := testMachine(t)
	insertAll(t, m, 100)
	_, err := m.Dispense()
	if !errors.Is(err, ErrNoSelection) {
		t.Fatalf("err=%v, want ErrNoSelection", err)
	}
	assertStock(t, m, "A1", 2)
	assertCredit(t, m, 100)
}

func TestDispenseTwiceWithoutNewSession(t *testing.T) {
	m := testMachine(t)
	insertAll(t, m, 50, 10, 5)
	if err := m.Select("A1"); err != nil {
		t.Fatal(err)
	}
	if _, err := m.Dispense(); err != nil {
		t.Fatal(err)
	}
	_, err := m.Dispense()
	if err == nil {
		t.Fatal("second Dispense should fail")
	}
	assertStock(t, m, "A1", 1)
}

func TestDispenseCannotMakeChangeRefusesVend(t *testing.T) {
	// C3=30; pay 100; drawer only one 50 → cannot make 70 change
	m := NewMachine(
		[]Product{{Code: "C3", Price: 30, Stock: 1}},
		Cash{50: 1},
	)
	insertAll(t, m, 100)
	if err := m.Select("C3"); err != nil {
		t.Fatal(err)
	}
	drawerBefore := maps.Clone(m.Drawer())
	_, err := m.Dispense()
	if !errors.Is(err, ErrCannotMakeChange) {
		t.Fatalf("err=%v, want ErrCannotMakeChange", err)
	}
	assertStock(t, m, "C3", 1)
	assertCredit(t, m, 100)
	if !sameCash(m.Drawer(), drawerBefore) {
		t.Fatalf("drawer mutated on failed change: got %v want %v", m.Drawer(), drawerBefore)
	}
}

// --- cancel ---

func TestCancelReturnsInsertedCash(t *testing.T) {
	m := testMachine(t)
	insertAll(t, m, 20, 20)
	returned, err := m.Cancel()
	if err != nil {
		t.Fatal(err)
	}
	if returned.Total() != 40 {
		t.Fatalf("returned total=%d, want 40 (%v)", returned.Total(), returned)
	}
	assertCredit(t, m, 0)
	assertStock(t, m, "A1", 2)
}

func TestCancelAfterSelectDoesNotVend(t *testing.T) {
	m := testMachine(t)
	insertAll(t, m, 100)
	if err := m.Select("B2"); err != nil {
		t.Fatal(err)
	}
	returned, err := m.Cancel()
	if err != nil {
		t.Fatal(err)
	}
	if returned.Total() != 100 {
		t.Fatalf("returned=%d, want 100", returned.Total())
	}
	assertStock(t, m, "B2", 1)
	assertCredit(t, m, 0)
}

func TestCancelWithNoSession(t *testing.T) {
	m := testMachine(t)
	returned, err := m.Cancel()
	// no-op (nil err, empty) or ErrNoSession — both OK if credit stays 0
	if err != nil && !errors.Is(err, ErrNoSession) {
		t.Fatalf("unexpected err: %v", err)
	}
	if returned.Total() != 0 {
		t.Fatalf("returned=%v, want empty", returned)
	}
	assertCredit(t, m, 0)
}

func TestOutOfStockThenCancelRecoversMoney(t *testing.T) {
	m := NewMachine(
		[]Product{{Code: "A1", Price: 65, Stock: 0}},
		Cash{1: 20},
	)
	insertAll(t, m, 100)
	if err := m.Select("A1"); !errors.Is(err, ErrOutOfStock) {
		t.Fatalf("Select err=%v, want ErrOutOfStock", err)
	}
	returned, err := m.Cancel()
	if err != nil {
		t.Fatal(err)
	}
	if returned.Total() != 100 {
		t.Fatalf("returned=%d, want 100", returned.Total())
	}
}

// --- conservation ---

func TestSuccessfulVendAbsorbsPriceNotFullCredit(t *testing.T) {
	m := NewMachine(
		[]Product{{Code: "C3", Price: 30, Stock: 1}},
		Cash{10: 5, 5: 2, 1: 10},
	)
	before := m.Drawer().Total()
	insertAll(t, m, 50) // change 20
	if err := m.Select("C3"); err != nil {
		t.Fatal(err)
	}
	res, err := m.Dispense()
	if err != nil {
		t.Fatal(err)
	}
	if res.Change.Total() != 20 {
		t.Fatalf("change=%d, want 20", res.Change.Total())
	}
	// +50 from purse, −20 change → net +30 (price)
	after := m.Drawer().Total()
	if after-before != 30 {
		t.Fatalf("drawer delta=%d, want +30; before=%d after=%d change=%v",
			after-before, before, after, res.Change)
	}
}
