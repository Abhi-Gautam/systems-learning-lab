package vendingmachine

type Cash map[int]int

func (c Cash) Total() int {
	sum := 0
	for denom, count := range c {
		sum += denom * count
	}
	return sum
}

func makeChange(available Cash, amount int) (Cash, bool) {
	if amount == 0 {
		return Cash{}, true
	}
	remaining := amount
	out := Cash{}
	for i := len(DefaultAcceptedDenominations) - 1; i >= 0; i-- {
		d := DefaultAcceptedDenominations[i]
		need := remaining / d
		if have := available[d]; need > have {
			need = have
		}
		if need > 0 {
			out[d] = need
			remaining -= need * d
		}
	}
	if remaining != 0 {
		return nil, false
	}
	return out, true
}
