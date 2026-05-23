package internal

const alphabet = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"

// Base62 encodes n into the URL-safe 62-symbol alphabet.
// Empty/zero is encoded as "0".
func Base62(n uint64) string {
	if n == 0 {
		return "0"
	}
	buf := make([]byte, 0, 11) // max 11 chars for uint64
	for n > 0 {
		buf = append(buf, alphabet[n%62])
		n /= 62
	}
	// reverse
	for i, j := 0, len(buf)-1; i < j; i, j = i+1, j-1 {
		buf[i], buf[j] = buf[j], buf[i]
	}
	return string(buf)
}
