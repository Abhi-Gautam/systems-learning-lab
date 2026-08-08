// =============================================================================
// oop.go — How Go does "Object-Oriented Programming" (from the silicon up)
// =============================================================================
//
// RUN ME:
//
//	cd /Volumes/mac-devlopment/reading-list/labs/cs-from-silicone/labs/02-memory-runtime/oop
//	go run oop.go
//
// (Also kept clean for:  gofmt -l oop.go   and   go vet oop.go )
//
// -----------------------------------------------------------------------------
// THE CONCEPT LADDER WE CLIMB IN THIS FILE (silicon-first, language-last)
// -----------------------------------------------------------------------------
//
//	1. BYTES        A struct is just a run of bytes in memory. Fields are laid
//	                out in declaration order, with alignment padding holes.
//	2. CPU / ISA    A "method" is NOT magic: it is a plain function whose first
//	                argument is the receiver (the hidden `this`). A method call
//	                is a normal CALL instruction. An interface value is a TWO-WORD
//	                pair (type-info pointer, data pointer); a dynamic method call
//	                loads a function pointer out of a table (the itab) and does an
//	                INDIRECT CALL — the same family as a C++ vtable jump.
//	3. RUNTIME      Pointers may force a heap allocation (escape analysis); GC
//	                owns the lifetime, so there are no destructors. defer/Close()
//	                handle non-memory cleanup.
//	4. LANGUAGE     Only NOW do we name the features: struct, method, interface,
//	                embedding, exported/unexported. The machine reality came first.
//
// -----------------------------------------------------------------------------
// WHAT YOU'LL BE ABLE TO ARGUE AFTER THIS (especially vs Java / C++)
// -----------------------------------------------------------------------------
// Go deliberately has NO classes and NO inheritance. You'll be able to explain,
// down to the machine level, that Go replaces inheritance with two separate
// tools: COMPOSITION via struct embedding (code reuse) and IMPLICIT INTERFACES
// (polymorphism / "duck typing"). You'll be able to show that a Go interface
// value is literally 16 bytes (two pointers) and that dynamic dispatch is one
// indirect jump through a per-(interface, concrete-type) function table — so it
// is just as "real" as a C++ vtable, but decoupled from any class hierarchy.
// And you'll have concrete ammunition for WHY Go ducked inheritance: the fragile
// base class problem, deep brittle hierarchies, and the diamond problem — none
// of which exist when types only promise behavior (interfaces) instead of
// inheriting implementation.
// =============================================================================

package main

import (
	"fmt"
	"strings"
	"unsafe"
)

// hr prints a labeled section header so the program reads as a lesson when run.
func hr(title string) {
	fmt.Printf("\n==== %s ====\n", title)
}

// =============================================================================
// 01. NO CLASSES — A STRUCT IS JUST BYTES
// =============================================================================
//
// In Java/C++ you reach for `class`. Go has no `class` keyword at all. The data
// container is a `struct`: a named, ordered bundle of fields. At the machine
// level a struct is nothing but a contiguous block of memory; each field sits at
// a fixed byte OFFSET from the start of that block.
//
// Two silicon facts that drive struct layout:
//   - ALIGNMENT: the CPU prefers (and on some ISAs requires) an N-byte value to
//     start at an address that is a multiple of N. An int64 wants an 8-aligned
//     address; a bool only needs 1.
//   - PADDING: to satisfy alignment, the compiler inserts unused "hole" bytes
//     between fields. So sizeof(struct) is usually >= sum of field sizes.
//
// We will literally print Sizeof and Offsetof to SEE the holes.

type Account struct {
	Active  bool    // 1 byte  ... then 7 padding bytes so Balance is 8-aligned
	Balance int64   // 8 bytes
	Rate    float32 // 4 bytes ... then 4 trailing padding bytes so the whole
	//             struct stays 8-aligned (in case it sits in an array)
}

// A second layout to make the point that ORDER changes size. Same three fields,
// reordered badly: bool, int64, float32 with the bool wedged between big fields
// creates more holes. (Here we show a deliberately wasteful ordering.)
type AccountPadded struct {
	A bool  // 1 byte + 7 pad
	B int64 // 8 bytes
	C bool  // 1 byte + 7 pad  (trailing, to keep 8-alignment)
}

func demoStructBytes() {
	hr("01. NO CLASSES — A STRUCT IS JUST BYTES")

	var acc Account
	fmt.Println("A struct is a contiguous block of bytes; fields live at fixed offsets.")
	fmt.Printf("unsafe.Sizeof(Account)      = %d bytes\n", unsafe.Sizeof(acc))
	fmt.Printf("  offset of Active  (bool)  = %d\n", unsafe.Offsetof(acc.Active))
	fmt.Printf("  offset of Balance (int64) = %d   <- jumped past 7 padding bytes\n", unsafe.Offsetof(acc.Balance))
	fmt.Printf("  offset of Rate    (f32)   = %d\n", unsafe.Offsetof(acc.Rate))
	fmt.Println("  sum of field sizes = 1+8+4 = 13, but Sizeof is bigger: that gap is PADDING.")

	var bad AccountPadded
	fmt.Printf("\nReorder the fields and the size changes (alignment holes):\n")
	fmt.Printf("unsafe.Sizeof(AccountPadded) = %d bytes (bool,int64,bool wastes more)\n", unsafe.Sizeof(bad))

	// Print the base address to drive home "it's just memory at an address".
	fmt.Printf("\n&acc = %p  (this is where the byte block begins)\n", &acc)
	fmt.Printf("&acc.Balance = %p (= base + offset %d)\n", &acc.Balance, unsafe.Offsetof(acc.Balance))
}

// =============================================================================
// 02. METHODS, AND VALUE vs POINTER RECEIVERS
// =============================================================================
//
// CPU/ISA VIEW FIRST. A "method" is sugar. When you write:
//
//	func (a Account) IsRich() bool { ... }
//
// the compiler emits an ordinary function roughly like:
//
//	func Account_IsRich(a Account) bool { ... }
//
// The receiver `a` is just the FIRST argument — the explicit version of C++'s
// hidden `this` pointer. Calling acc.IsRich() compiles to a normal CALL with acc
// pushed as arg 0. There is no per-object function pointer, no vtable: the
// target address is known at compile time (this is a STATIC, direct call).
//
// VALUE RECEIVER (func (a Account)):   the call COPIES the whole struct into the
//                                      argument slot. Mutations touch the copy
//                                      and vanish when the method returns.
// POINTER RECEIVER (func (a *Account)): the call passes the ADDRESS (8 bytes).
//                                      The method dereferences it, so writes hit
//                                      the original bytes.

// Value receiver: gets a COPY. Can read, cannot mutate the caller's value.
func (a Account) Describe() string {
	return fmt.Sprintf("Account{Active:%v Balance:%d Rate:%.2f}", a.Active, a.Balance, a.Rate)
}

// Value receiver that TRIES to mutate — and fails, because it edits the copy.
func (a Account) TryDepositByValue(amount int64) {
	a.Balance += amount // writes to the local copy `a`, not the caller's struct
}

// Pointer receiver: gets the ADDRESS, so it mutates the real struct.
func (a *Account) Deposit(amount int64) {
	a.Balance += amount // a.Balance is (*a).Balance — auto-dereferenced sugar
}

func demoReceivers() {
	hr("02. VALUE vs POINTER RECEIVERS (the copy is real)")

	acc := Account{Active: true, Balance: 100, Rate: 1.5}
	fmt.Printf("start:            %s\n", acc.Describe())
	fmt.Printf("address of caller's acc:        %p\n", &acc)

	// Value receiver mutation is silently lost.
	acc.TryDepositByValue(50)
	fmt.Printf("after TryDepositByValue(50):     %s  <- UNCHANGED (mutated a copy)\n", acc.Describe())

	// Pointer receiver mutation sticks.
	acc.Deposit(50)
	fmt.Printf("after Deposit(50) [ptr receiver]: %s  <- changed for real\n", acc.Describe())

	// PROOF of the copy: a value-receiver method sees a DIFFERENT address than
	// the caller's variable, because the receiver is a fresh copy on the stack.
	fmt.Printf("\nproof of copying — addresses of the receiver inside each method:\n")
	acc.showValueReceiverAddr()
	acc.showPointerReceiverAddr()

	// METHOD SET RULE (this is the bridge to interfaces in section 04):
	//   - Methods with a VALUE receiver belong to BOTH T and *T.
	//   - Methods with a POINTER receiver belong ONLY to *T.
	// Why: to call a pointer-receiver method, the runtime needs an ADDRESS to
	// take. A plain T value (e.g. one returned from a function, or a map element)
	// may not be addressable, so Go does not put pointer methods in T's set.
	//
	// PRACTICAL RULE: pick ONE receiver style per type and be consistent. Use
	// pointer receivers if ANY method mutates, or if the struct is large (to
	// avoid copying many bytes on every call).
	fmt.Println("\nMethod-set rule: pointer-receiver methods are in *T's set, NOT T's.")
	fmt.Println("=> a plain T value may fail to satisfy an interface (see section 04).")
}

func (a Account) showValueReceiverAddr() {
	fmt.Printf("  inside VALUE-receiver method:   &a = %p (a fresh copy)\n", &a)
}

func (a *Account) showPointerReceiverAddr() {
	fmt.Printf("  inside POINTER-receiver method: a  = %p (the original)\n", a)
}

// =============================================================================
// 03. ENCAPSULATION — CAPITALIZATION IS THE ONLY ACCESS CONTROL
// =============================================================================
//
// Go has NO public/private/protected keywords. The rule is a single, blunt
// convention enforced by the compiler:
//
//	Identifier starts UPPERCASE  -> EXPORTED   (visible from other packages)
//	Identifier starts lowercase  -> unexported (visible only within this package)
//
// CRUCIAL nuance vs Java/C++: visibility is PACKAGE-scoped, not TYPE-scoped.
// Any code in the SAME package can read/write an unexported field, even of a
// different type. There is no "private to this class" — the unit of privacy is
// the package directory, not the struct.

type wallet struct {
	Owner   string // Exported: other packages can read/write
	balance int64  // unexported: only code in package `main` can touch it
}

// NewWallet is a constructor-by-convention (see section 10) that lets outside
// packages create a wallet even though `balance` is hidden from them.
func NewWallet(owner string, opening int64) *wallet {
	return &wallet{Owner: owner, balance: opening}
}

// Balance is the exported "getter" — controlled read access to a hidden field.
func (w *wallet) Balance() int64 { return w.balance }

func demoEncapsulation() {
	hr("03. ENCAPSULATION — EXPORTED (Cap) vs unexported (lower)")

	w := NewWallet("Ada", 500)
	fmt.Printf("w.Owner   = %q   (Exported field — readable everywhere)\n", w.Owner)
	fmt.Printf("w.Balance() = %d (controlled access to the hidden `balance`)\n", w.Balance())

	// Because THIS code is in package main (same package as `wallet`), it can
	// still see `balance` directly. From ANOTHER package, `w.balance` would not
	// even compile. Privacy is per-PACKAGE, not per-type.
	w.balance += 1 // legal here; would be illegal from a different package
	fmt.Printf("same-package code CAN poke w.balance directly -> %d\n", w.balance)
	fmt.Println("Privacy unit = the package, not the type. No public/private keywords exist.")
}

// =============================================================================
// 04. INTERFACES — IMPLICIT SATISFACTION (climax #1)
// =============================================================================
//
// THE BIG IDEA: a type satisfies an interface simply by HAVING the required
// methods. There is no `implements` keyword, no declared relationship between
// the type and the interface. This is "structural typing" / "duck typing":
// if it has Area() and Perimeter(), it IS a Shape — nobody had to say so.
//
// THE MECHANISM (the part that wins arguments). An interface VALUE is not the
// concrete object. It is a TWO-WORD pair, 16 bytes on a 64-bit machine:
//
//	interface value (16 bytes):
//	  +----------------------+----------------------+
//	  |  *itab               |  *data               |
//	  |  (type + method tbl) |  (pointer to value)  |
//	  +----------------------+----------------------+
//	         word 0                  word 1
//
// The itab ("interface table") is built once per (interface type, concrete type)
// pair and contains: the concrete type's identity AND an array of function
// pointers — one per interface method, already resolved to the concrete
// implementation. A dynamic dispatch is therefore:
//
//	1. load itab pointer        (word 0)
//	2. load the method's fn ptr  (from a fixed slot in the itab)
//	3. CALL through that pointer  (an INDIRECT call), passing word 1 as receiver
//
// That is exactly the SHAPE of a C++ virtual call (load vptr -> load slot ->
// indirect call). The difference: a C++ vtable is bolted to a class hierarchy,
// while a Go itab is synthesized per (interface, type) pair with no hierarchy.

type Shape interface {
	Area() float64
	Perimeter() float64
}

// Three concrete types. NOTICE: none of them mentions `Shape`. They just happen
// to have the right methods, so they satisfy it automatically.

type Rectangle struct{ W, H float64 }

func (r Rectangle) Area() float64      { return r.W * r.H }
func (r Rectangle) Perimeter() float64 { return 2 * (r.W + r.H) }

type Circle struct{ R float64 }

func (c Circle) Area() float64      { return 3.141592653589793 * c.R * c.R }
func (c Circle) Perimeter() float64 { return 2 * 3.141592653589793 * c.R }

type Triangle struct{ A, B, C float64 } // three side lengths

func (t Triangle) Area() float64 {
	s := (t.A + t.B + t.C) / 2 // Heron's formula
	// crude sqrt via Newton to avoid importing math (keeps file dependency-free)
	x := s * (s - t.A) * (s - t.B) * (s - t.C)
	return newtonSqrt(x)
}
func (t Triangle) Perimeter() float64 { return t.A + t.B + t.C }

// newtonSqrt: tiny self-contained sqrt so we don't pull in the math package.
func newtonSqrt(x float64) float64 {
	if x <= 0 {
		return 0
	}
	g := x
	for i := 0; i < 30; i++ {
		g = 0.5 * (g + x/g)
	}
	return g
}

// pointerOnly demonstrates the method-set gotcha from section 02. Its only
// method has a POINTER receiver, so a *PointerOnly satisfies Shape but a plain
// PointerOnly value does NOT.
type PointerOnly struct{ Side float64 }

func (p *PointerOnly) Area() float64      { return p.Side * p.Side }
func (p *PointerOnly) Perimeter() float64 { return 4 * p.Side }

func demoInterfaces() {
	hr("03 -> 04. INTERFACES (IMPLICIT SATISFACTION)")

	// Polymorphism: a heterogeneous slice of Shape, each element a different
	// concrete type. The call site does not know or care which one it holds.
	shapes := []Shape{
		Rectangle{W: 3, H: 4},
		Circle{R: 2},
		Triangle{A: 3, B: 4, C: 5},
	}
	for _, s := range shapes {
		// s.Area() here is a DYNAMIC dispatch: load fn ptr from s's itab, indirect call.
		fmt.Printf("%-20T area=%6.2f perimeter=%6.2f\n", s, s.Area(), s.Perimeter())
	}

	// PROVE the two-word layout: an interface value is 16 bytes regardless of how
	// big the concrete type inside it is.
	var s Shape = Rectangle{W: 3, H: 4}
	fmt.Printf("\nunsafe.Sizeof(Shape value) = %d bytes  (= two 8-byte words: *itab, *data)\n", unsafe.Sizeof(s))
	fmt.Printf("unsafe.Sizeof(Rectangle)   = %d bytes  (the concrete payload it points at)\n", unsafe.Sizeof(Rectangle{}))

	// The method-set / pointer-receiver tie-back. *PointerOnly satisfies Shape;
	// a PointerOnly VALUE does not, because its methods need an address.
	var ok Shape = &PointerOnly{Side: 5} // works: *PointerOnly has the methods
	fmt.Printf("\n&PointerOnly{} satisfies Shape: area=%.2f\n", ok.Area())
	fmt.Println("but a plain PointerOnly VALUE does NOT satisfy Shape — its methods")
	fmt.Println("have pointer receivers, so they live in *PointerOnly's method set only.")
	fmt.Println("(`var bad Shape = PointerOnly{...}` would be a COMPILE error.)")
}

// =============================================================================
// 05. THE EMPTY INTERFACE: any / interface{}
// =============================================================================
//
// `interface{}` (spelled `any` since Go 1.18) requires ZERO methods, so EVERY
// type satisfies it. It is still the same two-word (type, data) pair — it just
// has an empty method table. This is the box Go used for "hold anything" before
// real generics existed. To get the concrete value back out you must ASSERT its
// type, which checks word 0 (the type info) at runtime.

func demoEmptyInterface() {
	hr("04 -> 05. EMPTY INTERFACE (any) + TYPE ASSERTIONS / SWITCH")

	var box any // alias for interface{}
	box = 42
	fmt.Printf("box now holds an int: %v (still a 16-byte two-word value: %d bytes)\n", box, unsafe.Sizeof(box))

	// (1) Plain assertion: x.(T) — PANICS if the stored type is wrong.
	n := box.(int)
	fmt.Printf("assertion box.(int) = %d\n", n)

	// (2) Comma-ok assertion: safe; ok=false instead of panicking.
	if s, ok := box.(string); ok {
		fmt.Println("box held a string:", s)
	} else {
		fmt.Println("comma-ok: box does NOT hold a string (ok=false, no panic)")
	}

	// (3) Type switch: branch on the dynamic type (word 0 of the pair).
	describe := func(v any) string {
		switch t := v.(type) {
		case int:
			return fmt.Sprintf("int:%d", t)
		case string:
			return fmt.Sprintf("string:%q", t)
		case Shape:
			return fmt.Sprintf("a Shape with area %.2f", t.Area())
		default:
			return fmt.Sprintf("unknown %T", t)
		}
	}
	fmt.Println("type switch ->", describe(7))
	fmt.Println("type switch ->", describe("hi"))
	fmt.Println("type switch ->", describe(Circle{R: 1}))
	fmt.Println("type switch ->", describe(3.14))

	fmt.Println("\nNote: `any` was the pre-1.18 'generics'. Real generics (type")
	fmt.Println("parameters, e.g. func Max[T constraints.Ordered](a, b T) T) exist now.")
}

// =============================================================================
// 06. EMBEDDING — COMPOSITION WITH METHOD PROMOTION (Go's "inheritance")
// =============================================================================
//
// You embed a struct by writing its TYPE with NO field name. The embedded
// struct's fields and methods are PROMOTED: you can reach them through the outer
// struct as if they were declared on it. At the byte level the embedded struct
// is simply laid out INLINE inside the outer one — it is composition, not magic.
//
// THIS IS NOT INHERITANCE. There is no "is-a", no subtype relationship, no
// virtual override of the embedded method. car.Start() works only because the
// compiler FORWARDS the call to car.Engine.Start(). The outer type does not
// participate in the embedded method's dispatch.

type Engine struct {
	Horsepower int
}

func (e Engine) Start() string {
	return fmt.Sprintf("engine starting (%d hp)", e.Horsepower)
}

func (e Engine) Stop() string { return "engine stopping" }

type Car struct {
	Engine // embedded: no field name -> fields & methods are promoted
	Brand  string
}

// "Override" by shadowing: define Start on the OUTER type. It hides the promoted
// Engine.Start for calls written as car.Start(). This is NOT virtual override;
// it is name shadowing resolved at compile time.
func (c Car) Start() string {
	// Go's closest thing to `super`: reach the embedded method EXPLICITLY by
	// naming the embedded field/type. This is the only "call the parent" move.
	inner := c.Engine.Start()
	return fmt.Sprintf("%s: %s, then lights on", c.Brand, inner)
}

func demoEmbedding() {
	hr("05 -> 06. EMBEDDING (composition + method promotion)")

	car := Car{Engine: Engine{Horsepower: 300}, Brand: "Tesla"}

	// Promotion: Stop() is not defined on Car, but is reachable via promotion.
	fmt.Println("car.Stop()  (promoted from Engine) ->", car.Stop())

	// Promoted FIELD: car.Horsepower really means car.Engine.Horsepower.
	fmt.Printf("car.Horsepower (promoted field)     -> %d\n", car.Horsepower)

	// Shadowing: Car.Start() shadows Engine.Start(); the outer one wins.
	fmt.Println("car.Start() (Car shadows Engine)   ->", car.Start())

	// "super": call the embedded version explicitly.
	fmt.Println("car.Engine.Start() (explicit inner) ->", car.Engine.Start())

	// Byte view: the Engine sits INLINE inside Car (offset 0 here), proving
	// embedding is plain memory composition.
	fmt.Printf("\nunsafe.Sizeof(Car) = %d, Offsetof(car.Engine) = %d (Engine is laid out inline)\n",
		unsafe.Sizeof(car), unsafe.Offsetof(car.Engine))
	fmt.Println("Embedding is composition, NOT inheritance: no is-a, no virtual override,")
	fmt.Println("the outer type merely FORWARDS to the inner one.")
}

// =============================================================================
// 07. INTERFACE EMBEDDING
// =============================================================================
//
// Interfaces can embed other interfaces. The composed interface requires the
// UNION of all embedded method sets. This is how the standard library builds
// io.ReadWriter out of io.Reader + io.Writer.

type Reader interface {
	Read(p []byte) (n int, err error)
}

type Writer interface {
	Write(p []byte) (n int, err error)
}

// ReadWriter requires BOTH Read and Write — it embeds the two smaller interfaces.
type ReadWriter interface {
	Reader
	Writer
}

// pipe satisfies ReadWriter by implementing both methods. (Toy implementation.)
type pipe struct{ buf []byte }

func (p *pipe) Read(out []byte) (int, error) {
	n := copy(out, p.buf)
	p.buf = p.buf[n:]
	return n, nil
}
func (p *pipe) Write(in []byte) (int, error) {
	p.buf = append(p.buf, in...)
	return len(in), nil
}

func demoInterfaceEmbedding() {
	hr("06 -> 07. INTERFACE EMBEDDING (ReadWriter = Reader + Writer)")

	var rw ReadWriter = &pipe{}
	_, _ = rw.Write([]byte("silicon"))
	out := make([]byte, 4)
	n, _ := rw.Read(out)
	fmt.Printf("wrote 'silicon', read back %d bytes: %q\n", n, out[:n])
	fmt.Println("ReadWriter embeds Reader+Writer; one type satisfies it by having both methods.")
}

// =============================================================================
// 08. POLYMORPHISM IS INTERFACE-BASED ONLY (mix interfaces + embedding)
// =============================================================================
//
// Go gives you exactly two tools and you combine them:
//   - INTERFACES  -> polymorphism (call the same method on different types)
//   - EMBEDDING   -> code reuse (share an implementation without inheriting)
//
// Below: a Named base provides a reusable Name() implementation. We embed it
// into two unrelated types, then treat both polymorphically through an interface
// they each satisfy.

type Named struct{ name string }

func (n Named) Name() string { return n.name }

type Greeter interface {
	Name() string
	Greet() string
}

type Dog struct {
	Named // embed for the reusable Name() method
}

func (d Dog) Greet() string { return d.Name() + " says woof" }

type Robot struct {
	Named // same reuse, totally unrelated type
}

func (r Robot) Greet() string { return "UNIT " + r.Name() + " online" }

func demoPolymorphism() {
	hr("07 -> 08. POLYMORPHISM (interfaces) + REUSE (embedding) together")

	crowd := []Greeter{
		Dog{Named{name: "Rex"}},
		Robot{Named{name: "B-9"}},
	}
	for _, g := range crowd {
		// g.Name() is promoted from embedded Named (reuse);
		// g.Greet() is dynamic dispatch through the interface (polymorphism).
		fmt.Printf("%-12T -> %s\n", g, g.Greet())
	}
	fmt.Println("No shared base CLASS — just a shared embedded struct + a shared interface.")
}

// =============================================================================
// 09. NO INHERITANCE — THE DESIGN DEBATE (your ammunition vs Java/C++)
// =============================================================================

func demoDesignDebate() {
	hr("08 -> 09. WHY GO HAS NO INHERITANCE (the argument)")

	essay := `Go deliberately omits implementation inheritance. The problems it dodges:

  * FRAGILE BASE CLASS: in Java/C++, changing a base class can silently break
    every subclass that overrode or depended on its internals. Behavior is
    coupled across a hierarchy you may not even control.
  * DEEP, BRITTLE HIERARCHIES: "A extends B extends C extends D" forces an
    is-a taxonomy you must get right up front, and refactoring it is painful.
  * THE DIAMOND PROBLEM: multiple inheritance ("A inherits B and C, both inherit
    D") creates ambiguous method resolution and duplicated state. C++ needs
    virtual inheritance to patch it; Java bans multiple class inheritance.
  * IMPLEMENTATION COUPLING: subclassing inherits IMPLEMENTATION, not just a
    contract, so subclasses break when the parent's internals change.

Go's bet instead:
  * COMPOSITION OVER INHERITANCE: embed structs to REUSE behavior; the outer
    type forwards calls. No is-a, no override surprises, no diamond.
  * SMALL, IMPLICIT INTERFACES: interfaces are usually 1-2 methods and are
    defined by the CONSUMER, not the producer. You write the interface where you
    USE it; any existing type that has the methods already satisfies it.
  * "ACCEPT INTERFACES, RETURN STRUCTS": functions take the narrowest interface
    they need (flexible input) and return concrete structs (so callers get the
    full, well-defined type). Decoupling at the boundary, concreteness inside.

The counter-argument a Java/C++ dev will raise: inheritance gives you free code
reuse + dynamic dispatch in one mechanism, and frameworks (templates, abstract
base classes) lean on it. Go's reply: those two concerns SHOULD be separate, and
splitting them (embedding for reuse, interfaces for dispatch) avoids the coupling
that makes large inheritance trees fragile.`

	fmt.Println(essay)
}

// =============================================================================
// 10. CONSTRUCTORS (none!) + ZERO-VALUE-USEFUL DESIGN
// =============================================================================
//
// Go has no constructors and no destructors. Conventions instead:
//   - Construction: a plain function `NewT(...) *T` (or `T`) that builds and
//     validates the value. It is ordinary code, not a special language feature.
//   - Zero value: a freshly declared `var b Buffer` has all fields zeroed. A
//     well-designed type is USABLE at its zero value with no constructor at all
//     (the std lib's bytes.Buffer and sync.Mutex are the famous examples).
//   - Cleanup: there are no destructors. GC reclaims MEMORY automatically. For
//     non-memory resources (files, sockets) you call Close(), usually via
//     `defer`, which runs the cleanup when the function returns.

// Buffer is a zero-value-useful type: you can `var b Buffer` and immediately
// Write to it. No NewBuffer() required (though we provide one to show the
// convention).
type Buffer struct {
	data []byte // nil slice is a valid empty buffer — that's the zero value
}

func (b *Buffer) Write(s string) { b.data = append(b.data, s...) }
func (b *Buffer) String() string { return string(b.data) }

// NewBuffer is the constructor-by-convention (optional here, since zero value works).
func NewBuffer(initial string) *Buffer {
	return &Buffer{data: []byte(initial)}
}

// resource models something needing explicit cleanup (no destructor in Go).
type resource struct{ name string }

func openResource(name string) *resource {
	fmt.Printf("  open  %s\n", name)
	return &resource{name: name}
}
func (r *resource) Close() { fmt.Printf("  close %s (via defer, not a destructor)\n", r.name) }

func demoConstructors() {
	hr("09 -> 10. CONSTRUCTORS (NewT convention) + ZERO-VALUE-USEFUL")

	// Zero value works with no constructor at all:
	var b Buffer // all fields zeroed; data is a nil slice = empty buffer
	b.Write("zero-value ")
	b.Write("buffer works")
	fmt.Printf("zero-value Buffer: %q\n", b.String())

	// Constructor-by-convention:
	b2 := NewBuffer("made by NewBuffer")
	fmt.Printf("NewBuffer result : %q\n", b2.String())

	// Cleanup via defer/Close, not destructors. GC handles the memory.
	func() {
		r := openResource("db-conn")
		defer r.Close() // runs when this inner func returns — deterministic cleanup
		fmt.Printf("  using %s...\n", r.name)
	}()
	fmt.Println("No destructors: GC frees memory; defer/Close frees other resources.")
}

// =============================================================================
// 11. RECAP TABLE — Java/C++ OOP term -> Go equivalent
// =============================================================================

func demoRecapTable() {
	hr("11. RECAP: Java/C++ OOP  ->  Go equivalent")

	rows := [][2]string{
		{"class", "struct"},
		{"method", "method with a receiver arg (explicit `this`)"},
		{"public / private / protected", "Exported (Cap) / unexported (lower), package-scoped"},
		{"inheritance (extends)", "embedding (composition + method promotion)"},
		{"interface (explicit implements)", "interface (implicit / structural satisfaction)"},
		{"abstract class", "interface"},
		{"virtual dispatch (vtable)", "interface itab dispatch (two-word value, indirect call)"},
		{"constructor", "NewX(...) function by convention"},
		{"destructor", "defer / Close() + GC (no destructors)"},
		{"super.method()", "Outer.Embedded.Method() (explicit forward)"},
	}
	fmt.Printf("%-34s | %s\n", "JAVA / C++", "GO")
	fmt.Println(strings.Repeat("-", 34) + "-+-" + strings.Repeat("-", 45))
	for _, r := range rows {
		fmt.Printf("%-34s | %s\n", r[0], r[1])
	}
}

// =============================================================================
// MAIN — run every section in order; close with the mental model.
// =============================================================================

func main() {
	fmt.Println("GO 'OOP' FROM THE SILICON UP — run output below.")
	fmt.Printf("(64-bit machine: a pointer is %d bytes, so an interface value is %d bytes.)\n",
		unsafe.Sizeof(uintptr(0)), unsafe.Sizeof(any(nil)))

	demoStructBytes()
	demoReceivers()
	demoEncapsulation()
	demoInterfaces()
	demoEmptyInterface()
	demoEmbedding()
	demoInterfaceEmbedding()
	demoPolymorphism()
	demoDesignDebate()
	demoConstructors()
	demoRecapTable()

	hr("MENTAL MODEL — keep these five lines")
	fmt.Println(`1. A struct is just bytes: ordered fields + alignment padding. No class.
2. A method is a function whose first arg is the receiver (explicit ` + "`this`" + `);
   value receiver = copy (can't mutate), pointer receiver = address (can mutate).
3. An interface value is TWO words (16 bytes): *itab (type + method fn-ptrs) and
   *data. Dynamic dispatch = load fn-ptr from itab, indirect call. Like a vtable,
   but built per (interface, concrete-type) pair with NO class hierarchy.
4. Satisfaction is IMPLICIT/structural: have the methods => you implement it.
5. No inheritance: COMPOSITION (embedding) for reuse, INTERFACES for polymorphism.
   Encapsulation = capitalization, package-scoped. Cleanup = defer/Close + GC.`)
}
