# WELC Notes

_Entries follow the template at `Notes/TEMPLATE.md`. Append-only. **Newest entry at top**, immediately after this header._

---

## [2026-05-26] Refactoring tools & xUnit isolation · pp.68–81 · Ch.5 *Tools* (Automated Refactoring → Mock Objects → xUnit → FIT/Fitnesse) → Ch.6 *I Don't Have Much Time* (intro)

- Why "behaviour-preserving" refactors aren't always behaviour-preserving
- xUnit's one-object-per-test design and why `setUp` is a method, not a constructor
- The cost-of-tests vs. cost-of-debugging trade — the day-to-day decision

### History — "why does this exist?"
Refactoring as a named discipline arrives with **Bill Opdyke's 1992 PhD thesis** at UIUC, which formalised the idea that you could change a program's structure without changing its observable behaviour. **Brant & Roberts' Smalltalk Refactoring Browser (mid-1990s)** turned the thesis into a working tool that mechanically rewrote code while *proving* equivalence in a limited algebra. Eclipse JDT (2001) brought it to mainstream Java; ReSharper (2004) to C#; IntelliJ matured it across languages. Meanwhile, **Kent Beck wrote SUnit (the first xUnit) in Smalltalk in 1994**; Beck and Erich Gamma ported it to Java as JUnit in 1997. The book's whole pitch — *you need tests before you refactor* — exists because the first generation of refactoring tools couldn't (and many still can't) prove behaviour preservation across all edge cases.

### Intuition — "this is like…"
A modern refactoring tool is a **`git rebase --autosquash` for code structure**. It promises to rewrite history (the AST) preserving the externally-visible result. But just as a rebase can silently drop a commit if you mistype a `fixup!` line, a refactoring tool can silently change behaviour when an operation it implements has a gap in its semantic check. The fix is the same in both worlds: have a fast verification loop (tests, or `git diff prev_head..HEAD`) so you *see* the regression the moment it happens, not three commits later.

### Mechanics

**The "behaviour-preserving" loophole.** Fowler's definition is famous: *"A change made to the internal structure of software to make it easier to understand and cheaper to modify without changing its existing behavior."* The "without changing existing behavior" clause is the whole load-bearing wall, and not every tool actually checks it.

**Feathers' example — inline-variable refactor that breaks the program:**

```java
// Before — refactoring tool sees: 'v' is read once, can be inlined
public void doSomething() {
    int v = getValue();        // getValue() increments alpha as a side effect
    int total = 0;
    for (int n = 0; n < 10; n++) {
        total += v;            // v evaluated once before loop
    }
}

// After — inline-variable rewrites use site
public void doSomething() {
    int total = 0;
    for (int n = 0; n < 10; n++) {
        total += getValue();   // now called 10 times — alpha +10, not +1
    }
}
```

The tool's check was syntactic — "is `v` only read here?" — not semantic — "does `getValue()` have side effects?" The refactor moved the call from once-before-loop to ten-times-inside-loop, multiplying its side effect by 10. **A "safe" refactor changed observable behaviour.**

**The general failure mode:**

```
┌──────────────────────────────────────────────────────────────┐
│  Refactor tool's safety = AST equivalence under its model    │
│  Real safety = observable-behaviour equivalence              │
│  Gap = side effects the AST model didn't account for         │
│                                                              │
│  Side effects that bite:                                     │
│    • method calls with hidden state mutation                 │
│    • finalizers, weak refs                                   │
│    • thread-local state, request-scoped DI                   │
│    • lazy evaluation order (Kotlin `by lazy`, Scala vals)    │
│    • exception order (rename-symbol changing throw-site)     │
└──────────────────────────────────────────────────────────────┘
```

**Feathers' sanity checks for a new tool.** When you adopt a new refactoring engine, *probe its safety* before trusting it:

- *Extract-method into an existing name in the same class.* Does it error or silently overwrite?
- *Extract-method matching a base-class method name.* Does it detect the accidental override?
- *Inline-variable across a method call with side effects.* Does it warn?

If any of these fail, you have a *syntactic* refactor tool, not a *semantic* one. Use it only with characterisation tests already in place.

**xUnit — one-object-per-test, the architectural choice that defines the framework.**

When JUnit runs a `FormulaTest` containing `testEmpty()` and `testDigit()`, it does **not** create one `FormulaTest` and call both methods on it. It uses reflection to:

```
1. Find all `public void testXxx()` methods on FormulaTest      → [testEmpty, testDigit]
2. For each method, instantiate a separate FormulaTest object   → obj₁, obj₂
3. Configure obj₁ to run only testEmpty; obj₂ to run only testDigit
4. Run each object in sequence (or in parallel with isolation)
```

Effect: **each test gets a fresh instance with fresh fields**. There is *no path* by which `testEmpty` can leak state into `testDigit` through `this.something`, because they don't share a `this`.

```
┌─ Conventional class usage ──────┐  ┌─ JUnit test class usage ────────┐
│  one instance, many methods     │  │  one method, one instance       │
│  state accumulates across calls │  │  state is fresh per call        │
│  → ordering matters             │  │  → ordering is irrelevant       │
└─────────────────────────────────┘  └─────────────────────────────────┘
```

**Why `setUp()` is a method, not a constructor.** If you constructed all `FormulaTest` instances eagerly at class-load (one per test method), and each constructor did expensive work (open DB connection, allocate fixture), you'd pay 100× resources for a 100-test class even though only one test runs at a time. JUnit's trick: instantiate cheaply (default ctor), then call `setUp()` *just before* the test method, and `tearDown()` *just after*. Resources exist only during the test's execution window, not for the entire test run.

**Worked timeline:**

```
class EmployeeTest { setUp(): create employee; testNormalPay(); testOvertime(); }

run testNormalPay:
  ctor()           ← cheap
  setUp()          ← create employee, allocate fixture
  testNormalPay()  ← run assertion
  tearDown()       ← release fixture
  obj eligible for GC

run testOvertime:
  ctor()           ← cheap (new instance!)
  setUp()          ← create employee, allocate fixture (fresh)
  testOvertime()
  tearDown()
```

The fixture is rebuilt twice. That feels wasteful but is **the entire point**: tests cannot have order-dependent failures because they share no mutable state.

**Mock objects' role.** Refactoring legacy code typically requires breaking *dependencies* (DB, network, file I/O). Mock objects substitute behaviour-fakes at those seams. Frameworks: Mockito (Java), Moq (C#), `unittest.mock` (Python), gomock (Go). Common antipattern Feathers calls out implicitly: mocks that don't verify *interaction* (call order, argument values) provide false confidence — they assert on return values for calls that may never have happened.

**Cost-of-tests vs cost-of-debugging — the day-to-day decision (Ch.6 intro).** Feathers' framing:

```
Investment now:   2 hours to write characterisation tests
                  + 15 min to change code

Naive savings:    skip tests → 15 min to change → 1h45m saved

Real expected cost of skipping:
   P(bug introduced) × E[time to find & fix without tests]
 = (0.30) × (4 hours debugging)
 = 1.2 hours expected debugging cost
   + 0.5 hours expected re-read time on next change (no tests = re-read)
 = 1.7 hours expected, with high variance

Real savings of skipping ≈ 1h45m − 1h42m = ~3 min,
with a fat tail when you happen to roll the bug.
```

The book's point: the expected value is barely in favour of "skip tests"; the *variance* makes skipping a losing bet over a year.

### If you were a refactoring-tool author…

You're implementing Inline-Variable for Kotlin. The user's variable is read once. Do you replace the read with the expression at the declaration site?

**No — not without checking side effects.** You need to know whether the RHS expression has observable side effects: method calls to user code, property access on a class with a custom getter, anything involving I/O, anything involving lazy/by-lazy. The Smalltalk Refactoring Browser handled this by maintaining a *side-effect-free* annotation derived from a method-purity analysis; tools without that analysis must conservatively refuse the refactor, ask the user, or display a warning. Real tools split the difference: IntelliJ shows a "may change semantics" tooltip; Eclipse JDT silently refactors. This is exactly Feathers' "find out what the tool says about safety" advice in action.

### Cross-language view

| Language | xUnit harness | Test-per-instance? | setUp/tearDown name |
|---|---|---|---|
| Java | JUnit 5 (`@Test`) | yes (one per `@Test` method) | `@BeforeEach`/`@AfterEach` |
| Python | `unittest.TestCase` | yes (one per `test_*`) | `setUp`/`tearDown` |
| Go | `testing.T` (`TestXxx(t *testing.T)`) | **no** — fns, not objects | `TestMain` / sub-test `t.Cleanup` |
| Rust | `#[test]` functions | n/a — fns, not objects | `#[ctor]` crates or per-test setup fn |
| C# | xUnit.net (`[Fact]`) | yes (one per `[Fact]`) | constructor / `IDisposable.Dispose` |

The Go and Rust models break with xUnit's instance-per-test orthodoxy because their idioms (functions, not test classes) make shared-state-by-accident harder anyway. C# xUnit.net cleverly uses the *constructor* as `setUp` and `IDisposable.Dispose` as `tearDown` — making the instance-per-test architecture visible in language-native terms.

### Where this shows up in real systems

- **IntelliJ's "may change semantics" warnings** are direct descendants of Opdyke's safety-check work. The IDE colors the inline-variable preview red when the RHS has a method call; the engineering decision is to make the *user* the side-effect-check oracle.
- **Mockito's `verify(mock).method(args)`** is the explicit answer to the broken-mock antipattern — without `verify`, a mock returning canned values gives false confidence; with it, you assert call shape.
- **`go test -race` and Postgres pg_regress** are non-xUnit harnesses that share xUnit's isolation goal differently: `-race` runs each test under the data-race detector; pg_regress spawns a fresh Postgres per suite. Same principle (state isolation), different mechanics (process-level, not object-level).
- **JaCoCo / Coverage.py** measure how much of production code your characterisation tests touched before a refactor — closing the feedback loop Feathers describes: you can't trust tool-driven refactor safety unless you know which lines you actually exercised.

### Diagnostic questions

1. **Q:** Why doesn't every "safe" refactor mean behaviour-preserving?
   *Wrong-answer trap:* "Because some tools have bugs." Bugs aside, the real issue is the **gap between syntactic checks and semantic equivalence**. Side effects, exception order, lazy evaluation all live in the gap.

2. **Q:** In JUnit, why does the framework create a separate object per `@Test` method instead of reusing one?
   *Wrong-answer trap:* "For thread safety." Wrong direction — JUnit runs tests sequentially by default. The real reason: **prevent state leakage across tests via fields**.

3. **Q:** Why is `setUp()` a method called per-test instead of work done in the constructor?
   *Wrong-answer trap:* "Constructors can't take parameters." They can — and JUnit's per-test instances default-construct anyway. The real reason: **defer resource allocation to the moment of need**, so 100-test classes don't pay 100× allocation cost up-front, and so allocation errors are reported as a test failure (not a class-load failure).

4. **Q:** Mockito returns canned values from a mock. Without `verify(mock).foo()`, what could go wrong?
   *Wrong-answer trap:* "Nothing — the value comes back fine." The trap is that the mock could be **called 0 times** (because the production code took a different branch) and your assertion on the *return* still passes; you've tested nothing.

5. **Q:** What concrete probe should you run on a refactoring tool you've never used?
   *Wrong-answer trap:* "Look at the docs." Docs lie or omit. Feathers' answer: **extract-method into an existing name, on a base class.** If the tool doesn't detect the accidental override, it's syntactic-only — treat its refactors as edits, not behaviour-preserving moves.

---

## [2026-05-25] Seams & enabling points · pp.54–67 · Ch.4 *The Seam Model* — Seams → Seam Types (Preprocessing / Link / Object)

- Seam = "alter behavior without editing in place"; every seam needs an **enabling point**
- Three seam types map to compilation stages: preprocessing → link → object
- Choosing the right seam: object first in OO, link/preprocessing as escape hatches

### History — "why does this exist?"
**Michael Feathers coined "seam"** specifically for this book (*Working Effectively with Legacy Code*, 2004). The metaphor is sewing — a seam is the joint between two pieces of cloth, the place you can pull apart and re-stitch without re-weaving the fabric. The need was a 1990s–2000s reality: enormous C++ codebases (telecom switches, trading platforms, embedded firmware) had no dependency injection framework, no mocking library, and no language-level interfaces beyond inheritance. **LD_PRELOAD** (SunOS 4.0, 1988) had given Unix programmers link-time interposition for a decade, and **function pointers in C** (1970s onward) were the working-class object seam — but no one had named the pattern. By 2004 Java had Spring (2003) and JUnit (1997), but C++ shops were still wiring tests by hand. Feathers's contribution was naming the move so engineers could *plan* for it instead of stumbling into it. Mockito (2007), jMock (2003), and pytest's `monkeypatch` (2012) later automated the **enabling point** — the test setup phase became the canonical place to flip the switch.

### Intuition — "this is like…"
A seam is a **light switch** you wired into the system specifically so you could turn it on or off later. The **enabling point** is where the switch is physically mounted:

| Enabling point location | Seam type | Real-world analogue |
|---|---|---|
| In the test code itself (constructor arg, setter, override) | **Object** | A toggle in the room — easy to see, easy to flip |
| In the build script / linker flags / `LD_PRELOAD` env | **Link** | At the circuit breaker — flip the whole apartment |
| In a preprocessor `#define` or `--config` flag | **Preprocessing** | At the power meter — flip everything in the building |

The further the enabling point lives from the code being tested, **the harder it is to notice during code review**, and the higher the chance someone runs the wrong configuration in production. Object seams are the safest because the test reader can *see* the substitution in the test setup. Link seams are infamous for "but it works locally" — the locally-installed library masked a broken production call.

In modern terms: **object seams are Spring's `@Autowired` / Jest's `jest.mock(ClassName)`**, **link seams are LD_PRELOAD / Docker volume mounts for shared libraries**, **preprocessing seams are `#ifdef DEBUG` blocks and Webpack's `DefinePlugin`**.

### Mechanics

#### The definition, boxed (Feathers's, verbatim-in-spirit)

> **Seam**: a place where you can alter behavior in your program without editing in that place.
>
> **Enabling point**: every seam has one — the location at which you decide which behavior the seam will exhibit.

The two halves matter equally. A point in code that *could* be swapped but has no enabling point isn't a seam yet — it's a dependency waiting to become one.

#### Three seam types, one row each

| Type | Where the seam is | Where the enabling point is | When it's the right choice |
|---|---|---|---|
| **Preprocessing** | Anywhere `#include` / macros run *before* the compiler sees the code | A `#define` flag (e.g. `TESTING`) in the build | Last-resort for C/C++ when no object seam exists and global functions are called from many places |
| **Link** | At any external function/library call resolved at link time | The linker's library search path / `LD_PRELOAD` / build script | Pervasive third-party calls (graphics libs, vendor SDKs) with no virtual hook |
| **Object** | Any polymorphic dispatch — virtual call, interface method, function passed as arg | The point where the object is *constructed* (or the arg supplied) | Default in any OO language; the only seam that's visible in the test itself |

#### One example, three seams — the `PostReceiveError` problem

The book's running example: a method calls `PostReceiveError(SOCKETCALLBACK, SSL_FAILURE)`, a global C function that calls a real subsystem you can't tolerate in tests. **Three valid seams**:

```cpp
// ── Option 1: Object seam ──
// Promote PostReceiveError to a virtual method, override in test subclass
class CAsyncSslRec {
    virtual void PostReceiveError(UINT type, UINT errorcode);  // ← seam HERE
};
class TestingAsyncSslRec : public CAsyncSslRec {
    void PostReceiveError(UINT, UINT) override { /* no-op */ }
};
// Enabling point: the line that constructs TestingAsyncSslRec instead of CAsyncSslRec.

// ── Option 2: Link seam ──
// Compile a separate test-only object file that defines PostReceiveError as a no-op
// libssl_production.a  → real PostReceiveError
// libssl_test.a        → stub PostReceiveError { /* no-op */ }
// Enabling point: the Makefile target chooses which .a to link.

// ── Option 3: Preprocessing seam ──
#ifdef TESTING
  #define PostReceiveError(t, e) /* nothing */
#endif
// Enabling point: -DTESTING on the test build's compiler invocation.
```

#### Why object seams are preferred (when available)

```
                Visibility of substitution     Risk of prod mistake
Object seam:    ████████████  HIGH             ██  LOW
Link seam:      ████          LOW              ██████  MEDIUM
Preproc seam:   ██            VERY LOW         ████████  HIGH
```

**The visibility argument**: an object seam appears in the test's setup. A reader of the test can see `new TestingAsyncSslRec()` and know what's substituted. A link seam's enabling point lives in a Makefile, three directories away. A preprocessing seam's enabling point is a `-D` flag in a CI config buried in `.github/workflows/`.

**The risk argument**: ship the wrong build configuration once, and link/preprocessing seams silently put test stubs into production. Object seams can't make that mistake — they're chosen at runtime per instance, and prod code constructs real objects.

#### Modern-toolchain map — where each seam type lives today

| Era pattern (WELC, 2004) | Modern embodiment (2026) |
|---|---|
| Hand-written subclass + virtual override | **Mockito.mock(MyService.class)**, **unittest.mock.MagicMock**, **gomock**, **sinon.stub** |
| Linker substitution of test library | **LD_PRELOAD** still, **Docker volume mount over /usr/lib**, **Bazel's `cc_library` test deps**, **Go's `-tags=test`** |
| `#ifdef TESTING` macros | **Webpack/Vite `DefinePlugin`**, **Rust's `#[cfg(test)]`**, **Go's `_test.go` files** (compile-time inclusion) |
| C function pointer table | **Go's interface vtables**, **Rust trait objects**, **Java's `MethodHandle`** |
| DI by constructor arg | **Spring `@Autowired`**, **Guice**, **Wire (Go)**, **Dagger (Android)** |

**The deeper modern shift**: most of these tools moved the enabling point *into the test code itself*. `jest.mock('./db')` reads as if it's an object seam (called from a test) but mechanically is a link seam (intercepts module resolution). Frameworks blurred the categories — but the underlying taxonomy still tells you *what could break* and *where to look* when something does.

#### Worked example — picking a seam for a real situation

You have a graphics-heavy CAD codebase. `CrossPlaneFigure::rerender()` makes 5 direct calls to `drawLine()`, `drawText()`, etc. — all global C functions in a third-party library. The functions return void and write to a framebuffer you can't inspect. What seam?

**Walk the decision:**
1. **Object seam?** Would require introducing a `Renderer` interface and threading it through every figure class — touches hundreds of call sites. *Expensive.*
2. **Link seam?** Compile a `libgraphics_test.a` whose `drawLine` pushes `(LINE_DRAW, x1, y1, x2, y2)` into a `std::queue<GraphicsAction>`. Test inspects the queue. Zero call-site changes. *Cheap.* ✅
3. **Preprocessing seam?** Could `#define drawLine` to a recorder, but every translation unit would need the same header — error-prone and easy to forget. *Brittle.*

**Pick link seam.** The book argues this exact case: when a third-party dependency is *pervasive* and *low-information* ("tell" calls returning void), the link seam wins because it's cheap to install and the substitution naturally records side effects. The downside — "looks like it works locally" — is mitigated by separating `test/` and `prod/` library targets in the build system.

### If you were the legacy-code engineer…
You're handed a 200K-line C++ codebase, no tests, one global function `db_update()` called 142 times. You have a week. What's the move?

**Sequence: link seam first, object seam as the migration.** Week one, create `libdb_test.a` with a `db_update` that appends `(account_no, item)` to a vector you can query. You get 80% of the testing leverage for 10% of the work — most existing tests can now run without a database. *In the same week*, start the slow migration: introduce a `Database` interface with one method `update(int, Item*)`, change one caller at a time to take a `Database*` (object seam), use the interface's default impl to call the global as a temporary shim. Eighteen months later, every caller takes the interface and the global is deletable.

The book's answer is the same shape but more conservative — Feathers doesn't usually recommend mass refactors. The link seam is the *characterization-testing scaffold* that buys you safety to do the slow object-seam migration without breaking production.

### Where this shows up in real systems
- **Linux kernel modules** are a giant preprocessing-seam system. Every `CONFIG_X` flag enables/disables blocks of code at compile time. The same kernel source compiles to a server-tuned binary or a phone-tuned binary by changing 800 `.config` lines — the enabling point is `make menuconfig`.
- **Spring's `@Profile`** is a runtime object seam at scale. `@Profile("test")` beans only get instantiated when the active profile matches — an entire substitute object graph swaps in for tests vs prod. The enabling point is `spring.profiles.active` in application.yml.
- **Jest's automatic mocking** (`jest.mock('./db')`) is preprocessing+link masquerading as object: Jest rewrites the module resolution map at runtime, so when `require('./db')` runs, the fake module is returned. The enabling point is the `jest.mock()` call in the test file — visible like an object seam, but mechanically more like LD_PRELOAD.
- **Cilium and other eBPF-based service meshes** rely on a kernel link seam: BPF programs are attached to syscall hooks, intercepting `connect()`, `accept()`, `sendmsg()` for every process. The enabling point is `bpftool prog attach`. Production observability built on kernel-level link seams.

### Diagnostic questions
1. **A C function is called from 200 places in one file. Which seam is cheapest?** *Wrong:* "object seam, just wrap them all." *Right:* **preprocessing or link seam**. Wrapping 200 call sites is more disruptive than a one-line `#define` or a swapped library. The book is explicit about this trade.
2. **Where is the enabling point of a `MagicMock` substitution in Python's `unittest.mock`?** *Wrong:* "the mock object itself." *Right:* **the `with patch(...)` block or `@patch` decorator** in the test. The mock is the *fake*; the enabling point is the call that installs it into the import system.
3. **A link seam works locally but fails in CI. What's the most likely cause?** *Wrong:* "Different OS version." *Right:* **Library search path order**. The CI environment's linker found the production library first; the local environment found the test library first. Link seams are infamous for this — the enabling point lives in a path-resolution mechanism that's environment-dependent.
4. **Why is `@Profile` (Spring) safer than `#ifdef DEBUG` (C++)?** *Wrong:* "Java is safer." *Right:* `@Profile` evaluates at *application boot* — wrong profile = early crash, easy to diagnose. `#ifdef DEBUG` evaluates at *compile time* — a wrong build produces a binary that runs silently with wrong behavior in production. The enabling point's *binding time* is the safety dimension.
5. **You see `cell.Recalculate()` in code. Is it an object seam?** *Wrong:* "Yes, virtual methods are seams." *Right:* **only if there's an enabling point**. If `cell` is constructed three lines above with `new FormulaCell(...)`, the type is locked; no enabling point, no seam. The seam exists only when *something else* — a constructor arg, a setter, a factory — decides the runtime type.

---

## [2026-05-24] Sensing, Separation, and Fakes — How to See What Code Does When It Won't Tell You · pp.40–53 · Ch.2 §*Test Coverings* (end) → Ch.3 *Sensing and Separation* + Faking Collaborators + Mock Objects → Ch.4 *The Seam Model* (intro)

### TL;DR
The two reasons to break dependencies in legacy code aren't aesthetic — they're **sensing** ("I can't see what this code does because its effects leave through a channel I can't observe") and **separation** ("I can't even instantiate this code to run it because it drags in the database, the network, the hardware"). Both are solved with the same lever — a **seam**, a place where you can substitute one implementation for another without editing in place — typically expressed as an interface with a real implementation (`ArtR56Display`) and a **fake** implementation (`FakeDisplay`) that records what was done to it. Fakes are not "not really testing" — they're how you **localize behavior to a single class** so a test failure points at one thing instead of "something in the system broke." The chapter then introduces the **Legacy Code Change Algorithm** (5 steps: identify change → find test points → break dependencies → write tests → make changes), which is the entire book's table of contents compressed into a loop. The deepest move is the introduction of the **Seam Model** in Ch.4 — the conceptual reframing that makes all of this less ad-hoc.

### Intuition — "this is like…"
Sensing is a **stethoscope problem** — you suspect the heart is doing something wrong but the only data you have is "the patient looks unwell." You need a way to listen to the heart directly without opening the chest. Separation is a **dialysis problem** — you can't observe the kidney working in situ because it's plumbed into every other organ; you need a way to take it out, run it on a bench, and feed it controlled input. Both problems share the same fix: **insert a coupling joint** at the boundary between the organ and the rest of the body, such that the joint can be replaced with measuring instruments. In code, the coupling joint is an interface; the measuring instrument is a fake.

### Mechanics

#### The two distinct dependency-breaking motivations

| Motivation | What the problem looks like | What the fake does |
|---|---|---|
| **Sensing** | Code computes a value or has an effect, but the test can't observe it (writes to a real display, calls hardware, fires an HTTP request to a vendor) | **Records** the effect locally so the test can assert on it (`getLastLine()` returns the last string the production code tried to display) |
| **Separation** | Code can't even be instantiated in a test because the constructor demands the database / network / hardware | **Substitutes** for the unavailable collaborator so the class under test can run at all (`new Sale(new FakeDisplay())` works without a real cash register) |

The two motivations often coexist — `NetworkBridge` on p.45 has *both* problems: you can't instantiate it without real network hardware (separation), and even if you could, you can't see what it did to that hardware (sensing). Solving either alone helps; solving both unlocks the test.

#### The seam — the conceptual lever

A **seam** is a place where you can change program behavior without editing in place. Feathers introduces it formally in Ch.4 (the chunk reaches the chapter's opening) but the Sale/Display refactor on pp.46–49 is the canonical motivating example:

```
   BEFORE (no seam — sensing impossible)
   ──────────────────────────────────
   class Sale {
       void scan(String barcode) {
           ...
           realCashRegisterDisplayAPI.showLine(itemLine);   // ← hardware call, can't test
       }
   }
```

```
   AFTER (seam at Sale ↔ Display)
   ──────────────────────────────
   interface Display { void showLine(String line); }

   class ArtR56Display implements Display { /* real hardware */ }
   class FakeDisplay   implements Display {
       String lastLine = "";
       public void showLine(String s) { lastLine = s; }
       public String getLastLine() { return lastLine; }   // ← test-only sensing
   }

   class Sale {
       private final Display display;                     // ← seam
       Sale(Display d) { this.display = d; }
       void scan(String b) { ... display.showLine(itemLine); ... }
   }

   class SaleTest {
       void testDisplayAnItem() {
           FakeDisplay d = new FakeDisplay();             // ← held as concrete type
           new Sale(d).scan("1");
           assertEquals("Milk $3.99", d.getLastLine());   // ← assert via the test-only side
       }
   }
```

The refactor *adds no production functionality*. It only moves the boundary where you can intercept. The cost is one interface + constructor parameter; the benefit is every future test of Sale becomes trivial.

#### "The two sides of a fake" — the key conceptual move

The fake has **two interface surfaces** the test must hold simultaneously:

```
                                      ┌────────────────────────────┐
   The test holds the fake as         │   FakeDisplay              │
   the concrete type to access        │                            │
   the test-only side:                │  ┌───────────────┐         │
                                      │  │ showLine(s)   │ ← Sale sees this
   FakeDisplay d = new FakeDisplay()  │  │   (Display    │         │
                                      │  │    interface) │         │
   Sale sees the fake as              │  └───────────────┘         │
   the production interface,          │                            │
   knows nothing about getLastLine:   │  ┌───────────────┐         │
                                      │  │ getLastLine() │ ← test sees this
   sale = new Sale(d)                 │  │   (test-only) │         │
                                      │  └───────────────┘         │
                                      └────────────────────────────┘
```

This is why the test declares `FakeDisplay d` (not `Display d`): it needs the *fatter* type so `getLastLine()` is visible. The production code holds the *thinner* type. **Same object, two contracts.** This dual-typing is the entire technique — without it, the test can't sense and the production code can't be substituted.

#### Fakes vs. mocks — the spectrum

| | **Fake** | **Mock** |
|---|---|---|
| What it does | Records calls; test asserts afterward | **Pre-specifies** expected calls; self-verifies via `verify()` |
| Where assertions live | In the test, after `act` | In the mock, set up before `act` |
| Lines of test code | More (manual assert) | Fewer (declarative expectation) |
| Failure messages | "expected X got Y" | "expected showLine('Milk') was never called" |
| Need a framework | No (write it by hand in 5 lines) | Usually yes (Mockito, EasyMock, gMock, unittest.mock) |
| When to use | Default — simpler, no magic | When you have *many* similar fakes to write |

Feathers's stance is conservative: **fakes first, mocks if you find yourself writing too many of them**. Mock frameworks introduce dynamic-proxy magic, sometimes break under refactoring, and can produce tests that over-specify implementation rather than behavior. The community has drifted further toward mocks since 2004 (when WELC was published), but Feathers's note that *"simple fake objects suﬃce in most situations"* remains the safest default — especially in dynamically-typed languages.

#### The Legacy Code Change Algorithm (p.41) — the book's spine

```
   1. Identify change points     ← where do I need to modify?
   2. Find test points           ← where can I observe/assert the change?
   3. Break dependencies         ← what seams must I create to write a test there?
   4. Write tests                ← the safety net
   5. Make changes and refactor  ← now do the thing you came to do
```

Two things to notice:
1. **Steps 1–4 happen before any production behavior changes.** This is counterintuitive — every minute spent here is a minute *not* spent on the feature ticket. Feathers's whole argument is that those minutes are recovered (and then some) the moment you avoid a single production regression.
2. **The chapters of the book map directly to obstacles in this loop.** Stuck on step 3? See Ch.9 ("I Can't Get This Class into a Test Harness") and the dependency-breaking catalog at the back. Stuck on step 2? See Ch.11 ("What Methods Should I Test?"). The book is a *catalog indexed by which step of the loop is blocking you*.

#### "Islands rising out of the ocean" — the long-run vision

The chapter contains its most quoted passage:

> Over time, tested areas of the code base surface like islands rising out of the ocean. Work in these islands becomes much easier. Over time, the islands become large landmasses. Eventually, you'll be able to work in continents of test-covered code.

The geological metaphor matters: the *unit* of test coverage isn't "the test suite covers 80% of lines" but "this region of code is **ergonomic to change** because its seams already exist." Coverage is a *side effect* of having broken dependencies; the dependency-breaking is the real investment. **A class with 100% test coverage and no seams is harder to extend than a class with 50% coverage and clean seams.**

#### The cost of conservatism (a quiet trade-off)

Feathers admits on pp.40–41: dependency-breaking refactors sometimes "leave a scar" — methods get parameters they don't strictly need in production, classes get split in non-ideal ways. The pragmatic position: **accept the scar to get the test, then heal the scar later from inside the safety net.** This inverts the common engineering instinct ("make the design clean before adding tests") because that instinct, applied to legacy code, leaves you forever stuck without tests. The order is: ugly test first → clean refactor second (now safe). The "Primitivize Parameter" and "Extract Interface" refactorings mentioned on p.40 are the catalog entries for these scar-producing-but-test-enabling moves.

### If you were the engineer staring at `NetworkBridge`…
You'd resist the temptation to "just spin up a test cluster" — that's solving the wrong problem (it makes the test possible but slow and flaky, the worst combination per yesterday's entry). Instead you'd ask: **what would I need to substitute** so I can sense and separate? Probably an `EndPoint` interface that the real `RealEndPoint` and a `FakeEndPoint` both implement. The fake records "configure was called with X" and lets the test assert "I expected configure(X)." The constructor now takes `EndPoint[]` of either kind. **Total production behavior change: zero. Total testability change: total.** The book's claim is that this same shape — extract interface, inject collaborator, write fake — handles 80% of legacy-testing problems; the rest is the back-of-book catalog of weirder dependency-breaking techniques (parameterize method, expose static method, link substitution, preprocessing seam).

### Cross-language view

| Language | Seam mechanism | Fake idiom |
|---|---|---|
| Java / C# | Interface + DI constructor | Hand-rolled fake or Mockito/Moq |
| C++ | Virtual function or template parameter | Hand-rolled or gMock; also **link-seam** (swap `.o` files) |
| Python | Duck typing — no interface needed | `class FakeX: def method(self, ...): self.calls.append(...)` or `unittest.mock.Mock` |
| Go | Implicit interface satisfaction | Struct with same method set — `gomock` for the mock variant |
| Rust | Trait + generic param or `Box<dyn Trait>` | Hand-rolled struct impl; `mockall` crate for proc-macro mocks |
| JS/TS | Function injection or duck typing | `jest.fn()`, `sinon.stub()` |

The **seam concept is language-agnostic**; only its expression varies. In dynamic languages (Python, JS, Ruby) the seam is implicit — you can substitute *any* object that quacks right. In static OO languages (Java, C#) you need an explicit interface. In C++, Feathers later devotes a whole chapter to *link seams* and *preprocessing seams* because the language gives you alternatives to virtual functions when you need to avoid runtime overhead.

### Where this shows up in real systems
- **Hexagonal architecture / ports-and-adapters** (Alistair Cockburn, ~2005) is the architectural-scale generalization of the Sale↔Display refactor: every external dependency (DB, queue, HTTP client, filesystem) sits behind a port (interface) and gets a real adapter in prod, a fake adapter in tests. Spring Boot's `@Autowired`, Ruby on Rails' `ActionMailer.deliveries`, Django's `mail.outbox` — all are framework-level expressions of "production injects real, test injects fake."
- **The Linux kernel's `mock` for I/O** — drivers under test use `vfs_mock` and `socket_mock` infrastructure (kunit) to fake filesystem and network calls without spinning up real ones. Same pattern, kernel scale.
- **Cloud SDKs** — AWS SDK for Java exposes interfaces like `S3Client` not because the team loved abstraction but because they needed customers to be able to inject `S3Mock` in tests. Compare to the early AWS SDK (singleton, concrete) where testing required `localstack` (an entire fake AWS process). The interface refactor made unit-level fakes possible.
- **Hardware-in-the-loop testing** in embedded — same conceptual game at the HAL (Hardware Abstraction Layer): every register read/write goes through a `HalRead(addr)` function so tests can substitute a fake HAL that returns canned values.

### Diagnostic questions
1. **A senior engineer says "but the FakeDisplay test doesn't prove the real cash register works." How do you respond?** *They're right — and that's not what the test is for. The test proves Sale formats the display string correctly. A separate integration test proves the cash register hardware renders strings correctly. Conflating the two produces tests that are slow, brittle, and don't localize. The book's "divide and conquer" framing (p.49) is the textbook answer.*
2. **Why does the test declare `FakeDisplay d` and not `Display d`?** *Because the test needs `getLastLine()`, which lives only on the fake side. The production type (`Display`) is too narrow. This dual-typing is the entire idea — the fake satisfies the production contract *and* exposes a test-only inspection contract simultaneously.*
3. **When does `NetworkBridge`'s "spin up real hardware" approach beat the fake-EndPoint approach?** *When you're testing the integration boundary itself (does the real hardware respond to the real protocol?). Never when you're testing logic inside `NetworkBridge`. The mistake is using integration tests for logic — slow, flaky, and they don't localize when they fail.*
4. **What's the failure mode of "extract an interface for everything just in case"?** *Interface explosion — every class has a `IFoo`/`Foo` pair, navigation becomes Go-To-Definition → Go-To-Implementation→ Go-To-Implementation, and the design carries DI ceremony for boundaries that never get substituted. Feathers's discipline: extract interfaces **at known sensing/separation pain points**, not preemptively. YAGNI applies to seams too.*
5. **Why does Feathers say the "scar" is acceptable?** *Because the alternative is no test at all — and untested legacy code degrades forever. A scarred-but-tested class can be cleaned up later from inside its test suite. An ideal-design untested class can only be refactored by prayer. The asymmetry favors the scar.*

### See also
- WELC 2026-05-23 entry — the *why* of safe change (Cover-and-Modify, feedback loops). Today's entry is the *how* — the specific mechanism (seams + fakes) that makes Cover-and-Modify possible in code that resists it.
- WELC Ch.4–8 (coming) — the full Seam Model taxonomy (object seam, link seam, preprocessing seam) and the dependency-breaking catalog at the back of the book.
- *Working Effectively with Unit Tests* (Jay Fields) — modern complement; expands the fake-vs-mock debate with named test-double categories from Meszaros.
- LDDD 2026-05-24 entry — the *core subdomain* discipline mirrors this discipline at the strategic layer: identify the few high-leverage places to invest the most rigor; let supporting/generic stay cheap.
- DDIA Ch.4 — schema-evolution discipline depends on the same principle: identify the seam (encoding format) where producer and consumer can evolve independently.

---

## [2026-05-23] Edit-and-Pray vs. Cover-and-Modify — The Feedback Loop as the Unit of Safe Change · pp.26–39 · Ch.1 *Changing Software* + Ch.2 *Working with Feedback* (Four Reasons → Risky Change → Cover-and-Modify → What Is Unit Testing?)

### TL;DR
Feathers reframes every code change — feature add, bug fix, refactor, optimization — as a single decision about **what behavior changes vs. what behavior must be preserved**. The hard part of legacy work is almost never the changing part; it's the *preserving* part, because the preserved surface area is huge and you usually don't know which bits of it are at risk. Two strategies exist: **Edit and Pray** (think hard, change carefully, poke around to verify) and **Cover and Modify** (wrap the code in tests first, then change freely under the safety net). Edit-and-Pray scales as **O(your courage)**; Cover-and-Modify scales as **O(your test suite's speed × locality)**. The chapter closes by defining the *non-negotiable* properties of a unit test — **fast** (sub-100ms) and **localizing** (failure points at a single class or function) — because those two properties are what make the feedback loop tight enough to drive change at all.

### Intuition — "this is like…"
Edit-and-Pray is a surgeon operating with a butter knife but *with care* — the care isn't fake, but it doesn't compensate for the missing scalpel. Cover-and-Modify is the surgeon with proper instruments and an anesthesiologist watching vitals: the surgeon still concentrates, but the **monitoring** is what tells them, in real time, "you nicked something" rather than "we'll find out tomorrow during recovery." The whole book is an argument that **monitoring is a more important capability than care**, because care doesn't scale linearly with complexity but tooling does.

### Mechanics

#### The four reasons to change software (the chapter's organizing frame)

| Reason | Structure | Existing functionality | Resource use |
|---|---|---|---|
| **Add a feature** | changes | preserved (new functionality added alongside) | — |
| **Fix a bug** | changes | small targeted change | — |
| **Refactor** | changes | **invariant** (this is the definition) | — |
| **Optimize** | — (ideally) | invariant | changes |

Feathers's payoff insight on the second table: **all four are dominated by what stays the same, not what changes.** Even bug-fixing changes maybe 0.1% of the program's behavior — the other 99.9% must survive untouched. Refactoring and optimization make this explicit (functionality must be invariant by definition), but feature-adds and bug-fixes are *also* mostly behavior-preservation tasks dressed up as behavior-changing ones.

```
        Existing behavior (large)              Changed behavior (tiny)
        ┌────────────────────────────────────┐ ┌─────┐
        │                                    │ │     │
        │            preserve at all costs   │ │ new │
        │                                    │ │     │
        └────────────────────────────────────┘ └─────┘
                       ↑
              this is the actual job
```

#### The three risk questions
Every change in a legacy codebase reduces to:
1. **What changes do we have to make?**
2. **How will we know we've done them correctly?**
3. **How will we know we haven't broken anything?**

Edit-and-Pray answers (2) and (3) with "I'll be careful and poke around." Cover-and-Modify answers (2) with a new test for the intended change and (3) with the pre-existing test suite. The asymmetry is that (3) is *unbounded* in size while your attention is bounded — you cannot poke around the whole system.

#### Why "avoid all change" is the worst strategy
Teams under fear of breakage develop the *don't-touch-it* doctrine, which has three predictable failure modes:
1. **Classes/methods grow** because nobody splits them — the system gets harder to change next time.
2. **Refactoring skill atrophies** — extracting a method becomes scary because it's no longer routine.
3. **Fear compounds** — the team doesn't realize how much fear it carries until they get tests and the fear lifts.

The deep point: **avoiding change makes the codebase asymmetrically worse over time**, because all forces (entropy, new requirements, new dependencies) push toward more code, while the only force pushing toward *cleaner* code is deliberate restructuring. Defer restructuring and you've chosen entropy.

#### The Edit-and-Pray feedback loop vs. Cover-and-Modify

```
   EDIT-AND-PRAY                              COVER-AND-MODIFY
   ─────────────                              ────────────────
   Think about change.                        Add/find tests around target.
   Make change.                               Run tests — green baseline.
   Run app, poke around.                      Make change.
   Submit to QA team.                         Run tests — feedback in seconds.
   ─── 1 day later ───                        Loop ~50 times per session.
   Bug report "AE1029 failed" arrives.        Discover regression at change N.
   Bisect: was it your change or theirs?      Roll back change N (it's tiny).
   Debug.                                     Continue.
   
   Loop length: ~24 hours.                    Loop length: ~seconds.
   Localization: terrible (was it me?).       Localization: perfect (it was me, just now).
```

The economics are stark: a 1-second feedback loop run 50 times costs 50 seconds. A 1-day feedback loop run 50 times costs 50 days. **The loop length is the multiplier on every engineering activity** — debugging, refactoring, feature work, onboarding. This is why Feathers spends an entire book on getting tests around code: he's optimizing the multiplier, not any single activity.

#### The non-negotiable definition of a unit test
Feathers is dogmatic about two qualities:
1. **They run fast.** *"A unit test that takes 1/10th of a second to run is a slow unit test."*
2. **They localize problems.** Failure should point at one class/function, not "something in the system broke."

He's also explicit about what disqualifies a test from being a unit test (regardless of what its author calls it):
- Talks to a database
- Communicates across a network
- Touches the filesystem
- Requires environment setup (editing config files, etc.)

These aren't bad tests — they're **integration tests**, with a different role. The problem is when you have only integration tests, the feedback loop is too slow to drive change. Feathers's whole legacy-code technique catalogue (Chs. 9–11) is fundamentally about **breaking dependencies so that unit tests become possible at all** — most legacy code is "untestable" not because the logic is bad but because it can't be instantiated without dragging in the database, the network, and the filesystem.

#### Worked numerical example: why 1/10 second matters
A project with 3,000 classes × ~10 tests = **30,000 tests**.

| Per-test time | Full suite |
|---|---|
| 0.01 s (10 ms — "real" unit test) | **5 min** — run on every save |
| 0.10 s (100 ms — "slow" unit test) | **50 min** — run before lunch, maybe |
| 1.00 s (1 s — integration-shaped) | **8.3 hr** — overnight only |
| 10.00 s (10 s — full app spin-up) | **3.5 days** — never |

The order-of-magnitude jump in per-test time changes the **cadence of feedback**, which changes what's possible in the development loop. At 5 minutes, you run the suite after every meaningful change. At 50 minutes, you batch changes and accept slower bisection. At overnight, you're back to Edit-and-Pray.

### If you were the developer staring at a legacy module without tests…
Feathers's instruction: **don't refactor first** (you'll break things and won't know) and **don't add the feature first** (you'll entangle new behavior with broken old behavior). Instead, find the smallest seam where you can instantiate the class in a test harness — even if it requires ugly subclassing or extracting an interface just for testing. Get *one* test green. Then another. Now you have the safety net to refactor for clarity, and only then add the feature. The book's later chapters (esp. "I Can't Get This Class Into a Test Harness") are the technique catalog for that first step.

### Where this shows up in real systems
- **Google's testing pyramid culture** is built on Feathers's two qualities elevated to policy: unit tests must run in <100ms, must not touch databases or networks, and must localize. The "small/medium/large" test taxonomy at Google maps almost 1:1 to "unit / integration / system" with explicit budgets per tier.
- **TDD as practiced today** (write test → see it fail → write code → see it pass → refactor) is Cover-and-Modify formalized into a single-step workflow. The "refactor" step is *only* safe because the tests written one minute earlier act as the vise.
- **Continuous deployment pipelines** at companies like Etsy, Netflix, and Stripe are predicated on the feedback-loop math: if your tests run in 5 minutes, you can deploy 50 times a day; if they run in an hour, you deploy weekly. The deploy cadence is downstream of the test cadence, which is downstream of whether your tests are unit-shaped.

### Diagnostic questions
1. **Refactoring is defined as "structural change with invariant behavior." What does this imply about the relationship between refactoring and tests?** *Wrong answer: "Tests prove the refactor is correct." → Tests can only show that the **tested** behavior is invariant. Untested behavior may have changed silently. The honest framing is: tests bound the surface area of "behavior known to be preserved." Refactoring without tests is a behavior-preservation claim with no evidence.*
2. **Why is a test that takes 1 second "slow"?** *Because a 30,000-test suite at 1s/test = 8 hours, which breaks the inner feedback loop. The bound isn't "slow in absolute terms" — it's "too slow to run on every change."*
3. **What's the failure mode of teams that say "we'll just be careful"?** *Care doesn't scale with system complexity. As the system grows, the surface area you must mentally track exceeds working memory, and "careful" becomes statistical roulette dressed up as professionalism.*
4. **Why does Feathers call "Edit and Pray" the industry standard?** *Because most legacy code lacks the dependency-breaking seams needed to write unit tests against it. Edit-and-Pray isn't a choice — it's the default when Cover-and-Modify is unavailable. The book's purpose is to make Cover-and-Modify available.*
5. **If unit tests can't touch the database, how do you test database code?** *You don't unit-test it; you integration-test it. The unit-testable code is the logic that builds queries, parses results, and handles errors — extract that out from the I/O code so it can be tested in isolation. This is the dependency-inversion principle applied to data access.*

### See also
- WELC Ch.4 *The Seam Model* (coming up) — the technique catalog for *creating* the testability that this chapter takes as a precondition.
- TDD/Kent Beck → same feedback-loop logic, expressed as a rhythm rather than a remediation strategy.
- DDIA Ch.4 *Encoding and Evolution* — schema migrations are a behavior-preservation problem at the data layer; the same "tiny change, huge preserved surface area" shape applies.
- N2T Ch.5 (hardware testing) — the hardware project ships with test scripts and compare files for exactly this reason: even at the gate level, "Cover and Modify" beats "Edit and Pray."

---
