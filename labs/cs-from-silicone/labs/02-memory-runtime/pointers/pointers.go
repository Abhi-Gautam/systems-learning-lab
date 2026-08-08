// ============================================================================
// pointers.go  —  Go pointers, from silicon up.
// ============================================================================
//
// HOW TO READ THIS FILE
// ---------------------
// This is a LESSON, not a library. The comments do the teaching. Read it
// top-to-bottom while running it, and you will understand what a pointer
// physically IS and exactly how Go's pointers differ from C/C++/Java.
//
// THE CONCEPT LADDER (we climb it every time, in this order):
//
//   1. PHYSICAL  : RAM is a giant array of byte cells. Each cell has a number.
//                  That number is an "address". Nothing mystical — it's an
//                  index into a billion-slot array of bytes.
//   2. CPU / ISA : To the CPU an address is just a 64-bit integer. The only
//                  things it can do with it are LOAD (read the bytes at that
//                  number) and STORE (write bytes at that number).
//   3. OS/RUNTIME: The OS + Go runtime carve those bytes into regions — each
//                  goroutine gets a STACK; long-lived objects live on the
//                  HEAP; a garbage collector (GC) reclaims dead heap bytes.
//   4. LANGUAGE  : ONLY NOW do we name the Go feature: *T, &x, *p, new(), etc.
//                  A Go pointer is a typed, safe wrapper around step 1's number.
//
// HOW TO RUN
// ----------
//   cd .../labs/cs-from-silicone/labs/02-memory-runtime/pointers
//   go run pointers.go
//
// SEE THE COMPILER'S STACK/HEAP DECISIONS (escape analysis):
//   go build -gcflags=-m pointers.go      # prints "escapes to heap" etc.
//   rm -f pointers                        # delete the binary it leaves behind
//
// WHAT YOU'LL BE ABLE TO ARGUE AFTER THIS
// ---------------------------------------
// You'll be able to defend, to anyone: that a pointer is literally a byte
// address (a number), that Go is pass-by-value ALWAYS (a *T parameter is
// itself copied — it just copies the address), that Go forbids pointer
// arithmetic on purpose for memory safety (unlike C, where p+1 walks raw
// memory and is the classic foot-gun), that returning &local is a dangling
// bug in C but perfectly safe in Go because escape analysis promotes it to
// the GC-managed heap, and — the big one vs Java — that Go lets you point at
// the *inside* of a struct/array and decide value-vs-pointer yourself, while
// Java hides every object behind an implicit reference and never lets you
// take &field. You'll also be able to explain why a contiguous slice runs
// circles around a linked list purely because of CPU cache lines, even when
// both do the identical number of additions.
//
// ============================================================================

package main

import (
	"fmt"
	"math/rand"
	"time"
	"unsafe" // used ONLY for unsafe.Sizeof (compile-time, totally safe to read)
)

// ---- a tiny helper so every section prints a clean, labeled header ----------
func header(s string) {
	fmt.Printf("\n==== %s ====\n", s)
}

// ============================================================================
// MAIN — runs each lesson section in order. Scroll down to read each demo.
// ============================================================================
func main() {
	section01AddressIsANumber()
	section02AddrOfAndDeref()
	section03PointerIsAVariableToo()
	section04PassByValueOnly()
	section05NoPointerArithmetic()
	section06PointerToPointer()
	section07NewVsCompositeLiteral()
	section08ReferenceLikeTypes()
	section09ValueVsPointerReceivers()
	section10StackHeapEscape()
	section11GarbageCollection()
	section12CacheLessonBenchmark()
	finalSummary()
}

// ============================================================================
// 01. WHAT AN ADDRESS *IS*  — a number naming a byte cell.
// ============================================================================
//
// PHYSICAL: Picture RAM as one enormous array of 1-byte cells:
//
//	byte[0] byte[1] byte[2] ... byte[17179869183] ...   (16 GB = ~1.7e10 cells)
//
// Each cell holds 8 bits. The "address" of a cell is simply its index in that
// array. So address 0x16b3a2f4c8 just means "cell number 97,823,...,xyz".
//
// CPU: When you ask for the address of a variable, the CPU hands you that
// integer index. Go stores that index in a POINTER. On a 64-bit machine the
// index is 64 bits = 8 bytes wide (enough to name 2^64 cells).
//
// LANGUAGE: Go prints addresses in HEX (base-16, the 0x... form) because hex
// maps cleanly onto bytes (2 hex digits = 1 byte). The numbers below are real
// addresses from THIS run; they'll differ run-to-run (the OS randomizes where
// your stack lands — a security feature called ASLR).
func section01AddressIsANumber() {
	header("01. WHAT AN ADDRESS IS (a number naming a byte cell)")

	a := 10
	b := 20
	c := 30

	// &a means "give me the byte-cell index where a lives". %p prints it in hex.
	fmt.Printf("a = %d   lives at address &a = %p\n", a, &a)
	fmt.Printf("b = %d   lives at address &b = %p\n", b, &b)
	fmt.Printf("c = %d   lives at address &c = %p\n", c, &c)

	// Notice a, b, c sit a fixed number of bytes apart: they are neighbors on
	// the stack. An int is 8 bytes here, so consecutive locals are ~8 apart.
	fmt.Printf("an int is %d bytes wide (unsafe.Sizeof(a))\n", unsafe.Sizeof(a))

	// A POINTER itself is just that 64-bit number, so every pointer is 8 bytes,
	// no matter what it points AT (a *int and a *[1000]byte are both 8 bytes).
	p := &a
	var big *[1000]byte
	fmt.Printf("a pointer is %d bytes wide regardless of type: *int=%d  *[1000]byte=%d\n",
		unsafe.Sizeof(p), unsafe.Sizeof(p), unsafe.Sizeof(big))
}

// ============================================================================
// 02. & (ADDRESS-OF) AND * (DEREFERENCE)
// ============================================================================
//
// Two operators, easy to mix up. Pin them down:
//
//	&x   ADDRESS-OF : "what byte-cell number does x live at?"  -> a pointer.
//	*p   DEREFERENCE: "go to the cell p names and read/write the value there."
//
// And one piece of TYPE syntax that reuses the * symbol:
//
//	var p *int      <- here *int is a TYPE meaning "pointer to an int".
//	*p = 5          <- here *p is an OPERATION meaning "store 5 at p's cell".
//
// Same symbol, two jobs. In a type position (after var, in a func signature,
// in a struct field) the * builds a pointer type. In an expression the * does
// the LOAD/STORE at that address.
func section02AddrOfAndDeref() {
	header("02. ADDRESS-OF (&) AND DEREFERENCE (*)")

	x := 42
	var p *int = &x // p HOLDS the address of x. (*int is the TYPE here.)

	fmt.Printf("x = %d, &x = %p\n", x, &x)
	fmt.Printf("p (a copy of x's address) = %p, *p (value at that cell) = %d\n", p, *p)

	// Write THROUGH the pointer: store 5 into the cell p names. That cell IS x.
	*p = 5 // here *p is the DEREF/STORE operation, not a type.
	fmt.Printf("after *p = 5, x = %d (we mutated x without naming x)\n", x)

	// THE ZERO VALUE OF A POINTER IS nil: it names "cell number 0", which is a
	// forbidden address. Dereferencing nil asks the CPU to load from address 0;
	// the OS traps that and the Go runtime turns it into a panic. We demonstrate
	// it SAFELY: recover() inside a deferred func catches the panic so the
	// program keeps running instead of crashing.
	demonstrateNilDeref()
}

func demonstrateNilDeref() {
	// defer runs this function as we leave demonstrateNilDeref, even via panic.
	// recover() grabs the panic value so it doesn't propagate and kill main.
	defer func() {
		if r := recover(); r != nil {
			fmt.Printf("nil deref panicked (caught safely): %v\n", r)
		}
	}()

	var np *int // zero value of any pointer is nil (names address 0 = invalid)
	fmt.Printf("np == nil? %v\n", np == nil)
	_ = *np // <-- LOAD from address 0: panics with "invalid memory address".
	fmt.Println("this line never runs")
}

// ============================================================================
// 03. A POINTER IS ITSELF A VARIABLE (it has its own address)
// ============================================================================
//
// A pointer is not magic — it's a normal 8-byte variable that happens to hold
// an address. So it lives in its own byte cell, and that cell has an address
// too. Three distinct things, don't conflate them:
//
//	&p  : where the pointer variable itself lives
//	 p  : the value INSIDE the pointer (= the address of x)
//	*p  : the value at the cell p points to (= x's value)
func section03PointerIsAVariableToo() {
	header("03. A POINTER IS A VARIABLE AT ITS OWN ADDRESS")

	x := 7
	p := &x

	fmt.Printf("x  (the value)                 = %d\n", x)
	fmt.Printf("&x (where x lives)             = %p\n", &x)
	fmt.Printf("p  (the address stored in p)   = %p   <- equals &x\n", p)
	fmt.Printf("&p (where p itself lives)      = %p   <- a DIFFERENT cell\n", &p)
	fmt.Printf("*p (value at the cell p names) = %d   <- equals x\n", *p)
}

// ============================================================================
// 04. PASS-BY-VALUE IS GO'S *ONLY* MODEL (but the value can be a pointer)
// ============================================================================
//
// THE RULE: every Go function argument is COPIED into the parameter. Always.
// Go has NO pass-by-reference (unlike C++ `int&` or C#'s `ref`).
//
// So how do you let a function mutate the caller's variable? You copy a
// POINTER. The pointer (the address number) is copied, but the copy still
// names the same cell — so writing through it hits the caller's storage.
//
//	mutCopy(n int)   : copies the int. Writing n changes only the copy.
//	mutPtr(n *int)   : copies the address. *n writes to the caller's cell.
//
// We print addresses to PROVE the parameter is a separate variable (a copy).
func mutCopy(n int) {
	fmt.Printf("   inside mutCopy:  param n lives at %p (a COPY)\n", &n)
	n = 999 // mutates only this local copy
}

func mutPtr(n *int) {
	fmt.Printf("   inside mutPtr:   param n lives at %p (a COPY of the pointer)\n", &n)
	fmt.Printf("   inside mutPtr:   but n holds %p, which is the caller's cell\n", n)
	*n = 999 // writes THROUGH the address into the caller's variable
}

func section04PassByValueOnly() {
	header("04. PASS-BY-VALUE ONLY (opt into mutation with *T)")

	x := 1
	fmt.Printf("before: x = %d, &x = %p\n", x, &x)

	mutCopy(x)
	fmt.Printf("after mutCopy(x):  x = %d  <- UNCHANGED (we copied the value)\n", x)

	mutPtr(&x)
	fmt.Printf("after mutPtr(&x):  x = %d  <- CHANGED (we copied the address)\n", x)

	// Take-away: `*int` parameters are not "pass by reference". They're a
	// pass-by-VALUE copy of an address. The reference-like behavior is a
	// consequence of two copies pointing at one cell, nothing more.
}

// ============================================================================
// 05. NO POINTER ARITHMETIC — by design, for memory safety
// ============================================================================
//
// In C, a pointer is a raw integer you may do MATH on:
//
//	int a[3] = {10,20,30};
//	int *p = a;
//	int v = *(p + 2);   // C: legal — read a[2] by adding 2*sizeof(int) bytes
//	p += 99;            // C: legal — now p points into who-knows-what
//	*p = 0;             // C: undefined behavior, silent memory corruption
//
// That p+1 stepping is the #1 source of buffer overflows and security CVEs.
//
// Go FORBIDS arithmetic on pointers entirely. You cannot write p+1, p++, or
// p[2] on a *int. The compiler rejects it. The only legal pointer operations
// in safe Go are: take an address (&), dereference (*), compare (== / !=), and
// assign. This is a deliberate trade: you give up raw memory walking, you get
// a language where a valid pointer is ALWAYS valid (or nil) — never dangling,
// never pointing one-past-the-end into garbage.
//
// (Go does ship an escape hatch — the unsafe package's unsafe.Pointer +
// unsafe.Add — for runtime/FFI authors. It is OFF-LIMITS in normal Go and
// turns off the safety net. We don't use it; we only mention it exists.)
func section05NoPointerArithmetic() {
	header("05. NO POINTER ARITHMETIC (memory safety by design)")

	arr := [3]int{10, 20, 30}
	p := &arr[0]
	fmt.Printf("p = &arr[0] = %p, *p = %d\n", p, *p)

	// The following line, if uncommented, would NOT COMPILE:
	//     q := p + 1            // C does this; Go: "invalid operation: p + 1"
	// To reach arr[1] in Go you index the array/slice (which is bounds-checked
	// at runtime), you never do address math:
	fmt.Printf("to reach the next element you INDEX (bounds-checked): arr[1] = %d\n", arr[1])

	fmt.Println("Go rejects p+1 at compile time -> no buffer overflows from pointer walks.")
}

// ============================================================================
// 06. POINTER-TO-POINTER (**int) — a chain of address-of
// ============================================================================
//
// Nothing new, just one more level. If p holds the address of x, then pp can
// hold the address of p. Follow the chain:
//
//	x          a value
//	p  = &x    a cell holding x's address
//	pp = &p    a cell holding p's address
//
//	*pp        -> the value in p          (= x's address)
//	**pp       -> the value at that addr  (= x's value)
func section06PointerToPointer() {
	header("06. POINTER-TO-POINTER (**int)")

	x := 100
	p := &x
	pp := &p // *int address-of -> **int

	fmt.Printf("x   = %d\n", x)
	fmt.Printf("p   = %p   (&x)\n", p)
	fmt.Printf("*p  = %d   (value at p)\n", *p)
	fmt.Printf("&p  = %p   (where p lives)\n", &p)
	fmt.Printf("pp  = %p   (&p)\n", pp)
	fmt.Printf("*pp = %p   (value in pp = p = &x)\n", *pp)
	fmt.Printf("**pp= %d   (follow twice = x)\n", **pp)

	**pp = 200 // walk x <- p <- pp and store; mutates x two hops away
	fmt.Printf("after **pp = 200, x = %d\n", x)
}

// ============================================================================
// 07. new() vs &T{} vs COMPOSITE LITERALS
// ============================================================================
//
// Three ways to obtain a pointer to storage. They differ in ergonomics, not in
// kind — each gives you a *T to a freshly-allocated, properly-typed cell.
//
//	new(T)   : allocate one zeroed T, return *T. Good when you just want a
//	           pointer to a zero value (e.g. new(int) -> *int pointing at 0).
//	&T{...}  : allocate a T, fill fields via a composite literal, return *T.
//	           The idiomatic way to make a pointer to a populated struct.
//	&v       : take the address of an existing variable v.
//
// WHERE does the storage live — stack or heap? You don't say, and you can't
// tell from the syntax. The COMPILER decides via escape analysis (section 10).
type point struct {
	x, y int
}

func section07NewVsCompositeLiteral() {
	header("07. new() vs &T{} vs COMPOSITE LITERALS")

	pi := new(int) // *int to a zeroed int
	fmt.Printf("new(int): pi=%p, *pi=%d (zeroed)\n", pi, *pi)
	*pi = 5
	fmt.Printf("after *pi=5: *pi=%d\n", *pi)

	p1 := &point{x: 1, y: 2} // idiomatic: pointer to a populated struct
	fmt.Printf("&point{1,2}: p1=%p, *p1=%v\n", p1, *p1)

	p2 := new(point) // equivalent storage, but zeroed: &point{0,0}
	fmt.Printf("new(point):  p2=%p, *p2=%v (zeroed)\n", p2, *p2)

	// new(point) and &point{} produce the same thing: a *point to zeroed memory.
	fmt.Println("new(T) and &T{} are equivalent ways to get a *T to fresh storage.")
}

// ============================================================================
// 08. REFERENCE-LIKE TYPES: slices & maps  — THE #1 GO POINTER GOTCHA
// ============================================================================
//
// Slices and maps FEEL like references, but the rule "everything is copied"
// still holds. The subtlety is WHAT gets copied.
//
// A SLICE is a 3-word header: {ptr, len, cap}
//
//	ptr -> the first element of a backing ARRAY (somewhere in memory)
//	len  = how many elements are in view
//	cap  = how many fit before the backing array runs out
//
// When you pass a slice, the HEADER (those 3 words) is copied by value. But the
// COPY's ptr still points at the SAME backing array. Consequences:
//
//   - Writing s[0] = ... goes through ptr -> hits the shared array ->
//     VISIBLE to the caller.
//   - append(s, ...) when len < cap also writes into the shared array ->
//     may be visible. But append when len == cap must ALLOCATE A NEW, bigger
//     array and repoint the LOCAL copy's ptr at it. The caller's header still
//     points at the OLD array -> the appended data is NOT visible.
//
// This is where people get burned. We demonstrate both halves below.
func writeAndAppend(s []int) {
	fmt.Printf("   inside: header copied; ptr=%p len=%d cap=%d\n",
		&s[0], len(s), cap(s))
	s[0] = 999 // writes through shared backing array -> caller sees it

	// We deliberately built the caller's slice with len==cap, so this append
	// must allocate a NEW array and repoint OUR LOCAL copy only.
	s = append(s, 7)
	fmt.Printf("   inside: after append past cap, ptr=%p (NEW array), last=%d\n",
		&s[0], s[len(s)-1])
}

// A MAP value is essentially a single pointer to a runtime hashmap (hmap)
// struct. Copying a map value copies that pointer, so both copies see the same
// underlying table -> inserts/updates inside a function ARE visible to caller.
func insertIntoMap(m map[string]int) {
	m["added_inside"] = 1 // mutates the shared hmap -> caller sees it
}

func section08ReferenceLikeTypes() {
	header("08. REFERENCE-LIKE TYPES (slice header & map — the #1 gotcha)")

	// Build a slice with len == cap so the append below is forced to reallocate.
	s := make([]int, 3, 3)
	s[0], s[1], s[2] = 1, 2, 3
	fmt.Printf("caller before: %v  ptr=%p len=%d cap=%d\n", s, &s[0], len(s), cap(s))

	writeAndAppend(s)

	fmt.Printf("caller after:  %v  len=%d\n", s, len(s))
	fmt.Println("-> element s[0] CHANGED to 999 (shared backing array)")
	fmt.Println("-> the appended 7 is INVISIBLE (append past cap made a new array)")

	// Map demo.
	m := map[string]int{"start": 0}
	fmt.Printf("\ncaller map before: %v\n", m)
	insertIntoMap(m)
	fmt.Printf("caller map after:  %v  -> insert IS visible (map = ptr to hmap)\n", m)
}

// ============================================================================
// 09. VALUE vs POINTER RECEIVERS on methods (brief — deeper in the OOP file)
// ============================================================================
//
// A method receiver is just the first argument in disguise, so section 04's
// rule applies: a VALUE receiver gets a COPY (can't mutate the original); a
// POINTER receiver gets the address (can mutate the original).
type counter struct{ n int }

func (c counter) incByValue() { c.n++ } // mutates a copy -> caller unaffected
func (c *counter) incByPtr()  { c.n++ } // mutates through address -> sticks
func (c counter) get() int    { return c.n }

func section09ValueVsPointerReceivers() {
	header("09. VALUE vs POINTER RECEIVERS")

	c := counter{n: 0}
	c.incByValue()
	fmt.Printf("after incByValue(): n = %d  <- unchanged (receiver was a copy)\n", c.get())

	c.incByPtr() // Go auto-takes &c here since c is addressable
	fmt.Printf("after incByPtr():   n = %d  <- changed (receiver was *counter)\n", c.get())
}

// ============================================================================
// 10. STACK vs HEAP + ESCAPE ANALYSIS  (the Go-specific climax)
// ============================================================================
//
// PHYSICAL/RUNTIME refresher:
//   STACK : each goroutine has a contiguous stack. Locals live here; freeing is
//           free — the stack pointer just moves back when the function returns.
//   HEAP  : a shared region for objects that must outlive their function. The
//           GC tracks and reclaims heap objects.
//
// WHO DECIDES which one?
//   In C/C++ YOU decide: a local `int x;` is stack; `malloc`/`new` is heap.
//   And in C, returning &x (the address of a stack local) is a DANGLING-POINTER
//   bug: the stack frame is gone the instant the function returns, so the
//   address now names reused garbage. Classic crash / security hole.
//
//   In Go the COMPILER decides, using ESCAPE ANALYSIS. It asks: "does any
//   reference to this value survive past the function?" If NO, the value stays
//   on the stack even though you took its address. If YES — e.g. you return
//   &local, or store its address in a global/heap object — the value "escapes"
//   and the compiler quietly allocates it on the HEAP instead. So returning
//   &local in Go is SAFE: it was never on a dying stack frame; it lives on the
//   GC-managed heap and stays alive as long as something references it.
//
// Two functions below: one whose local does NOT escape, one that returns
// &local and therefore DOES. Run `go build -gcflags=-m pointers.go` to watch
// the compiler announce each decision.
//
// ----------------------------------------------------------------------------
// REAL COMPILER OUTPUT captured from this exact file via:
//     go build -gcflags=-m pointers.go 2>&1 | grep -E 'escap|moved to heap'
//
//   ./pointers.go:497:2:  moved to heap: local    <- escapes()'s local promoted
//   ./pointers.go:506:15: moved to heap: local    <- same, reported at the
//                                                      inlined call site too
//   ./pointers.go:536:17: make([]byte, 1048576) escapes to heap  <- the 1 MB in
//                                                      section 11 also escapes
//
// (Your exact line numbers will differ as you edit the file; the key signals
//  are the phrases "moved to heap" and "escapes to heap". The compiler also
//  inlined escapes() — that's why "moved to heap: local" shows up twice (once
//  for the body, once for the inlined copy). Crucially, noEscape's `local`
//  produces NO such line — proof it stayed on the stack even though we took
//  its address. That is escape analysis doing its job.)
// ----------------------------------------------------------------------------

// noEscape: we take &local and use it, but nothing leaves the function, so the
// compiler keeps `local` on the stack. (No "escapes" line for it under -m.)
func noEscape() int {
	local := 11
	p := &local // address taken...
	*p = 22     // ...but used only here; never returned or stored elsewhere
	return *p   // we return the VALUE, not the address -> no escape
}

// escapes: we return the ADDRESS of a local. Its lifetime must outlive this
// frame, so the compiler moves `local` to the heap. Safe in Go (GC keeps it);
// the identical code in C would be a dangling pointer.
func escapes() *int {
	local := 33
	return &local // &local escapes to heap
}

func section10StackHeapEscape() {
	header("10. STACK vs HEAP + ESCAPE ANALYSIS")

	fmt.Printf("noEscape() returned %d (local stayed on the stack)\n", noEscape())

	pe := escapes()
	fmt.Printf("escapes() returned a *int = %p, *pe = %d\n", pe, *pe)
	fmt.Println("Returning &local is a DANGLING-POINTER BUG in C, but SAFE in Go:")
	fmt.Println("  escape analysis promoted `local` to the GC-managed heap.")
	fmt.Println("Run `go build -gcflags=-m pointers.go` to see 'escapes to heap'.")
}

// ============================================================================
// 11. GARBAGE COLLECTION — no free()/delete in Go
// ============================================================================
//
// In C you must free() every malloc; in C++ you delete or rely on RAII
// destructors to release memory deterministically. Forget, and you leak;
// free twice or use-after-free, and you corrupt memory. In Go there is NO
// free and NO delete-the-object: the runtime ships a concurrent, tracing
// GARBAGE COLLECTOR. Periodically it starts from the "roots" (globals,
// goroutine stacks, registers), follows every pointer to find all REACHABLE
// objects, and reclaims the byte ranges of everything unreachable. So an
// object lives exactly as long as some pointer can still reach it; when the
// last reference drops, it becomes eligible for collection — you never say
// when. The trade vs C++/RAII: you give up deterministic, programmer-timed
// release (and pay some CPU for the GC), and you gain freedom from leaks,
// double-frees, and dangling pointers. (This is also why fewer pointers =
// less work for the GC to scan — relevant to the next section.)
func section11GarbageCollection() {
	header("11. GARBAGE COLLECTION (no free()/delete)")

	makeGarbage := func() {
		// Allocate something on the heap and immediately drop the only reference
		// to it. In C this would leak; in Go the GC will reclaim it later.
		leaked := make([]byte, 1<<20) // 1 MB, escapes (returned-by-closure pattern)
		_ = leaked
	}
	makeGarbage()
	fmt.Println("Allocated 1 MB and dropped all references — no free() needed.")
	fmt.Println("The GC traces reachable pointers from the roots and reclaims the rest.")
	fmt.Println("Trade vs C++ RAII: no manual free (no leaks/double-free) but non-deterministic timing.")
}

// ============================================================================
// 12. WHY POINTER-CHASING IS SLOWER THAN ARRAYS — THE CACHE LESSON (climax)
// ============================================================================
//
// PHYSICAL: RAM is slow (~100ns) compared to the CPU (sub-ns per op). To hide
// that gap, the CPU has small fast caches. The unit of transfer is a CACHE
// LINE — 64 bytes on arm64. When you touch one byte, the CPU loads the whole
// 64-byte line around it. It ALSO runs a hardware PREFETCHER that, on seeing
// you march forward through memory, fetches the NEXT lines before you ask.
//
// CONSEQUENCE:
//   - A []int is one CONTIGUOUS block. Summing it walks straight through
//     memory: each 64-byte line brings 8 ints, the prefetcher stays one step
//     ahead, almost every access is a cache HIT. This STREAMS.
//   - A linked list (node{v; next *node}) scatters its nodes anywhere the
//     allocator put them. Each `node.next` jumps to an unpredictable address
//     the prefetcher can't guess -> a likely cache MISS -> a full ~100ns RAM
//     stall, every node. We even build the list in SHUFFLED allocation order
//     to guarantee the nodes are scattered and defeat any accidental locality.
//
// Both loops do the SAME number of additions. The only difference is memory
// LAYOUT. The timing gap below is pure cache-miss cost.
//
// (Deterministic: we seed our own rand source with 1 so every run is identical
//
//	and gofmt/vet stay happy — we use rand.New(rand.NewSource(1)).)
type node struct {
	v    int
	next *node
}

func section12CacheLessonBenchmark() {
	header("12. CACHE LESSON: contiguous slice vs linked list")

	const N = 5_000_000
	r := rand.New(rand.NewSource(1)) // fixed seed -> deterministic, reproducible

	// --- contiguous []int ---
	slice := make([]int, N)
	for i := range slice {
		slice[i] = i
	}

	// --- linked list with SHUFFLED allocation/link order (scatter the nodes) ---
	// Allocate all nodes, then link them in a random permutation so successive
	// `.next` hops land on far-apart heap addresses -> cache-hostile.
	nodes := make([]node, N)
	perm := r.Perm(N) // random permutation of 0..N-1
	for i := 0; i < N; i++ {
		nodes[perm[i]].v = i
		if i+1 < N {
			nodes[perm[i]].next = &nodes[perm[i+1]] // link in scattered order
		}
	}
	headNode := &nodes[perm[0]]

	// --- time the contiguous sum ---
	t0 := time.Now()
	var sumSlice int
	for i := 0; i < N; i++ {
		sumSlice += slice[i]
	}
	dSlice := time.Since(t0)

	// --- time the pointer-chasing sum ---
	t1 := time.Now()
	var sumList int
	for n := headNode; n != nil; n = n.next {
		sumList += n.v
	}
	dList := time.Since(t1)

	fmt.Printf("N = %d additions each (identical work)\n", N)
	fmt.Printf("contiguous []int sum : %v   (sum=%d)\n", dSlice, sumSlice)
	fmt.Printf("linked-list   sum    : %v   (sum=%d)\n", dList, sumList)
	if dSlice > 0 {
		fmt.Printf("linked list was ~%.1fx SLOWER — purely cache misses / lost prefetch.\n",
			float64(dList)/float64(dSlice))
	}
	fmt.Println("Same add-count, very different time: memory LAYOUT, not work, decides speed.")
	fmt.Println("(Bonus: fewer pointers also means less for Go's GC to scan.)")
}

// ============================================================================
// FINAL SUMMARY — the mental model + a C-vs-Go contrast table
// ============================================================================
func finalSummary() {
	header("SUMMARY: the mental model & C-vs-Go contrast")

	fmt.Println(`MENTAL MODEL (one sentence per rung of the ladder):
  1. An address is just a NUMBER: the index of a byte cell in RAM.
  2. The CPU only LOADs and STOREs at addresses; a pointer holds one (8 bytes).
  3. The runtime puts short-lived values on the goroutine STACK and escaping
     ones on the GC-managed HEAP; you never call free().
  4. A Go pointer (*T) is a typed, safe handle on that number: &, *, ==, assign.

KEY ARGUMENTS YOU CAN NOW MAKE:
  - Go is pass-by-VALUE always; a *T parameter is a COPIED address, not a ref.
  - Slices/maps look like references because the copied header/pointer still
    points at one shared backing store — but append past cap breaks the link.
  - Returning &local is a dangling bug in C, but escape analysis makes it SAFE
    in Go by promoting the value to the heap.
  - Arrays beat linked lists not from doing less work but from cache locality.`)

	fmt.Println(`
+-----------------------+----------------------------+----------------------------+
| Aspect                | C / C++                    | Go                         |
+-----------------------+----------------------------+----------------------------+
| Pointer arithmetic    | yes (p+1, p[i] math)       | NO (compile error)         |
| Stack vs heap chosen  | by you (local vs malloc)   | by compiler (escape anal.) |
| return &local         | dangling-pointer BUG       | safe (escapes to heap)     |
| Freeing memory        | manual free()/delete/RAII  | automatic GC, no free()    |
| Point inside a struct | yes (&s.field)             | yes (&s.field)             |
| Java equivalent       | n/a                        | Java hides ALL refs; no &  |
| Pass model            | value, &ref (C++), pointer | VALUE only (*T to mutate)  |
| Null deref            | UB / segfault              | panic (recoverable)        |
+-----------------------+----------------------------+----------------------------+`)

	fmt.Println("\nDone. Re-run with `go build -gcflags=-m pointers.go` to watch escape analysis.")
}
