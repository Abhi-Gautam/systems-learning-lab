# REF Notes

_Entries follow the template at `Notes/TEMPLATE.md`. Append-only. **Newest entry at top**, immediately after this header._

---

## [2026-05-21] The Video Store Refactoring — Extract Method, Move Method, Replace Conditional with Polymorphism (Fowler's Canonical First Example) · pp.13–42 · Ch.1 Refactoring, a First Example (full chapter: Starting Point → Comments on the Program → First Step → Decomposing & Redistributing `statement` → Replacing the Conditional with Polymorphism → Final Thoughts)

### TL;DR
Fowler's canonical opening example takes a 40-line procedural `statement()` method that prints a video-store customer's bill and refactors it through **three named transformations** — Extract Method, Move Method, Replace Conditional with Polymorphism — into a small object graph where each class knows how to compute its own pricing and frequent-renter points. Each step is **behavior-preserving** (the test suite passes after every micro-edit), and the **only** justification for the work is that an upcoming feature request ("also print an HTML statement" + "introduce a new movie category") would have been painful in the original shape but is trivial in the refactored shape.

### Intuition — "this is like…"
Refactoring is **moving a piano without changing the song**. Each small move (Extract a chord into a function, Move a function to where it belongs) is mechanical and reversible. The trick is that you make the moves *before* you need to change the music — so when the band asks for a key change, the piano is already standing where the new player can reach it.

### Mechanics

#### 1. The starting shape — one big `statement()` method

```
   Customer
   ├── name
   ├── rentals: List<Rental>
   └── statement() : String   ← 40-line method doing 4 things:
                              ┌─ format header
                              ├─ loop rentals:
                              │   ┌─ switch on movie type
                              │   ├─ compute amount
                              │   ├─ compute frequent-renter points
                              │   ├─ append line
                              │   └─ accumulate totals
                              └─ format footer

   Rental         Movie
   ├── movie      ├── title
   └── days       └── priceCode  // REGULAR | NEW_RELEASE | CHILDRENS
```

Two looming change requests force the refactor:
1. **HTML statement** — duplicating the 40-line method only changing string-concat character is intolerable.
2. **New movie category / change pricing rules** — a switch on `priceCode` inside Customer is the wrong place to edit.

#### 2. The refactoring sequence (each step preserves behavior)

| Step | Refactoring name | What changes | Why |
|---|---|---|---|
| 1 | **Extract Method** | Pull `amountFor()` out of statement | Isolate a coherent chunk; name documents intent |
| 2 | **Rename Variable** | `aRental` → `each` in extracted method | Local naming context is different |
| 3 | **Move Method** | Move `amountFor()` to Rental, call it `getCharge()` | Method uses *only* Rental data — belongs on Rental |
| 4 | **Replace Temp with Query** | `thisAmount` temp → call `each.getCharge()` | Eliminate scratch variables in loop |
| 5 | **Extract Method** | Pull frequent-renter calc out | Same pattern, applied to second concern |
| 6 | **Move Method** | Move to Rental as `getFrequentRenterPoints()` | Same justification |
| 7 | **Replace Temp with Query** | Replace loop totals with helper methods on Customer | Now `statement()` is pure formatting |
| 8 | **Replace Type Code with State/Strategy** | Movie's `priceCode` int → `Price` subclass hierarchy | Open `getCharge()` to subclass override |
| 9 | **Replace Conditional with Polymorphism** | Move the `switch(priceCode)` body into `Price.getCharge()` per subclass | Adding a new category = adding a class |

#### 3. Before → after — code shape comparison

```
   BEFORE (procedural):                          AFTER (polymorphic):

   Customer.statement():                         Customer.statement():
   ┌────────────────────────────────┐            ┌────────────────────────────────┐
   │ loop rentals:                  │            │ loop rentals:                  │
   │   switch(movie.priceCode):     │            │   result += each.getCharge()   │ ← Rental
   │     case REGULAR: ...          │            │ totalAmount = getTotalCharge() │ ← Customer helper
   │     case NEW_RELEASE: ...      │            │ points = getTotalFrequentRen…  │
   │     case CHILDRENS: ...        │            └────────────────────────────────┘
   │   thisAmount = ...             │
   │   frequentRenterPoints += ...  │            Rental.getCharge():
   │ totals += thisAmount           │            ┌────────────────────────────────┐
   │ format line                    │            │ return movie.getCharge(days)   │ ← delegate to Movie
   └────────────────────────────────┘            └────────────────────────────────┘

                                                 Movie.getCharge(days):
   ALL CHANGES TO PRICING                        ┌────────────────────────────────┐
   = edit one 40-line method                     │ return price.getCharge(days)   │ ← delegate to Price
   = high risk of bug                            └────────────────────────────────┘

                                                 Price (abstract).getCharge(days):
                                                 ├── RegularPrice.getCharge() { ... }
                                                 ├── NewReleasePrice.getCharge() { ... }
                                                 └── ChildrensPrice.getCharge() { ... }

                                                 ADD A NEW CATEGORY
                                                 = create one new subclass
                                                 = touch zero existing code (OCP)
```

#### 4. The two-hat rule (Kent Beck, quoted by Fowler)

```
   ┌──────────────────────┐         ┌──────────────────────┐
   │  Adding Function     │         │     Refactoring      │
   │  hat                 │         │     hat              │
   ├──────────────────────┤         ├──────────────────────┤
   │ New tests            │         │ NO new tests         │
   │ Behavior changes     │         │ Behavior preserved   │
   │ Code shape may decay │         │ Code shape changes   │
   │                      │         │  only                │
   └──────────────────────┘         └──────────────────────┘
              ↑                                ↑
              └──── never both at once ────────┘
```

Wearing the wrong hat (or both) is the #1 source of "I broke things while cleaning up." Fowler's discipline: **swap hats consciously and never edit code under both hats simultaneously.**

#### 5. The micro-step heuristic — why so many tiny steps?

| Big-bang rewrite | Refactoring (Fowler) |
|---|---|
| Edit for hours, test at end | Edit 1–5 lines, test, commit |
| Bug found at end → bisect hours of changes | Bug found → roll back last micro-step |
| Hard to interleave with feature work | Each micro-step is safe to ship |
| Easy to get lost in scope | Each step has a single purpose |

The reason Fowler can keep a refactor coherent over 9 steps is that **each step is a named transformation with a known mechanical recipe** (in the book's catalogue chapters 6–12). You don't invent each move — you select from a known menu.

#### 6. Where the test suite has to be (and why)

```
   The refactoring loop:
   ┌─────────────────────────────────────────────────┐
   │ 1. Test suite green                             │
   │ 2. Pick one micro-refactoring                   │
   │ 3. Apply mechanical recipe                      │
   │ 4. Test suite green again? → commit; goto 2     │
   │    Red? → undo immediately; investigate; goto 2 │
   └─────────────────────────────────────────────────┘
```

Without (1) and (4), you can't tell whether step 3 broke anything. *The test suite is the safety net that lets you make moves you'd otherwise be too scared to make.*

### Where this shows up in real systems
- **Strategy pattern in payment processing**: PayPal/Stripe/ACH each handle `processPayment(amount)` differently — same `Replace Conditional with Polymorphism` move.
- **Visitor pattern in compilers** (LLVM, Clang): different AST node types handle `visit()` differently — same shape, different verbs.
- **Discount calculators in e-commerce**: pricing rules per product type / promotion tier — exactly the `Movie → Price` hierarchy generalized.
- **Test-driven refactoring in CI**: GitHub PR-template lines "Did you run the test suite? Did you split behavior changes from refactors?" — Beck's two-hat rule institutionalized.

### Diagnostic questions
1. *"Why doesn't Fowler combine multiple micro-steps before running tests?"* — Because if a test fails after 5 stacked edits, you can't tell *which one* broke it. Bisection costs more than the discipline of testing per step. (Wrong: "tests are slow" — Fowler explicitly says fast tests are non-negotiable for this discipline.)
2. *"Why move `amountFor()` from Customer to Rental?"* — Because the data it operates on (`rental.days`, `rental.movie`) lives on Rental. Putting the method on the class that owns the data is the *Feature Envy* code smell remedy. (Wrong: "Rental is a smaller class" — it's about *data locality*, not size.)
3. *"What's the difference between Replace Type Code with Subclasses and Replace Type Code with State/Strategy?"* — Subclasses: the type code is immutable (object never changes priceCode). State/Strategy: the type code can change at runtime. Movies do change category over time → State/Strategy. (Wrong: "they're the same" — they differ in *mutability* assumptions.)
4. *"Why is statement() still on Customer after the refactor, not on Rental?"* — Because the *concept* of a statement spans rentals (header, multiple lines, footer, totals). It's a Customer-level concern that *uses* Rental-level data. (Wrong: "it should be a free function" — Customer is the natural orchestrator object.)
5. *"What goes wrong if you start the HTML statement feature *before* the refactor?"* — You either (a) duplicate the 40-line method, or (b) parameterize it with a `format` argument, growing it to ~60 lines of mixed concerns. Both make the *next* feature harder. (Wrong: "ship the feature, refactor later" — "later" is a place you never go.)

### See also
- REF Ch.2 (Principles in Refactoring) — the *what* and *when* once you've seen the *how*.
- REF Ch.6–12 (the refactoring catalogue) — the menu of named transformations Fowler picks from.
- TPP 2026-05-19 (Tracer Bullets, etc.) — orthogonal practices for managing uncertainty; refactoring is the *post-tracer* discipline.
- WELC (Working Effectively with Legacy Code) — the natural successor; what to do when you *don't* have the test suite step 1 assumes.
- CC (Clean Code) — the SOLID/principles framing of why polymorphism beats switch-on-type.

