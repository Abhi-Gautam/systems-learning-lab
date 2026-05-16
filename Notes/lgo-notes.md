# LGO Notes

_Entries follow the template at `Notes/TEMPLATE.md`. Append-only. **Newest entry at top**, immediately after this header._

---

## [2026-05-15] `const`, Untyped Constants, and Why Go Refuses Runtime Immutability · pp.51–60 · Ch.2 § var vs := → Ch.3 § Arrays

### TL;DR
Go's `const` keyword **only names compile-time literals** — numeric literals, strings, runes, booleans, and `iota`-driven expressions over them. There is no `const` for a struct field, a slice element, a map value, or anything computed at runtime. The chapter's most important sentence is: **"Constants in Go are a way to give names to literals. There is no way in Go to declare that a variable is immutable."** This is deliberate — Bone et al. argue that within a function, mutation is locally obvious from reading top-to-bottom, and Go's call-by-value semantics (covered next chapter) prevent caller state from being mutated by callees. The chunk also nails down the **`:=` vs `var` decision tree** (use `:=` inside functions, `var` for zero-value declarations and package level), establishes the **unused-variable compile error** rule, and ends with the rigid `[N]T` array type that explains why slices — not arrays — are Go's everyday sequence.

### History — "why does this exist?"
Go's `const` design is a **direct rejection of three older approaches**: C's `#define` macros (textual, type-unsafe, scope-blind), C++'s `const` (which conflates "immutable binding," "immutable through this pointer," and "compile-time constant" — three concepts the chapter notes are different), and Java's `final` (which is runtime, not compile-time, and famously doesn't make fields deeply immutable — `final List<X>` is still mutable). **Pike, Thompson, and Griesemer's choice in 2007** was the minimal one: `const` means "compile-time literal name," nothing more. The **untyped constant** mechanism — where `const x = 10` has no type until used — is borrowed conceptually from **Algol 68** and **Modula-3** and lets `const Pi = 3.14159` be assigned to both `float32` and `float64` variables without conversion. Russ Cox's blog post "The Laws of Reflection" (2011) and Rob Pike's "Go at Google: Language Design in the Service of Software Engineering" (2012) both cite the same reason: **"every feature has to pay for itself"** — and full runtime immutability didn't pay enough to justify the type-system complexity it would have required. By contrast, **Rust (2010)** spent its entire complexity budget on borrow-checking and immutability; Go spent it elsewhere (goroutines, GC, fast compiles). The languages are aimed at different problems.

### Intuition — "this is like…"
Go's `const` is the difference between a **brass nameplate on a door** and a **deadbolt on the door**. The brass nameplate (`const`) is bolted on at *fabrication time* — you can read it, refer to it, build other plates that reference it, and nobody can change what's etched into the brass. But it doesn't lock the door. The deadbolt (runtime immutability — Rust's `&T`, Java's `final` for objects) actually prevents *the thing behind the door* from changing. Go ships brass nameplates, not deadbolts. The argument the chapter makes for this: within one function, you can *see* the deadbolt's job being done by your own eyes — you just read the code top to bottom. Across function boundaries, Go uses **call-by-value** as a softer substitute: the callee can't modify the caller's variables, period, because the callee got a *copy*.

### Mechanics

**The `:=` vs `var` decision matrix:**

| Situation | Use | Why |
|---|---|---|
| Inside function, you want to declare AND assign | `x := 10` | Idiomatic; the most common shape |
| Inside function, zero value intended | `var x int` | Makes the "I want the zero" choice visible |
| Inside function, untyped literal but you want a specific non-default type | `var x byte = 20` | `:=` would give `int`; `var` lets you pick |
| Inside function, want to make "new variable" explicit (shadowing risk) | `var x int = 10; x = newValue` | `:=` can silently create a new var inside a block |
| Package level | `var x = ...` | `:=` is illegal outside functions |
| Naming a literal you want to refer to | `const x = 10` | Compile-time, no storage |

**The `:=` shadowing trap (the reason "explicit var when in doubt" exists):**

```go
err := doThing()
if err != nil {
    err := otherThing()   // ← BUG: shadows outer err; the outer err is silently unused
    log.Print(err)
}
return err                // returns the OUTER err — possibly nil — even though inner err was set
```

This is the most common Go bug `go vet` and `golangci-lint`'s `govet -shadow` flag exists to catch. The chapter's defense of `var` in mixed-old-and-new contexts is squarely aimed at this.

**The untyped-constant superpower:**

```go
const Pi = 3.14159          // untyped float constant

var x float32 = Pi          // OK — Pi adopts float32
var y float64 = Pi          // OK — Pi adopts float64
var z complex128 = Pi       // OK — Pi adopts complex128

const TypedPi float64 = 3.14159
var w float32 = TypedPi     // COMPILE ERROR — cannot use float64 as float32
```

The untyped form is the **escape hatch from Go's no-implicit-conversion rule** (covered in the previous entry's note on numeric types). Without it, every literal in the language would need explicit conversions; with it, `const` and literals compose cleanly across numeric types.

**Unused-variable rule and its exceptions:**

```go
func f() {
    x := 10                 // COMPILE ERROR if x is never read

    const y = 20            // OK — unused consts are fine
                            //  (they compile away to nothing)

    x = 30                  // OK at compile time — but golangci-lint flags
    _ = x                   // "ineffectual assignment" via the ineffassign linter
}
```

Two surprises here: **(1)** Go's compiler enforces unused *variables* but not unused *assignments* — `x := 10; x = 20; fmt.Println(x)` compiles, even though the `10` is dead. The linter catches it, the compiler doesn't. **(2)** Package-level vars are *not* checked — `var globalDebug = false` at package scope can be unused forever. This (combined with the data-flow argument) is why the chapter says: **avoid package-level vars wherever possible**.

**The trapdoor into Ch.3: rigid arrays.**

```go
var x [3]int                   // type is "[3]int" — size is PART OF the type
var y [4]int                   // type is "[4]int" — DIFFERENT type from x
// x = y                       // compile error: type mismatch
```

Because `[3]int` and `[4]int` are distinct types, you cannot write a function that accepts "an array of any length" — you'd need generics (added in Go 1.18) or, far more commonly, **slices** `[]int` which carry length at runtime. This single design choice is why slices exist and why arrays are rarely seen in idiomatic Go.

### If you were the language designer…
You're told: "add real immutability to Go." Two paths: **(A)** Make `const` work on runtime values — `const u User = User{Name: "Bob"}`, requiring the compiler to verify no mutation of `u.Name`. **(B)** Add a `let`-style binding like Rust's `let` vs `let mut`. Both paths require **flow-sensitive analysis** through every aliasing path (pointers, slice elements, map values, channel sends) — exactly the analysis Rust spent a decade building. Go's team made the call that the engineering cost would dwarf the benefit, given that Go programs are mostly *small functions with short variable lifetimes* where mutation is locally obvious. The trade-off is real: Go programs do have classes of bugs Rust prevents (concurrent map writes, accidental aliasing). The Go answer is `go vet`, `go race`, and code review.

### Cross-language view
```rust
// Rust — runtime immutability is the default
let x = User { name: "Bob".to_string() };
x.name = "Alice".to_string();         // COMPILE ERROR — x is immutable
let mut y = User { ... };
y.name = "Alice".to_string();         // OK — mut is required to mutate
```
```go
// Go — no runtime immutability; reliance on call-by-value + convention
x := User{Name: "Bob"}
x.Name = "Alice"                     // OK — Go has no immutable bindings

const greeting = "hello"             // OK — compile-time literal
// const u = User{Name: "Bob"}       // COMPILE ERROR — not a literal
```
```java
// Java — `final` is a binding lock, not a deep immutability lock
final List<String> xs = new ArrayList<>();
xs.add("hi");                        // OK — the list is mutable
// xs = new ArrayList<>();           // COMPILE ERROR — can't rebind
```
```python
# Python — no const at all; convention is UPPER_CASE
MAX_SIZE = 100                       # mutable by anyone who imports it
MAX_SIZE = 200                       # legal
```

### Where this shows up in real systems
- **`iota` for enums.** Go's idiomatic enum is `const ( Red = iota; Green; Blue )`. The chapter foreshadows this — `iota` is the only language feature that makes `const` blocks *more* than just named literals, by generating sequential integer values at compile time. Kubernetes, etcd, and the standard library are saturated with this pattern.
- **`golangci-lint`'s `ineffassign` and `unused` checks.** Every Go CI pipeline in production runs these — they patch the gap between Go's narrow compiler check ("variable was read at least once") and the actual goal ("no dead code"). The chapter explicitly recommends this tooling.
- **The shadow bug in `errgroup` callers.** Go's `golang.org/x/sync/errgroup` example code shows the `err := g.Wait()` pattern; if you wrote `err := g.Wait()` inside an `if` block that shadows an outer `err`, your error gets eaten. This is the production version of the snippet in the Mechanics section.

### Diagnostic questions
1. **Q:** Can `const x = time.Now()` compile in Go?
   *Wrong-answer trap:* "Yes, with a flag." No — `time.Now()` is a function call, evaluated at runtime. `const` requires a value the compiler can compute *before* the program runs. The error is `const initializer time.Now() is not a constant`.
2. **Q:** Why is `var x = 10` legal at package level but `x := 10` is not?
   *Wrong-answer trap:* "Syntactic limitation." Real reason: `:=` is short for "declare AND assign in one step inside a scope that has a sequence of statements." Package level is a *declaration list*, not a sequence — no statements, no `:=`.
3. **Q:** What's the practical difference between `var x int` and `var x = 0`?
   *Wrong-answer trap:* "None." Both produce a zero int. The *signal* differs: `var x int` says "I want the zero value of the type." `var x = 0` says "I want zero specifically and I happen to be using int." Idiomatic Go uses the first form to make the "zero value is intended" choice readable.
4. **Q:** Why are unused package-level variables not a compile error?
   *Wrong-answer trap:* "Oversight." It's deliberate — package-level state often supports reflection, conditional compilation, or `init()` side-effects (e.g., registering a database driver). Compile-time checking would break those patterns.

### See also
- LGO previous entry (2026-05-14) — explicit type conversion, the rule untyped constants relax for literals only.
- DSG Ch.1 — Go service code where `const` is used for protocol version numbers and `iota` for log record types.
- TPP Topic 22 "Working with Text Streams" — the same "names for literals" principle, generalized.

---

## [2026-05-14] Explicit Type Conversion — Why Go Refuses Auto-Promotion · pp.41–50 · Ch.2 § Numeric Types → § var vs :=

### TL;DR
Go's numeric type system gives you a buffet (`int8`/16/32/64, unsigned variants, `float32`/64, `complex64`/128, plus the platform-sized `int`/`uint` and the aliases `byte`/`rune`/`uintptr`) — and then makes the buffet *strict*: **no implicit conversion between any two of them, ever**. Adding an `int` to an `int32` is a compile error; converting `int → float64` requires `float64(x)`. The chapter argues this is not pedantry but a deliberate response to **50 years of C-style implicit promotion bugs** — signed/unsigned silent comparisons, narrowing without warning, integer overflow on widening. The price is mild verbosity at type boundaries; the payoff is that *every* type change is visible in the source, so reviewers and tools can flag suspicious conversions instead of trusting that the compiler did the right invisible thing.

### History — "why does this exist?"
C's promotion rules (Kernighan & Ritchie, 1978) were designed when **PDP-11 register layout dictated the type system** — `int` was 16 bits because that's what the registers were, and promoting `char`/`short` to `int` for arithmetic was a hardware optimization disguised as a language rule. The rules became a bug source the moment portable code crossed word-size boundaries (the 1990s 16→32 bit transition, the 2000s 32→64 bit transition). C++ inherited every C rule and added its own (function overloading + user-defined conversions = the *worst* implicit conversion graph in mainstream languages). Java (1995) explicitly forbade narrowing implicit conversions but kept widening, and then accidentally re-introduced category errors via autoboxing (1.5, 2004 — `Integer == Integer` doing reference equality silently). When **Rob Pike, Ken Thompson, and Robert Griesemer** designed Go (announced Nov 2009), they explicitly said: "C's implicit conversions are a worse problem than verbosity." The result is the rule on p.48: *no automatic type promotion between variables*, even between same-category types of different sizes. The companion design — `untyped constants` (covered in the previous entry) — is what makes the strictness ergonomically tolerable.

### Intuition — "this is like…"
Auto-promotion is like a translator at a UN meeting who silently rephrases each delegate's words into "what they probably meant." It's convenient until the day they translate "we cannot agree" as "we can agree" and start a war. Go's design is: **every translation is explicit, recorded, and signed by the author.** If you wanted to convert dollars to cents, you wrote `int(d * 100)` — and if that loses precision, that's on you, in your code, where the reviewer can see it. The verbosity isn't aesthetic; it's an *audit trail* for every type-crossing.

### Mechanics

**The full Go integer family (Table 2-1, plus aliases):**

```
 Sized signed:    int8   int16   int32   int64
 Sized unsigned:  uint8  uint16  uint32  uint64
 Platform-sized:  int    uint    uintptr     ← 32 or 64 bits, set by GOARCH
 Aliases:         byte = uint8     rune = int32
```

**The strictness rule, made concrete (Example 2-2):**

```go
var x int     = 10
var y float64 = 30.2

// var z = x + y           // compile error: mismatched types int and float64
var z float64 = float64(x) + y     // OK — convert x explicitly
var d int     = x + int(y)         // OK — convert y explicitly (truncates 30.2 → 30)

// Even same-category, different-size is forbidden:
var a int32 = 1
var b int64 = 2
// var c = a + b           // compile error
var c int64 = int64(a) + b
```

**Choosing which integer to use (p.42, the three-rule table):**

| Situation | Use |
|---|---|
| Binary file format or network protocol with fixed-width fields | The matching sized type (`uint16` for a 16-bit field, etc.) |
| Library function meant to accept any integer | A pair of functions on `int64` and `uint64` (no generics in Go pre-1.18; the chapter predates them) |
| Everything else | `int` |

The default-to-`int` rule is a deliberate *anti-premature-optimization* nudge: most code doesn't care about word size, and using `int32` "to save memory" almost never saves measurable memory while costing readability and conversion friction.

**Floating-point: the "don't use these for money" warning.**

IEEE 754 `float64` stores `-3.1415` as exactly `-3.14150000000000018118839761883…`. The chapter spells this out because every senior dev has debugged a currency display showing `$10.000000004` once. The Go-specific rules:
- `==`/`!=` on floats is *allowed* but you almost never want it; use an epsilon: `math.Abs(a-b) < eps`.
- `0.0 / 0.0` returns `NaN`; nonzero `/ 0.0` returns `±Inf`. Both are real `float64` values (no panic).
- Integer `/ 0` *panics*. (The asymmetry is because integer division has no NaN representation; float math does.)

**The five primitive-literal categories (closing the count from the previous entry):**

```
1. Integer literals          42, 0x2A, 0b101010, 0o52
2. Floating-point literals   3.14, 6.022e23
3. Rune literals             'a', '\n', 'é'
4. String literals           "hello"  or  `raw\nstring`
5. Imaginary literals        2.5i, 3i              ← the fifth, only revealed on p.47
```

**Variable declaration: five forms, one rule.**

```go
var x int = 10        // explicit type + value          — verbose, useful at package level
var x = 10            // value, type inferred          — common
var x int             // type, zero-valued              — common when filling later
var x, y int = 10, 20 // multi-decl, same type
var (                 // declaration block — package-level convention
    x int
    y     = 20
    z int = 30
)
x := 10               // short form: function-scope only, type inferred
```

The rule: **at package scope, use `var`; inside functions, prefer `:=` unless you specifically need a zero value or an explicit type**. The book's stated reason — each form *communicates* something to the reader about how the variable will be used.

### If you were the compiler…

You see `func median(a, b int32) float64 { return (a + b) / 2 }`. What do you do?

**You reject the program.** `a + b` is `int32`, dividing by the *untyped* constant `2` is fine (untyped constants take the operand's type — `int32` here), but the result is `int32`, not `float64`. The fix the compiler *won't* invent for you: `return float64(a+b) / 2` (now `2` is typed as `float64`). The bug it prevents: silently truncating `(3 + 4) / 2 = 3` when the caller expected `3.5`. C, Java, and Python would all silently do the wrong thing in some configuration; Go forces the author to *write down* whether they meant integer or float division. **Verbosity here is the price of correctness at the boundary.**

### Cross-language view

The same conversion, four idioms:

```c
// C — silent widening, silent narrowing, signed/unsigned promotion bugs galore
int   x = 10;
float y = x;          // OK, implicit widen
int   z = y;          // OK, silent truncate — no warning by default
```
```cpp
// C++ — same as C, plus user-defined conversions and overload resolution noise
int x = 10; double y = x; int z = y;   // all silent
```
```rust
// Rust — like Go: no implicit numeric conversion at all
let x: i32 = 10;
let y: f64 = x as f64;     // 'as' is the explicit cast keyword
let z: i32 = y as i32;     // also explicit; saturating behavior since Rust 1.45
```
```go
// Go — the chapter's rule
var x int     = 10
var y float64 = float64(x)
var z int     = int(y)
```

What the stdlib actually does: `strconv.FormatInt`/`ParseInt` work on `int64` because that's the chapter's "library function on any integer" idiom — callers pass `int64(x)` going in, then convert back coming out. The verbosity *is* the API contract.

### Where this shows up in real systems

- **The Therac-25 / Ariane 5 family of bugs.** Ariane 5's 1996 maiden-flight failure was a 64-bit float silently converted to a 16-bit signed int — overflow, exception unhandled, rocket destroyed, $370M lost. A Go-style strict type system would have flagged the conversion at compile time; Ada's strict typing *did* flag it, but the engineers had disabled the check for performance.
- **Postgres wire protocol decoders.** Drivers like `pgx` read fixed-width network bytes into Go sized integers (`int32` for `INT`, `int64` for `BIGINT`). The strict conversion rules are exactly what you want here: the type system prevents a `BIGINT` value from being silently truncated into an application's `int32` field.
- **JSON unmarshaling and the `json.Number` type.** Go's `encoding/json` decodes numeric JSON into `float64` by default — which loses precision above 2^53. The fix (`d.UseNumber()` → `json.Number`) returns a string-backed type that requires *explicit* conversion to `int64` or `float64`. This is the chapter's design philosophy at runtime: make precision-losing conversions visible at the call site.
- **`go vet` and `gosec` rules.** `G115` (integer overflow conversion `int → int32`) and the `gosec` integer-overflow checks exist *only because* the compiler enforces strict conversion — the checker can then audit every explicit conversion for overflow potential. In C, no such check is possible because conversions are everywhere implicit.

### Diagnostic questions

1. **Q:** Why does `var x int = 10; var y int32 = x` fail to compile, when both are 32+ bit signed integers on most platforms?
   *Wrong-answer trap:* "Because the sizes might differ on some platform." The compiler doesn't reason about *might* — it enforces the type-name rule: `int` and `int32` are *different types* even when they have the same representation. The reason: code that compiles on amd64 must compile identically on amd64p32; the compile-time error catches the bug *before* it bites on the cross-compile.

2. **Q:** Why is `byte` an alias for `uint8` while `rune` is an alias for `int32`?
   *Wrong-answer trap:* "Convenience." `byte` is unsigned because raw byte arithmetic (XOR, shifts, comparisons against `0xFF`) is cleaner unsigned. `rune` is *signed* because Unicode code points top out at `U+10FFFF` (well below `int32` max) and signing leaves room for sentinel values like `-1` from `utf8.DecodeRune` for "invalid input." The signedness choice is a domain hint.

3. **Q:** Why does Go panic on integer `/ 0` but return `±Inf` on float `/ 0.0`?
   *Wrong-answer trap:* "Floats are special." Integer types have *no representation* for infinity or NaN, so the only options are panic or undefined behavior; Go chose panic. IEEE 754 floats *do* have those representations, so returning `Inf` is a valid result, not an error. The asymmetry is a type-system consequence, not a design preference.

4. **Q:** When *should* you prefer `int32` or `int64` over `int`?
   *Wrong-answer trap:* "When you want a smaller integer." Memory is rarely the constraint. The real triggers: (a) wire/file format with a fixed-width field, (b) you're indexing into a `[]int32` exposed by some external API, (c) you genuinely need overflow-at-2^31 semantics (rare). Outside those, `int` is correct.

5. **Q:** Why is `==` allowed but discouraged on `float64`, while it's outright forbidden on Go maps and slices?
   *Wrong-answer trap:* "Because float equality is unreliable." The deeper reason: Go's `==` is defined on types with a *well-defined identity*. Floats have one (bit pattern equality) — it's just usually not the *semantic* equality you want. Maps and slices have no canonical identity (two maps with the same contents can have different internal hash layouts), so the language refuses to commit. The difference is "well-defined but wrong tool" vs. "not well-defined at all."

### See also
- [lgo-notes.md](Notes/lgo-notes.md) 2026-05-13 — *Untyped Literals* — the *companion* rule: untyped constants exist precisely so that strictness at variable boundaries doesn't extend to every integer literal in the source.
- [cod-notes.md](Notes/cod-notes.md) 2026-05-14 — *The Translation Stack* — Go's `int` sizing across GOARCH values is a concrete ABI consequence of the ISA-as-contract principle (see that entry's Diagnostic Q4).
- [dsg-notes.md](Notes/dsg-notes.md) — distributed services constantly cross the wire/Go boundary; the explicit conversion rule is the chapter most directly relevant to writing safe protocol decoders.
- [welc-notes.md](Notes/welc-notes.md) — Feathers' "sensing variables" technique relies on every type-change being visible; Go's strictness makes that visibility free.

---

## [2026-05-13] Untyped Literals — Go's One Concession to Convenience · pp.31–40 · Ch.1 § Playground/Makefiles → Ch.2 § Literals

### TL;DR
Go is famously strict about types — you cannot add an `int32` to an `int64` without an explicit conversion — but it makes **one calculated exception**: numeric and string *literals* are **untyped** until the surrounding context forces a type on them. This is why `var x float64 = 3` compiles even though `3` looks like an `int`; the `3` has no type yet, and the assignment context types it as `float64`. The mechanism keeps the syntax convenient without weakening the type system, because the untyped-ness is **erased at the boundary** — the moment a literal participates in an expression with a *typed* operand, it adopts that operand's type, and any size or category mismatch is a compile error.

### History — "why does this exist?"
The design comes directly from **Go's reaction to C's implicit promotion rules** (1972) and Java's autoboxing (1996). In C, `int + float` silently promotes the `int`, and 50 years of subtle bugs followed (signed/unsigned comparisons, narrowing conversions, integer overflow on promotion). Java added autoboxing for ergonomics and got a different set of bugs (`Integer == Integer` doing reference equality silently). When Rob Pike, Ken Thompson, and Robert Griesemer designed Go (announced **November 10, 2009** — the date the Playground's clock is frozen at, p.33), they wanted **C's compile-time discipline without C's implicit conversions**. Untyped constants are the compromise: literals are convenient like in a dynamic language, but the convenience is purely a *compile-time* phenomenon — once the compiler picks a type, normal Go strictness resumes. The full design is in the spec section "Constants" and Pike's 2014 blog post "Constants," which is the canonical text on this feature.

### Intuition — "this is like…"
Untyped literals are like **a guest at a dinner party who hasn't picked a chair yet**. They can sit anywhere — int seat, float seat, complex seat — and once they sit, they're committed. But until they sit, the host (compiler) doesn't get to assume which seat is theirs. Contrast with C, where the guest always brings their own chair (the default type of an integer literal is `int`, full stop) and you have to carry it through every promotion rule.

### Mechanics

**The five literal categories Go recognizes (pp.37-39):**

| Category | Examples | Default type when forced |
|----------|----------|--------------------------|
| **Integer literals** | `42`, `0xff`, `0b1010`, `0o777`, `1_000_000` | `int` |
| **Floating-point literals** | `3.14`, `6.03e23`, `0x1p10` | `float64` |
| **Rune literals** | `'a'`, `'\n'`, `'a'`, `'\x61'` | `rune` (alias for `int32`) |
| **Interpreted string** | `"hello\n"` (escapes processed) | `string` |
| **Raw string** | `` `hello\n` `` (escapes literal, backticks) | `string` |
| *(5th: complex)* | `2 + 3i` | `complex128` |

**The "untyped" mechanism (the actual rule):**

```
Compile-time rule: a literal's type is determined by its context.

1. If the literal is the entire RHS of `var x T = literal`,
   the literal takes type T.
   → must be representable in T, else compile error.

2. If the literal is used in an expression with a typed operand,
   the literal takes that operand's type.
   → `var x int32 = 5; y := x + 100` → 100 becomes int32.

3. If the literal is used in an expression with only other untyped
   literals, the result stays untyped until step 1 or 2 applies.
   → `const big = 1<<63` is fine even though it overflows int64,
      because it's still untyped — the overflow only matters at
      the moment a type is forced.

4. If neither 1, 2, nor 3 forces a type by the time the value must
   exist at runtime (e.g., `fmt.Println(100)`), the literal falls
   back to its default type (int / float64 / rune / string).
```

**Worked example — what the textbook is really showing on pp.39-40:**

```go
var x int32 = 5
var y int64 = 10

// x + y // ✗ compile error: mismatched typed operands
x + 100  // ✓ 100 is untyped, takes x's type → int32
y + 100  // ✓ 100 is untyped, takes y's type → int64
1 + 2    // ✓ result is untyped — still "an integer-ish thing"

const huge = 1 << 100      // ✓ untyped const, value fits in arbitrary-precision
var i int64 = huge          // ✗ overflows int64 at the moment a type is forced
var f float64 = huge        // ✓ representable in float64 (with rounding)
```

The last three lines are the punch line: `huge` is **legal to declare** because untyped constants have arbitrary precision; the legality of *using* it depends entirely on the destination type. Go is reproducing the symbolic-math feel of Python or Mathematica at compile time, then collapsing back to fixed-width machine types at the boundary.

**Where the typed/untyped boundary is — a diagram:**

```
                  ┌──────────────────────────────┐
                  │      Untyped literals        │
                  │   (arbitrary precision,      │
                  │   no runtime representation) │
                  └──────────────┬───────────────┘
                                 │  context forces a type
                                 ▼
                  ┌──────────────────────────────┐
                  │       Typed values           │
                  │  (int8/int32/int64/float64…) │
                  │  Strict: no implicit promotion│
                  └──────────────────────────────┘
```

Everything below the line obeys Go's strict typing rules. Everything above the line is, for design purposes, **a different small language** — a compile-time arithmetic engine with no runtime existence.

### If you were the Go compiler, what would you do with `var f float32 = 0.1`?

You'd:
1. See the literal `0.1`. Tag it untyped, value = the exact rational `1/10`.
2. See the destination type `float32`. Attempt to represent `0.1` exactly in `float32`'s 23-bit mantissa.
3. Discover that `0.1` is **not exactly representable** in any binary float — but it *is* representable approximately, and Go's spec says "if the value is representable with at most an unspecified rounding," allow it.
4. Insert the IEEE-754 rounded `float32` value (`0x3DCCCCCD`) into `f`.

The interesting part is step 3: Go does *not* require exact representability for floats (unlike for ints), so `var f float32 = 0.1` compiles with silent rounding, but `var i int8 = 200` **does not** compile (200 doesn't fit in int8, exact representability is required). The asymmetry is deliberate and matches IEEE-754's expectations.

### Cross-language view

```c
// C — literals have a default type at parse time
int  x = 5;       // 5 is type 'int' at lex
long y = 5;       // 5 is type 'int', implicitly promoted to long
long z = 1<<63;   // UB or implementation-defined — 1 is int
long w = 1L<<63;  // OK — you carry the suffix yourself

// Rust — strict, like Go, but solves it with type inference + suffixes
let x: i32 = 5;       // OK
let y       = 5;      // type inferred from later use, else defaults to i32
let z       = 5u64;   // suffix locks the type at lex
let w: i32  = 5_i64;  // ✗ compile error — explicit conflict

// Python — runtime types, literals are objects
x = 5            # type(x) is int (arbitrary precision since Py3)
y = 5.0          # type(y) is float (C double)
z = 1 << 1000    # fine — int is arbitrary precision at runtime

// Go — untyped at compile time, fixed at boundary
var x int32 = 5      // 5 is untyped → int32
y := 5               // no context → default → int
const z = 1 << 100   // untyped, arbitrary precision
var i int64 = z      // ✗ overflows int64 — caught at compile time
```

What the stdlib actually does: `fmt.Println(5)` makes `5` adopt its default type `int`; `math.Sqrt(2)` makes `2` adopt `float64` because `math.Sqrt`'s parameter is `float64`. You almost never *see* the untyped-ness in real code — that's the design intent. It exists to make the strict type system feel ergonomic, not to be a feature you reach for.

### Where this shows up in real systems

- **`time.Sleep(100 * time.Millisecond)` works without conversions.** `100` is untyped, `time.Millisecond` is typed `time.Duration` (which is `int64`-based). The literal adopts `Duration`, the multiplication is `Duration * Duration`, and the result fits the parameter. In C++ you'd need `100ms` (C++14 user-defined literals) or `std::chrono::milliseconds(100)`. The Go version reads cleaner *only because* the literal is untyped.
- **Bitmask constants in `os` and `syscall` packages.** Flags like `os.O_RDWR | os.O_CREATE | os.O_TRUNC` work because each constant is an untyped integer constant; the `|` happens at compile time at arbitrary precision; the result is forced to `int` when passed to `os.OpenFile`. No casts needed.
- **Common bug it prevents: silent narrowing.** In C, `int8_t x = 200;` compiles with a warning (sometimes silently) and gives you `-56`. In Go, `var x int8 = 200` is a hard compile error: "constant 200 overflows int8." The untyped-literal machinery is what makes this check possible — the compiler still has the exact value `200` available at the moment the type is forced, so it can verify representability before truncating.

### Diagnostic questions

1. **Q:** Why does `var x float64 = 3` work but `var x float64 = i` (where `i` is an `int`) not?
   *Wrong-answer trap:* "Because 3 is small." It's because `3` is **untyped** — it has no type yet, so it freely adopts `float64`. `i` is already typed `int`, and Go forbids implicit int→float64 promotion. The same value, different syntactic categories, different rules.

2. **Q:** What's the default type of `'a'` if no context forces one?
   *Wrong-answer trap:* "`byte`." It's **`rune`**, which is an alias for `int32` — Go treats characters as Unicode code points by default, not as 8-bit bytes. This bites C transplants who expect `'a'` to be `char` (8-bit).

3. **Q:** Can you declare `const c = 1 / 0` in Go?
   *Wrong-answer trap:* "Yes, untyped constants don't run." No — division by zero of *integer* constants is a compile-time error in Go (spec: "constant expressions ... a division or modulo operation by zero is illegal"). Untyped doesn't mean unevaluated.

4. **Q:** If untyped literals exist at compile time only, what does `fmt.Printf("%T\n", 100)` print?
   *Wrong-answer trap:* "untyped int." It prints `int` — the moment the literal crosses into a function call with an `interface{}` parameter (which is how `Printf`'s variadic args work), it must have a runtime type, so the **default type** rule fires and `100` becomes `int`. There is no runtime "untyped" state — that's the whole point.

5. **Q:** Why doesn't Go use a suffix system like C (`100L`, `100ULL`) or Rust (`100u64`)?
   *Wrong-answer trap:* "Aesthetic preference." The Go design rationale (Pike's blog) is that suffixes leak the type system into every literal site; with context-driven typing, the *call site's signature* is the single source of truth for the type, and the literal stays decluttered. The cost is that you can't read a literal in isolation and know its type — you have to look at the surrounding expression.

### See also

- [lgo-notes.md](Notes/lgo-notes.md) 2026-05-12 — *gopls and the Language Server Protocol* — `gopls` is the tool that surfaces the type Go inferred for any literal on hover; understanding untyped literals makes its hover output stop being mysterious.
- Rob Pike, "Constants" (golang.org/blog, 2014) — the canonical explainer; this entry is a compressed reading of it.
- CUDA / C++17 `constexpr` — the C++ analogue (compile-time arbitrary-precision math) but bolted on, not load-bearing.
- DDIA / DBI on serialization formats — every numeric serialization is essentially the inverse of Go's literal-typing question: at what point does an abstract number commit to a concrete bit-width?

---

## [2026-05-12] gopls and the Language Server Protocol · pp.21–30 · Ch.1 § Choose Your Tools

### TL;DR
Before LSP, every editor had to write its own integration for every language — an `N × M` problem of editors × languages. The **Language Server Protocol (Microsoft, 2016)** broke the matrix by defining one JSON-RPC interface: each language provides one "server" (e.g. `gopls`), each editor implements one "client." LGO catches the Go ecosystem mid-transition: in 2019 VS Code's Go support still depended on a dozen separately-maintained tools; by 2026 **gopls is the unambiguous default** and the older tool zoo is mostly retired.

### History — "why does this exist?"
The `N × M` editor-integration problem was visible by 2010 — Eclipse had JDT for Java, Vim had a half-dozen Python plugins, every IDE shipped its own indexer. **Microsoft built the Language Server Protocol for VS Code (open-sourced 2016)** and crucially *did not* keep it proprietary: LSP is a JSON-RPC schema published under MIT license, and within four years every serious editor (Vim/Neovim via coc.nvim or built-in LSP, Emacs via lsp-mode, Sublime, JetBrains as a fallback) spoke it. For Go specifically, the pre-LSP stack was painful: `gocode` for completion, `godef` for jump-to-definition, `gorename` for refactors, `guru` for analysis, `goimports` for imports — each maintained by a different volunteer, each broken differently when Go modules landed in 2018. **gopls** (pronounced "go please") is the Go team's official replacement: one binary, maintained by the same people who maintain the compiler, that speaks LSP. The book's 2019 caveat — "as of this writing, gopls is still under development, which is why it is not the default setting" — is exactly the moment of the transition, frozen.

### Intuition — "this is like…"
LSP is **the HTTP of editor tooling**. Before HTTP, every client/server pair invented its own wire format; after HTTP, you write one client (browser) and one server (web app) and the matrix collapses to addition instead of multiplication. LSP does for editor-language pairings what HTTP did for client-server: turn an `N × M` problem into `N + M`.

### Mechanics

**The pre-LSP world (matrix problem):**

```
              ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
              │ VS Code  │  │  Vim     │  │  Emacs   │  │ JetBrains│
              └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘
                   │             │             │             │
   Go support:  vscode-go     vim-go        go-mode       GoLand
                (custom)       (custom)     (custom)      (custom)
   Rust:       rust-analyzer  rust.vim     rust-mode     IntelliJ Rust
                (custom)       (custom)     (custom)      (custom)
   Python:     python-tools  YouCompleteMe elpy         PyCharm
                (custom)       (custom)     (custom)      (custom)

   N editors × M languages = N×M integrations to maintain.
   In practice: most cells are broken or half-working.
```

**The post-LSP world (addition):**

```
   Editors (clients)                Languages (servers)
   ┌──────────┐                     ┌──────────────────┐
   │ VS Code  │ ─────┐         ┌───►│  gopls (Go)      │
   ├──────────┤      │         │    ├──────────────────┤
   │  Vim     │ ─────┤   LSP   ├───►│  rust-analyzer   │
   ├──────────┤      ├─JSON-RPC┤    ├──────────────────┤
   │  Emacs   │ ─────┤  over   ├───►│  pyright (Py)    │
   ├──────────┤      │  stdio  │    ├──────────────────┤
   │ JetBrains│ ─────┘         └───►│  typescript-     │
   └──────────┘                     │  language-server │
                                    └──────────────────┘
   N + M integrations. Each cell either works or doesn't —
   no silent half-broken combinations.
```

**The LSP wire protocol is plain JSON-RPC over stdin/stdout.** A request to find the definition of a symbol looks like this:

```json
// Editor → Server
{
  "jsonrpc": "2.0",
  "id": 42,
  "method": "textDocument/definition",
  "params": {
    "textDocument": { "uri": "file:///home/user/main.go" },
    "position": { "line": 17, "character": 8 }
  }
}

// Server → Editor
{
  "jsonrpc": "2.0",
  "id": 42,
  "result": [{
    "uri": "file:///home/user/util.go",
    "range": {
      "start": { "line": 4, "character": 5 },
      "end":   { "line": 4, "character": 12 }
    }
  }]
}
```

That's the entire trick. Every LSP method (`textDocument/completion`, `textDocument/hover`, `textDocument/rename`, `textDocument/references`, `workspace/symbol`…) is a similarly shaped request/response. The editor knows nothing about Go; gopls knows nothing about VS Code.

**What gopls replaces** (LGO's pre-LSP tool list):

| Old tool | Function | Replaced by gopls method |
|---|---|---|
| `gocode` | Autocompletion | `textDocument/completion` |
| `godef` / `guru` | Jump to definition | `textDocument/definition` |
| `gorename` | Symbol rename | `textDocument/rename` |
| `goimports` | Auto-add/remove imports | `textDocument/formatting` (+ codeAction) |
| `golint` (subset) | Style hints | `textDocument/diagnostic` |
| `gocode-mod`, etc. | Module-aware variants of above | (gopls is module-aware natively) |

The Go team's diagnosis was correct: maintaining the legacy zoo across the module transition was strictly impossible with their volunteer base. Consolidation into one official server was the only path.

**The bidirectional gain.** LSP didn't just save the editors; it saved the *language teams*. Before LSP, the Go team had no real way to ship analysis features to users — they could write `go vet` checks but couldn't surface them in editors without lobbying each editor's volunteers. With gopls, a new diagnostic ships from `go.googlesource.com` and is in every LSP-aware editor within a release cycle.

### If you were designing tooling for a new language…

In 2026, you don't write an editor plugin. You write **one LSP server**, host it on GitHub, and document the install path. The plugin ecosystem comes for free because every modern editor either ships an LSP client or has one as a community plugin. Skipping this step (writing per-editor plugins) is the single most common time-sink for new-language authors — you can spot a language whose tooling story is going to suffer by whether its first README has "VS Code extension" instructions or "language server" instructions.

### Cross-language view

```rust
// rust-analyzer — Rust's LSP server, the reference implementation
//  · written in Rust, ships standalone
//  · `rustup component add rust-analyzer` in 2026
//  · the bar to clear; gopls is comparable in completeness
```

```python
# Pyright (Microsoft) — Python's LSP server
#  · types-first; treats Python as if it were gradually typed
#  · plus pylsp (community fork of older pyls) for non-typed projects
#  · ecosystem hasn't fully consolidated — Python's LSP story is "two servers"
```

```typescript
// typescript-language-server — wraps tsserver (the actual TS compiler service)
//  · the TS compiler API predates LSP; the LSP server is a thin adapter
//  · proof that LSP can wrap an existing rich API without a rewrite
```

**The stdlib note:** in Go, gopls is now part of the official toolchain — it lives at `golang.org/x/tools/gopls` and is versioned alongside Go releases. `go install golang.org/x/tools/gopls@latest` is the canonical install. Run `gopls --help` and you can drive it from the shell to debug editor issues — a habit worth forming.

### Where this shows up in real systems

- **AI coding assistants (Copilot, Cursor, Claude Code) all consume LSP.** When Claude Code "jumps to definition" or "finds references," it's calling LSP under the hood, not running a custom indexer. Every improvement to gopls is automatically an improvement to AI-assisted Go editing.
- **The "editor of the year" debate is mostly settled.** Pre-LSP, choosing an editor meant signing up for whoever maintained your language's plugin in that editor. Post-LSP, the language-tooling quality is roughly *constant across editors*, so the choice collapses to UX preference. This is why Vim users in 2026 get the same completion quality as VS Code users — same gopls.
- **Polyglot monorepos.** A repo with Go + TypeScript + Python services used to need a different editor profile per language. Today, one editor with three LSP servers running gives the same experience across all three. Companies like Stripe and Shopify standardized on this model around 2021–2022.

### Diagnostic questions

1. **Q:** Why did LSP succeed when prior attempts at editor-tooling standards (e.g. CTAGS, GNU Global) didn't?
   *Wrong-answer trap:* "Because Microsoft backed it." More substantively: prior attempts standardized *the index* (a static artifact). LSP standardized *the protocol* (a live conversation), which lets each language's server use whatever indexing strategy fits — type-aware for Rust, AST-walking for Python, full compiler for Go. The flexibility was the win.

2. **Q:** Why does gopls run as a separate process per editor session rather than as a library linked into the editor?
   *Wrong-answer trap:* "Because of language barriers (Go vs JavaScript)." That's true but secondary. The real reason is *fault isolation* — gopls crashing should not crash your editor, and editor crashes should not lose gopls's in-memory index. Process boundaries are the simplest crash-isolation primitive available.

3. **Q:** The book mentions enabling "Use Language Server" as a setting in VS Code. Why was it opt-in at first?
   *Wrong-answer trap:* "Microsoft was being cautious." gopls was *demonstrably less complete* than the legacy tool zoo in 2019 — it lacked features users depended on. Opt-in let early adopters provide bug reports without disrupting the median user; opt-out (then on-by-default) followed as parity was reached. This is the standard pattern for replacing a *working* system with a *better-architected* one.

4. **Q:** If you wanted to add support for a niche language to your editor in 2026, what's the minimum you need?
   *Wrong-answer trap:* "Write a plugin in the editor's extension API." Write *or find* an LSP server. The plugin is generic LSP-client glue that most editors already ship; the value is in the server. Skipping that step is how language tooling falls behind.

### See also

- LGO Ch.1 § "go vet & golangci-lint" (pp.18–21) — the static-analysis tools that *predate* and partially *feed* gopls. Worth reading immediately before this entry.
- DDIA Ch.4 "Encoding and Evolution" — LSP is JSON-RPC, which means its schema-evolution problems are exactly DDIA's chapter content (backwards compat, optional fields, capability negotiation via `initialize`).
- DSG Ch.1–2 (gRPC vs JSON) — interesting counterpoint: LSP chose JSON-RPC for tooling-friendliness (you can curl it); DSG argues for protobuf+gRPC for service-to-service work. The trade-off (debuggability vs. throughput) is a recurring axis.
- COD 2026-05-12 — the ARMv7→ARMv8 entry is a *different* version of the same pattern: an `N × M` matrix problem (every chip vendor × every legacy quirk) collapsed by a once-per-generation cleanup.

---

## [2026-05-11] The Semicolon Insertion Rule — Why Go's Brace Style Is Mandatory · pp.11–20 · Ch.1 · Setting Up Your Go Environment

### TL;DR
Go requires semicolons at the end of every statement — but the lexer inserts them automatically, following one short rule: if a line ends in an identifier, literal, `}`, `)`, `++`, `--`, or one of a few keywords, a semicolon is appended before the newline. This single rule is the entire reason Go's brace placement is non-negotiable (`func main() {` must be on one line) and the reason `go fmt` ships with the compiler. It's also a clear example of Go's design philosophy: solve formatting wars by making "wrong format" a syntax error, then build tooling that makes the right format effortless.

### History — "why does this exist?"
The semicolon-insertion idea is older than Go — **JavaScript** has had Automatic Semicolon Insertion (ASI) since ECMAScript 1 (1997), and ASI is infamous for being too aggressive and biting developers (the classic `return\n  {value}` bug). Go's designers (Robert Griesemer, Rob Pike, Ken Thompson at Google, 2007–2009) deliberately designed a simpler, more predictable rule than JS's: only seven token classes trigger insertion, and the rule depends only on the last token of the line — no semantic lookahead. The motivation, per **Russ Cox's public statement** the book quotes, was not avoiding format wars but **making code more amenable to tools**: a uniform layout means `gofmt`, `goimports`, `gopls`, and AST-based refactorers can be smaller, faster, and more reliable than their C++/Java equivalents.

### Intuition — "this is like…"
Imagine a court stenographer who's been told: "If the speaker pauses after finishing a sentence-shaped word (a noun, a closing parenthesis, a `}`), insert a period. If they pause anywhere else, assume they're still talking." That rule lets the speaker drop the explicit periods — but it means they can't pause for breath in the middle of a sentence, because the stenographer will end it for them. That's exactly what happens when you put `{` on a new line in Go: the lexer "ends the sentence" at the `)` of `func main()`, and your `{` becomes a stray block.

### Mechanics

**The rule, verbatim from the Go spec:**

> A semicolon is automatically inserted after a line's final token if that token is:
> - an identifier (including `int`, `float64`, `nil`, etc.)
> - a basic literal (number, string, rune)
> - one of: `break`, `continue`, `fallthrough`, `return`, `++`, `--`, `)`, `}`

**Why C-style brace placement breaks Go.** If you write:

```go
func main()       // last token is ')' → semicolon inserted!
{
    fmt.Println("Hello, world!")
}
```

The lexer produces:

```go
func main();      // function declaration with no body
{                 // stray block
    fmt.Println("Hello, world!");
};
```

This is two top-level statements (a function declaration with no body, then a block) — a syntax error. The compiler complains `missing function body`, which is honest: there really is no body attached to `func main`.

**The correct form puts the `{` on the same line:**

```go
func main() {     // last token is '{' → NO semicolon, statement continues
    fmt.Println("Hello, world!")
}                 // last token is '}' → semicolon inserted at end of block
```

**The full toolchain that depends on this discipline:**

```
 source.go
    │
    ▼
 ┌─────────┐    ┌──────────────────┐    ┌────────────┐
 │  go fmt │ ─→ │  gofmt-formatted │ ─→ │ go build / │
 │         │    │      source      │    │  go run    │
 └─────────┘    └──────────────────┘    └────────────┘
       ▲                  │
       │                  ▼
       │           ┌──────────────┐
       └─────────  │  goimports   │ ← adds + sorts imports
                   │  golint      │ ← idiomatic naming
                   │  go vet      │ ← deeper bug-detection
                   └──────────────┘
```

**`go run` vs `go build` — a useful distinction the chapter highlights:**

| Command | Output | Best for |
|---------|--------|----------|
| `go run hello.go` | Builds to temp dir, runs it, deletes binary | Scripting, REPL-like iteration |
| `go build hello.go` | Persistent binary in cwd (`hello` / `hello.exe`) | Distributing artifacts; the normal case |
| `go install pkg@vX.Y` | Builds + places binary in `$GOPATH/bin` | Installing tools (`hey`, `golint`, etc.) without a central registry |

Note that `go install` taking a *source URL plus version* is Go's idiosyncratic answer to "where's our npm/PyPI?" — there isn't one. The Go ecosystem treats the source repository as the registry, with **proxy.golang.org** as a caching layer (introduced in Go 1.13).

### If you were the language designer…

You want to eliminate brace-style wars (K&R vs Allman vs GNU) and tabs-vs-spaces flame threads forever. You have two design options:

**Option A:** Add a style guide and a formatter tool, but make formatting optional. (Java, Python, C# — most languages.) Outcome: every team writes its own style guide; the wars move from the language to the team.

**Option B:** Make the grammar itself depend on whitespace/layout, so "wrong" formatting becomes a syntax error. Then ship the formatter in the compiler. (Go, Python's significant indentation, F#'s offside rule.) Outcome: the wars become unwinnable, because the language has already chosen.

Go picked B — with a twist: the layout rule (the semicolon insertion rule) is *simple enough to fit on a postcard*, unlike Python's indentation rules which have ~20 pages of corner cases (continuation lines, hanging indents, tab/space mixing). The trade-off: Go has fewer layout choices than Python, but those choices are unambiguous.

### Cross-language view

Side-by-side, the same "hello world" function across four languages reveals the design space:

```c
// C — brace placement is purely aesthetic; preprocessor handles continuation
int main(void)
{
    printf("Hello, world!\n");
    return 0;
}
```

```rust
// Rust — semicolons are explicit, but braces can go anywhere
fn main()
{
    println!("Hello, world!");
}
```

```python
# Python — significant indentation, no braces, no semicolons
def main():
    print("Hello, world!")
```

```go
// Go — '{' MUST be on the same line as func declaration
func main() {
    fmt.Println("Hello, world!")
}
```

**What the standard library / tooling actually does:**
- Rust's `rustfmt` is opt-in and configurable (`rustfmt.toml`); the language has no opinion.
- Python's `black` ("the uncompromising formatter") emerged in 2018 to retroactively impose Go-style rigidity on Python — it's wildly popular precisely because the lack of a built-in formatter created the void it filled.
- Go's `gofmt` is non-configurable and bundled. Russ Cox's stated goal — "tooling first" — is visible in `gopls` (the language server), which can do refactors no Rust analyzer can match because every Go file looks the same.

### Where this shows up in real systems

- **Kubernetes, Docker, etcd, Terraform** — every major Go project's CI checks `gofmt -d` and rejects PRs that aren't formatted. The cost of dispute over formatting in these multi-thousand-contributor projects is near zero, because there's nothing to dispute. Compare to LLVM's clang-format wars or the Linux kernel's tab-vs-space subsystem feuds.
- **The `gopls` language server.** Modern Go IDE features (rename, extract function, find-usages) work across millions of LOC at near-instant speeds because the AST is canonical — every reader sees the same tree. Rust-analyzer and TypeScript's tsserver are excellent but require more compute because they must normalize layout first.
- **The "you can't escape gofmt" effect on code review.** Go shops report PR reviews focus on logic, not style — a measurable win in throughput and reviewer fatigue.

### Diagnostic questions

1. **Q:** Why does the brace `{` need to be on the same line as `func main()` but the `}` can be on its own line?
   *Wrong-answer trap:* "It's an arbitrary style choice." It's a direct consequence of the semicolon-insertion rule: `)` triggers insertion (so a newline after `)` ends the statement), but `{` does not (so a newline after `{` is just whitespace inside a block).

2. **Q:** Why is `go fmt` non-configurable while `rustfmt` and `prettier` have hundreds of options?
   *Wrong-answer trap:* "Go's authors are opinionated." More fundamentally: Go's tooling chain (gopls, goimports, gorename) depends on a single canonical form. Making `gofmt` configurable would mean every tool downstream has to handle N variants — exactly the situation Rust and TypeScript are in.

3. **Q:** What's the failure mode of JavaScript's ASI that Go's rule avoids?
   *Wrong-answer trap:* "JavaScript inserts too aggressively." The real difference: JS's ASI depends on parser state (whether what follows is a valid continuation), so the rule is non-local. Go's rule depends only on the last token of the line, so it's lexer-local and trivially predictable. The famous JS bug — `return\n {x:1}` returning `undefined` — cannot have a Go analog.

4. **Q:** If `go run` deletes its binary, where does it actually live during the run?
   *Wrong-answer trap:* "In RAM only." In a temp directory (usually `$TMPDIR/go-build*`). The OS still needs an executable file with an inode to launch; Go just unlinks it afterward. You can trace `go run` to see the file appear and disappear.

5. **Q:** Why does Go install third-party tools via source repository URLs (`go install github.com/.../hey@latest`) instead of a central registry?
   *Wrong-answer trap:* "Because Google is cheap." It's a deliberate decentralization choice — registries are single points of failure (cf. the 2016 `left-pad` npm incident) and bottlenecks for namespace disputes. Go's tradeoff: more verbose import paths in exchange for no central authority. The `proxy.golang.org` cache layer recovers most of the registry's UX without the political surface.

### See also

- [dsg-notes.md](Notes/dsg-notes.md) — Distributed Services with Go assumes this toolchain is muscle memory; expect heavy `go install` of third-party CLIs.
- [cod-notes.md](Notes/cod-notes.md) — Go's compile-to-single-binary model exists because Ch.1's abstraction stack statically links everything; there's no equivalent of dynamic linking. This is why `go run` can throw the binary away — the binary is the entire dependency closure.
- [tpp-notes.md](Notes/tpp-notes.md) — The Pragmatic Programmer's "use tools that fit your hand" chapter; `gofmt`/`goimports`/`gopls` are the canonical fit-your-hand example.

---
