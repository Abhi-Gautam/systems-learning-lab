// ============================================================================
//  pointers.cpp  --  Pointers from the silicon up.
// ----------------------------------------------------------------------------
//  THE CONCEPT LADDER (we climb it in this exact order, every time):
//
//      1. PHYSICAL THING  : memory is a giant row of 1-byte cells. Each cell
//                           has a number stamped on it. That number is its
//                           ADDRESS. A byte holds 8 bits; that's the smallest
//                           thing the machine can name.
//      2. CPU / ISA VIEW  : the CPU has registers. To touch memory it issues a
//                           LOAD (read cell at address A into a register) or a
//                           STORE (write a register's value into cell at A).
//                           An "address" is just a number sitting in a register.
//                           On arm64 that number is 64 bits wide -> 8 bytes.
//      3. OS / RUNTIME    : the OS hands each process a private VIRTUAL address
//                           space. Locals live in the STACK (grows downward from
//                           high addresses). new/malloc carve from the HEAP (a
//                           different region). The hardware MMU translates these
//                           virtual addresses to real RAM behind your back.
//      4. LANGUAGE LAST   : ONLY NOW do we say the word "pointer". A C++ pointer
//                           is just a variable whose value is one of those
//                           address-numbers. Everything else (&, *, references,
//                           const, smart pointers) is sugar over LOAD/STORE on
//                           an address. We never wave our hands; we print the
//                           actual numbers.
//
// ----------------------------------------------------------------------------
//  BUILD & RUN (exactly this):
//      clang++ -std=c++17 -O0 -g -Wall -Wextra pointers.cpp -o /tmp/pointers \
//          && /tmp/pointers
//
//  Target: macOS arm64 (Apple clang). Pointers are 64-bit = 8 bytes.
//
// ----------------------------------------------------------------------------
//  WHAT YOU'LL BE ABLE TO ARGUE AFTER THIS:
//      You'll be able to say, precisely and from first principles, that a
//      pointer is nothing magical -- it's a fixed-width integer that happens to
//      name a byte cell, and that &x asks "what number names x?", while *p says
//      "go LOAD/STORE the cell whose number is held in p". You'll know why a
//      reference is the same load/store with no rebinding, why p+1 jumps
//      sizeof(T) bytes and not one, why the stack and heap sit in different
//      neighborhoods, why a returned-local pointer dangles, what RAII actually
//      guarantees, and -- the climax -- why chasing a linked list is brutally
//      slower than scanning an array even though it does the identical number
//      of additions: the array respects the 64-byte cache line and the
//      hardware prefetcher; the scattered list defeats both and eats a cache
//      miss on every hop. You could hold that line in a debate with Linus.
// ============================================================================

#include <cstdio>      // printf, for clean %p address formatting
#include <cstdint>     // uintptr_t, intptr_t -- integer types wide enough for an address
#include <cstdlib>     // malloc / free
#include <cstring>     // (not strictly needed; here for completeness)
#include <iostream>    // std::cout for a few spots
#include <vector>      // contiguous storage for the cache benchmark
#include <memory>      // unique_ptr, shared_ptr, weak_ptr
#include <chrono>      // high_resolution_clock for timing
#include <random>      // to shuffle linked-list node order (defeat locality)
#include <algorithm>   // std::shuffle
#include <numeric>     // std::iota

// A tiny helper so every section prints a loud, labeled banner. Seeing the
// output split into sections is half the lesson: each block is a demo you can
// match back to the code.
static void banner(const char* title) {
    std::printf("\n==== %s ====\n", title);
}

// printf's %p wants a (void*). We funnel every address through this so the
// formatting is consistent and we never accidentally print the *value* when we
// meant the *address*. Casting a pointer to void* says "treat this purely as a
// raw address number, forget what it points at".
static void show_addr(const char* label, const void* p) {
    std::printf("    %-22s = %p\n", label, p);
}

// ----------------------------------------------------------------------------
// Forward-declared demo functions for SECTION 04 (pass-by-*). Defined later.
// ----------------------------------------------------------------------------
static void take_by_value(int copy);
static void take_by_pointer(int* addr);
static void take_by_reference(int& alias);

// ----------------------------------------------------------------------------
// SECTION 11 helper: a struct that announces its own birth and death so the
// reader can SEE deterministic destruction (RAII). Printing in ctor/dtor turns
// "the destructor runs at scope exit" from a claim into something you watch.
// ----------------------------------------------------------------------------
struct Noisy {
    int id;
    explicit Noisy(int i) : id(i) {
        std::printf("    [Noisy %d] constructed  (alive at %p)\n", id, (void*)this);
    }
    ~Noisy() {
        std::printf("    [Noisy %d] destroyed    (was at  %p)\n", id, (void*)this);
    }
};

// SECTION 11 cycle demo: two nodes that point at each other. With shared_ptr in
// BOTH directions you get a reference cycle that never hits refcount 0 -> leak.
// Swapping one side to weak_ptr breaks the cycle.
struct CycleStrong {
    std::shared_ptr<CycleStrong> partner; // strong both ways == leak
    int id;
    explicit CycleStrong(int i) : id(i) {
        std::printf("    [CycleStrong %d] ctor\n", id);
    }
    ~CycleStrong() {
        std::printf("    [CycleStrong %d] dtor\n", id);
    }
};
struct CycleWeak {
    std::weak_ptr<CycleWeak> partner;     // weak does NOT bump the count
    int id;
    explicit CycleWeak(int i) : id(i) {
        std::printf("    [CycleWeak %d] ctor\n", id);
    }
    ~CycleWeak() {
        std::printf("    [CycleWeak %d] dtor\n", id);
    }
};

// SECTION 12: linked-list node for the cache benchmark.
struct Node {
    long value;   // payload we will sum
    Node* next;   // address of the next node -- a pointer to chase
};

// ============================================================================
int main() {
    std::printf("POINTERS FROM THE SILICON UP  (macOS arm64, 64-bit addresses)\n");

    // ========================================================================
    banner("01. WHAT IS AN ADDRESS");
    // PHYSICAL: declare three ints. Each one is stored in some byte cells in
    // RAM. Because they are plain int, each occupies sizeof(int) bytes (4 on
    // this platform). The COMPILER decided where on the stack to put them.
    // CPU VIEW: &a means "give me the number that names the first byte cell of
    // a". That number is the address. It is literally an integer; we even print
    // it as one to drive the point home.
    int a = 11;
    int b = 22;
    int c = 33;

    std::printf("    sizeof(int)   = %zu bytes  (each int spans this many cells)\n", sizeof(int));
    std::printf("    sizeof(int*)  = %zu bytes  (an ADDRESS is this wide -> 64-bit)\n", sizeof(int*));
    show_addr("&a (address of a)", &a);
    show_addr("&b (address of b)", &b);
    show_addr("&c (address of c)", &c);

    // The same address, but printed as a plain unsigned integer. uintptr_t is
    // an unsigned integer type guaranteed wide enough to hold any pointer. This
    // is the proof that "an address is just a number".
    std::printf("    &a as a raw integer = %llu (0x%llx)\n",
                (unsigned long long)(uintptr_t)&a,
                (unsigned long long)(uintptr_t)&a);

    // Neighboring addresses: a, b, c were declared together, so their cells sit
    // close. The byte distance between them is a real, printable number.
    std::printf("    distance &a -> &b = %lld bytes\n",
                (long long)((intptr_t)&b - (intptr_t)&a));
    std::printf("    distance &b -> &c = %lld bytes\n",
                (long long)((intptr_t)&c - (intptr_t)&b));
    // TAKE-AWAY: memory is numbered cells. An address is the cell's number.
    // The number is 8 bytes wide here because the CPU's address registers are
    // 64 bits.

    // ========================================================================
    banner("02. THE & (ADDRESS-OF) AND * (DEREFERENCE) OPERATORS");
    int x = 7;
    // DECLARATION: `int* p` -- read it as "p is a variable of type 'pointer to
    // int'". The star here is part of the TYPE. p will hold an address that
    // names an int-sized region. `&x` produces that address.
    int* p = &x;

    show_addr("&x (where x lives)", &x);
    show_addr("p  (what p holds) ", p);
    std::printf("    *p (LOAD the cell p names) = %d   (== x, which is %d)\n", *p, x);

    // DEREFERENCE / STORE: `*p = 5` -- here the star is the dereference
    // OPERATOR, not a type. It means "go to the address in p and STORE 5 into
    // that int cell". Because p names x, this mutates x itself.
    *p = 5;
    std::printf("    after *p = 5 :  x = %d   (we wrote through the pointer)\n", x);

    // SAME GLYPH, TWO JOBS:
    //   * in a DECLARATION (int *q) : "q is a pointer" -- describes a type.
    //   * in an EXPRESSION  (*q)    : "follow q to the cell it names" -- an action.
    // The compiler tells them apart by position; humans must too.
    // TAKE-AWAY: & turns a name into the number that locates it; * turns a
    // number-in-a-pointer back into the cell it names (for read or write).

    // ========================================================================
    banner("03. A POINTER IS ITSELF A VARIABLE WITH ITS OWN ADDRESS");
    // p is not abstract. It is a real 8-byte variable living in its own cells
    // on the stack. So p has THREE distinct things worth printing:
    //   &p  : the address of the pointer variable itself (where p is stored)
    //   p   : the value stored in p (which is the address of x)
    //   *p  : the value at the address p holds (i.e. x's value)
    show_addr("&p (where p itself lives)", &p);
    show_addr("p  (the address p stores) ", p);
    std::printf("    *p (value at that address) = %d\n", *p);
    std::printf("    sizeof(p) = %zu bytes  (p is an 8-byte slot holding one address)\n", sizeof(p));
    // Notice &p and &x are DIFFERENT cells: the pointer and the thing it points
    // to are two separate boxes in memory. p just happens to contain x's number.
    // TAKE-AWAY: "pointer" is not special storage -- it's an ordinary 8-byte
    // variable whose contents we have agreed to interpret as an address.

    // ========================================================================
    banner("04. PASS BY VALUE vs BY POINTER vs BY REFERENCE");
    int n = 100;
    std::printf("    BEFORE: n = %d  (&n = %p)\n", n, (void*)&n);

    // BY VALUE: the function gets a COPY in a brand-new cell. Mutating the copy
    // cannot touch n. We print the copy's address inside the function to PROVE
    // it lives somewhere else.
    take_by_value(n);
    std::printf("    after take_by_value:     n = %d  (unchanged -- it only saw a copy)\n", n);

    // BY POINTER: we hand over n's ADDRESS. The function does *addr = ... which
    // STOREs straight into n's cells. n changes.
    take_by_pointer(&n);
    std::printf("    after take_by_pointer:   n = %d  (changed -- we passed the address)\n", n);

    // BY REFERENCE: `int&` is an alias. Under the hood the compiler passes the
    // same address as a pointer would, but the syntax is plain (no & at the
    // call site, no * inside). It mutates n too.
    take_by_reference(n);
    std::printf("    after take_by_reference: n = %d  (changed -- alias to the same cell)\n", n);
    // TAKE-AWAY: "pass by value" copies bytes; the callee can't reach the
    // original. To let a callee edit your variable, give it the ADDRESS --
    // explicitly (pointer) or implicitly (reference).

    // ========================================================================
    banner("05. REFERENCES vs POINTERS");
    int r_target = 42;
    int& ref = r_target;     // ref is now just another NAME for r_target.
    int* ptr = &r_target;    // ptr HOLDS the address of r_target.

    std::printf("    r_target = %d\n", r_target);
    ref = 50;                // writes through the alias -> r_target becomes 50
    std::printf("    after ref = 50 : r_target = %d  (reference edits the original)\n", r_target);
    *ptr = 60;               // writes through the pointer -> r_target becomes 60
    std::printf("    after *ptr = 60: r_target = %d  (pointer edits the original)\n", r_target);

    show_addr("&ref      (alias -> same cell as r_target)", &ref);
    show_addr("&r_target                                 ", &r_target);
    // &ref == &r_target: a reference has NO storage of its own to point at; it
    // IS the variable under another name. A pointer (ptr) had its own &ptr.

    // DIFFERENCES (demonstrated where safe, explained where not):
    int other = 999;
    ptr = &other;            // POINTERS REBIND: ptr now names a different cell.
    std::printf("    pointer rebound: *ptr = %d (ptr now points at `other`)\n", *ptr);
    // ref = other;          // <-- this does NOT rebind ref. It COPIES other's
    //                        //     value into r_target. A reference can never
    //                        //     be re-seated after init.
    // int& bad;             // <-- ILLEGAL: a reference MUST bind at birth.
    // int& z = nullptr;     // <-- you don't make null references in practice.
    // ptr = nullptr;        // <-- legal: pointers can be null (means "names nothing").
    ptr = nullptr;
    std::printf("    ptr can be null: ptr = %p (references cannot)\n", (void*)ptr);
    // ptr + 1               // <-- legal pointer ARITHMETIC; references have none.
    // IDIOM: use a reference when the thing always exists and you won't rebind
    // (function params you must edit, range-for elements, operator overloads).
    // Use a pointer when it may be null, may rebind, or you need arithmetic
    // (optional output params, data structures, walking arrays).

    // ========================================================================
    banner("06. POINTER-TO-POINTER (chaining address-of)");
    int xx = 7;
    int*  pxx = &xx;     // pxx names xx
    int** ppxx = &pxx;   // ppxx names pxx  (the address OF a pointer)
    // Each extra * in the type is one more "hop". &pxx is legal because pxx,
    // being a variable, has its own address -- exactly the SECTION 03 insight.
    std::printf("    xx     = %d        (the value)\n", xx);
    show_addr("&xx                          ", &xx);
    show_addr("pxx   (holds &xx)            ", pxx);
    std::printf("    *pxx   = %d        (one hop: xx)\n", *pxx);
    show_addr("&pxx                         ", &pxx);
    show_addr("ppxx  (holds &pxx)           ", ppxx);
    show_addr("*ppxx (one hop: == pxx)      ", *ppxx);
    std::printf("    **ppxx = %d        (two hops: ppxx -> pxx -> xx)\n", **ppxx);
    // TAKE-AWAY: ** is not exotic -- it's a pointer whose pointed-to value is
    // itself an address. Each * peels one layer; you stop when you reach the
    // payload. (int*** would be three peels, same idea.)

    // ========================================================================
    banner("07. CONST CORRECTNESS (read the declaration right-to-left)");
    int v1 = 1, v2 = 2;

    // (a) `const int* pc`  == "pointer to const int". The POINTEE is read-only
    //     through pc; the pointer itself can roam.
    const int* pc = &v1;
    //  *pc = 9;      // <-- ILLEGAL: cannot write through pointer-to-const.
    pc = &v2;         //     LEGAL: repoint it; the pointer is mutable.
    std::printf("    const int* pc      : can repoint (pc=&v2 ok, *pc reads %d), cannot do *pc=9\n", *pc);

    // (b) `int* const cp`  == "const pointer to int". The POINTER is frozen
    //     (must init now, can't repoint); the pointee is writable.
    int* const cp = &v1;
    *cp = 9;          //     LEGAL: edit the pointee.
    //  cp = &v2;     // <-- ILLEGAL: cannot repoint a const pointer.
    std::printf("    int* const cp      : can do *cp=9 (v1 now %d), cannot repoint cp\n", v1);

    // (c) `const int* const ccp` == both frozen: can't repoint, can't write through.
    const int* const ccp = &v2;
    //  *ccp = 5;     // <-- ILLEGAL: pointee is const.
    //  ccp = &v1;    // <-- ILLEGAL: pointer is const.
    std::printf("    const int* const   : neither *ccp=5 nor repoint allowed (reads ok: %d)\n", *ccp);
    // TRICK: read the declaration right-to-left. `int* const` -> "const pointer
    // to int". `const int*` -> "pointer to int-that-is-const". The word before
    // the * binds to the pointee; `const` after the * binds to the pointer.

    // ========================================================================
    banner("08. POINTER ARITHMETIC + ARRAYS (decay & stride)");
    int arr[5] = {10, 20, 30, 40, 50};
    // ARRAY DECAY: in most expressions the array NAME `arr` becomes a pointer
    // to its first element -- i.e. arr behaves like &arr[0]. The 5 ints sit in
    // ONE contiguous block of 5*sizeof(int) bytes. No gaps.
    int* base = arr;     // decay: base == &arr[0]
    show_addr("arr (decays to &arr[0])", (void*)arr);
    show_addr("&arr[0]                ", (void*)&arr[0]);

    // STRIDE: base + 1 does NOT add 1 byte. The compiler scales by sizeof(int).
    // "Move one element forward" = "move sizeof(int) bytes forward". The CPU
    // computes address = base + i*sizeof(int) under the hood.
    std::printf("    sizeof(int) = %zu, so each step is %zu bytes\n", sizeof(int), sizeof(int));
    for (int i = 0; i < 5; ++i) {
        // PROVE the three equivalences:
        //   arr + i   == &arr[i]            (same address)
        //   *(arr+i)  == arr[i]             (subscript IS pointer deref)
        long stride = (i == 0) ? 0
                    : (long)((intptr_t)(arr + i) - (intptr_t)(arr + i - 1));
        std::printf("    i=%d  arr+i=%p  &arr[i]=%p  same=%s  *(arr+i)=%d  arr[i]=%d  step=%ld B\n",
                    i, (void*)(arr + i), (void*)&arr[i],
                    ((arr + i) == &arr[i]) ? "yes" : "no",
                    *(arr + i), arr[i], stride);
    }
    // arr[i] is literally defined as *(arr + i). Subscripting is sugar for
    // pointer arithmetic + dereference. (Fun fact: i[arr] also works, because
    // *(arr+i) == *(i+arr). Don't write that in real code.)
    (void)base; // silence "unused" in case the reader strips lines
    // TAKE-AWAY: pointers know their pointee's size. +1 means +1 element. That
    // is the whole reason arrays and pointers feel interchangeable.

    // ========================================================================
    banner("09. STACK vs HEAP (two different neighborhoods)");
    // STACK: local variables (like everything above) live in the current
    // function's stack frame. The stack grows DOWNWARD from high addresses, so
    // locals tend to share high-looking addresses near each other.
    int stack_local = 1;

    // HEAP: when you say `new` (C++) or `malloc` (C) you explicitly ASK the
    // runtime for memory from the heap -- a separate region the allocator
    // manages. It hands back the ADDRESS of the block. The heap typically lives
    // far from the stack, so the numbers look very different.
    int* heap_int   = new int(2);          // C++ allocation
    void* malloc_blk = std::malloc(16);     // C allocation, 16 raw bytes

    show_addr("stack address (&stack_local)", &stack_local);
    show_addr("heap  address (new int)     ", heap_int);
    show_addr("heap  address (malloc 16B)  ", malloc_blk);
    // The GAP between the stack address and the heap addresses is the visible
    // proof that they are different regions of the virtual address space.
    std::printf("    |stack - heap| approx = %lld bytes  (they're far apart -> different regions)\n",
                (long long)llabs((long long)((intptr_t)&stack_local - (intptr_t)heap_int)));

    // WHEN does something "move to the heap"? ONLY when you ask, explicitly,
    // with new/malloc (or a container that does so for you). Returning the
    // address of a LOCAL is a classic BUG: the stack frame dies on return, so
    // the address dangles (see Section 10). The cure is to heap-allocate (and
    // own the lifetime) or return by value.
    delete heap_int;        // hand the int back to the allocator
    std::free(malloc_blk);  // hand the 16 bytes back
    // TAKE-AWAY: stack = automatic, scoped, free; heap = manual, you asked for
    // it, you must give it back. Different addresses because different regions.

    // ========================================================================
    banner("10. LIFETIME HAZARDS (explained; SAFE patterns demonstrated)");
    // We DESCRIBE each hazard in comments and only RUN the safe handling, so
    // this program never invokes undefined behavior / crashes.

    // (a) DANGLING via returned stack address:
    //     int* bad() { int local = 5; return &local; }  // local dies at return
    //     The returned address names a stack cell that no longer belongs to us.
    //     Reading/writing it is undefined. FIX: return by value, or heap-own it.
    std::printf("    (a) dangling: returning &local is UB; we don't do it. Return by value instead.\n");

    // (b) USE-AFTER-FREE: using a pointer after delete/free.
    int* h = new int(123);
    std::printf("    (b) allocated h at %p, *h = %d\n", (void*)h, *h);
    delete h;          // the int is returned to the allocator; *h is now invalid
    //  *h = 7;        // <-- USE-AFTER-FREE (undefined). We do NOT do this.
    h = nullptr;       // SAFE PATTERN: null it so accidental use is detectable.
    std::printf("        after delete+null: h = %p (a later deref of null is catchable, not silent corruption)\n", (void*)h);

    // (c) DOUBLE-FREE: deleting the same block twice corrupts the allocator.
    //     delete h; delete h;   // <-- UB. Because we set h=nullptr above and
    //     `delete nullptr` is a guaranteed no-op, the safe-null habit also
    //     protects against accidental double-delete.
    delete h;          // legal no-op: deleting nullptr does nothing.
    std::printf("    (c) double-free avoided: delete on a nulled pointer is a no-op\n");

    // (d) NULL DEREF: *p when p == nullptr touches address 0 -> crash/UB.
    int* maybe = nullptr;
    //  int boom = *maybe;       // <-- NULL DEREF. We guard instead:
    if (maybe == nullptr) {
        std::printf("    (d) null deref: we CHECK `if (maybe)` before *maybe -- never deref null blindly\n");
    }
    // TAKE-AWAY: raw lifetime bugs are silent and catastrophic. The cheap
    // disciplines: null-after-delete, check-before-deref, prefer return-by-value
    // -- and, better, let the type system own lifetime for you (Section 11).

    // ========================================================================
    banner("11. SMART POINTERS + RAII (let destructors free memory)");
    // RAII = Resource Acquisition Is Initialization. The idea: tie a resource's
    // lifetime to an object's lifetime. When the object goes out of scope, its
    // DESTRUCTOR runs automatically and releases the resource. No manual delete.

    std::printf("  -- unique_ptr: SOLE owner, move-only, frees on scope exit --\n");
    {
        // unique_ptr<Noisy> owns one Noisy on the heap. Watch the [ctor]/[dtor]
        // bracket the scope: construction at `make_unique`, destruction the
        // instant we leave this block -- DETERMINISTIC, no GC, no delete.
        std::unique_ptr<Noisy> u = std::make_unique<Noisy>(1);
        show_addr("    u.get() (heap object)", u.get());
        // std::unique_ptr<Noisy> u2 = u;   // <-- ILLEGAL: cannot COPY (one owner)
        std::unique_ptr<Noisy> u2 = std::move(u); // MOVE: ownership transfers
        std::printf("    after std::move: u is %s, u2 owns %p\n",
                    (u.get() == nullptr ? "empty (null)" : "non-null"), (void*)u2.get());
    } // <-- u2 leaves scope here; Noisy 1's destructor fires automatically.
    std::printf("    (left scope -> unique_ptr auto-freed the Noisy above)\n");

    std::printf("  -- shared_ptr: REFERENCE-COUNTED ownership --\n");
    {
        // shared_ptr keeps a count of how many shared_ptrs co-own the object.
        // The object is destroyed exactly when the count hits 0. We print the
        // count as owners come and go.
        std::shared_ptr<Noisy> s1 = std::make_shared<Noisy>(2);
        std::printf("    use_count after s1            = %ld\n", s1.use_count());
        {
            std::shared_ptr<Noisy> s2 = s1; // COPY allowed -> shared ownership, count++
            std::printf("    use_count after s2 = s1       = %ld\n", s1.use_count());
        } // s2 dies -> count--
        std::printf("    use_count after s2 left scope = %ld\n", s1.use_count());
    } // s1 dies -> count 0 -> Noisy 2 destroyed here
    std::printf("    (count reached 0 -> shared object freed)\n");

    std::printf("  -- weak_ptr: NON-OWNING observer, breaks reference cycles --\n");
    // THE CYCLE LEAK: if A holds shared_ptr<B> and B holds shared_ptr<A>, each
    // keeps the other's count >= 1 forever. Count never reaches 0 -> neither
    // destructor runs -> LEAK, even after both local handles vanish.
    {
        auto A = std::make_shared<CycleStrong>(1);
        auto B = std::make_shared<CycleStrong>(2);
        A->partner = B;   // A -> B (strong)
        B->partner = A;   // B -> A (strong)  == cycle
        std::printf("    strong cycle: A.use_count=%ld B.use_count=%ld (each pinned by the other)\n",
                    A.use_count(), B.use_count());
    } // A and B handles drop, but counts stay 1 because of the mutual links.
    std::printf("    NOTICE: no [CycleStrong dtor] printed above -> LEAKED (cycle never hit 0)\n");

    {
        // FIX: make one direction weak_ptr. A weak_ptr observes but does NOT
        // increment the count, so the cycle is broken and destructors run.
        auto A = std::make_shared<CycleWeak>(1);
        auto B = std::make_shared<CycleWeak>(2);
        A->partner = B;   // weak: does not bump B's count
        B->partner = A;   // weak: does not bump A's count
        std::printf("    weak cycle:   A.use_count=%ld B.use_count=%ld (weak links don't count)\n",
                    A.use_count(), B.use_count());
    } // now counts hit 0 -> both destructors fire (you'll see them print)
    std::printf("    NOTICE: both [CycleWeak dtor] printed -> cycle broken, no leak\n");
    // TAKE-AWAY: unique_ptr for single ownership (cheap, default choice);
    // shared_ptr when ownership is genuinely shared (pay for the atomic count);
    // weak_ptr for back-references/observers so you don't create immortal cycles.

    // ========================================================================
    banner("12. THE CACHE LESSON: array scan vs pointer chase (the climax)");
    // THE HARDWARE TRUTH:
    //   * RAM is SLOW relative to the CPU (hundreds of cycles per miss).
    //   * The CPU never fetches a single byte from RAM. It fetches a whole
    //     CACHE LINE -- 64 bytes on Apple silicon -- into fast on-chip cache.
    //   * 64 bytes / 8 bytes-per-long = 8 longs arrive per line "for free" once
    //     you've paid for the first. The hardware PREFETCHER also notices a
    //     forward scan and pulls the NEXT lines before you ask.
    //
    //   ARRAY/VECTOR: elements are contiguous. Walking it touches consecutive
    //   addresses -> each cache line serves ~8 elements, prefetch hides latency.
    //   Nearly every access is a cache HIT.
    //
    //   LINKED LIST: each node is a separate allocation. We deliberately build
    //   the nodes in SHUFFLED memory order so `node = node->next` jumps to an
    //   unpredictable address every hop. The prefetcher can't guess where next
    //   lives, and each node likely sits on its OWN uncached line -> a cache
    //   MISS per hop. Same number of additions; wildly different memory cost.
    //
    //   This is the textbook violation of CACHE LOCALITY: the linked list has
    //   no SPATIAL locality (next node isn't adjacent) and the access pattern
    //   defeats the prefetcher.

    const int N = 4'000'000; // big enough to blow past the L1/L2 caches
    std::printf("    cache line assumed = 64 bytes -> %zu longs per line\n", 64 / sizeof(long));
    std::printf("    N = %d elements; sizeof(Node) = %zu bytes\n", N, sizeof(Node));

    // (a) CONTIGUOUS ARRAY. One block; addresses run in lockstep.
    std::vector<long> arr_sum(N);
    std::iota(arr_sum.begin(), arr_sum.end(), 1L); // fill 1,2,3,...
    show_addr("vector front address", (void*)arr_sum.data());
    show_addr("vector back  address", (void*)(arr_sum.data() + N - 1));

    auto t0 = std::chrono::high_resolution_clock::now();
    volatile long array_total = 0; // volatile so the optimizer can't delete the loop
    for (int i = 0; i < N; ++i) {
        array_total += arr_sum[i]; // sequential -> cache-friendly
    }
    auto t1 = std::chrono::high_resolution_clock::now();

    // (b) LINKED LIST in SHUFFLED memory order. We allocate N nodes in one slab
    // (so the test is fair: same total bytes touched), then wire `next` to
    // follow a RANDOM permutation. Chasing `next` therefore leaps all over the
    // slab -- defeating spatial locality and the prefetcher, exactly the way a
    // list grown by repeated `new` over a program's lifetime would fragment.
    std::vector<Node> pool(N);
    std::vector<int> order(N);
    std::iota(order.begin(), order.end(), 0);
    std::mt19937 rng(12345);
    std::shuffle(order.begin(), order.end(), rng); // random visiting order

    for (int i = 0; i < N; ++i) {
        pool[order[i]].value = order[i] + 1; // same payload set as the array
        pool[order[i]].next  = (i + 1 < N) ? &pool[order[i + 1]] : nullptr;
    }
    Node* head = &pool[order[0]];

    auto t2 = std::chrono::high_resolution_clock::now();
    volatile long list_total = 0;
    for (Node* cur = head; cur != nullptr; cur = cur->next) {
        list_total += cur->value; // identical add; but each hop may cache-miss
    }
    auto t3 = std::chrono::high_resolution_clock::now();

    double array_us = std::chrono::duration<double, std::micro>(t1 - t0).count();
    double list_us  = std::chrono::duration<double, std::micro>(t3 - t2).count();

    std::printf("    array  sum = %ld   time = %10.1f us  (sequential, cache-friendly)\n",
                (long)array_total, array_us);
    std::printf("    list   sum = %ld   time = %10.1f us  (shuffled, cache-hostile)\n",
                (long)list_total, list_us);
    std::printf("    SAME number of additions (%d). Ratio list/array = %.1fx slower\n",
                N, (array_us > 0.0 ? list_us / array_us : 0.0));
    std::printf("    The ONLY difference is MEMORY LAYOUT. The list scatters across RAM and\n");
    std::printf("    eats a cache miss per hop; the array streams whole 64B lines + prefetch.\n");
    // (If the ratio ever looks small, the allocator happened to keep the slab
    // hot/contiguous; the principle stands -- the more fragmented the list, the
    // worse it gets. Re-run with a larger N to widen the gap.)

    // ========================================================================
    banner("MENTAL MODEL SUMMARY");
    std::printf(
        "    * Memory is numbered byte cells; an ADDRESS is a cell's number.\n"
        "    * A POINTER is an 8-byte variable whose value is such a number.\n"
        "    * &x = 'the number naming x'; *p = 'LOAD/STORE the cell p names'.\n"
        "    * & in a decl describes a TYPE; * in an expr is an ACTION.\n"
        "    * REFERENCE = an alias: same cell, no rebinding, no null, no math.\n"
        "    * POINTER   = rebindable, nullable, arithmetic (p+1 = +sizeof(T)).\n"
        "    * Arrays decay to &arr[0]; arr[i] IS *(arr+i).\n"
        "    * STACK = scoped/automatic; HEAP = you asked (new/malloc), you free.\n"
        "    * Dangling/use-after-free/double-free/null-deref are silent UB;\n"
        "      null-after-delete and check-before-deref are the cheap guards.\n"
        "    * RAII + smart pointers tie lifetime to scope: unique (sole),\n"
        "      shared (refcount), weak (observer, breaks cycles).\n"
        "    * CACHE: the CPU pulls 64B lines; arrays stream + prefetch (hits),\n"
        "      scattered linked lists miss per hop -- layout beats op-count.\n");

    std::printf("\nDone. Re-read the comments next to each section -- they ARE the lesson.\n");
    return 0;
}

// ============================================================================
//  SECTION 04 function definitions.
// ============================================================================

// BY VALUE: `copy` is a fresh variable initialized from the argument's bytes.
// Its address is DIFFERENT from the caller's n -- proof it's a separate cell.
static void take_by_value(int copy) {
    std::printf("      [by-value]     got copy=%d at %p (a DIFFERENT cell from caller's n)\n",
                copy, (void*)&copy);
    copy = 999; // edits the local copy only; caller's n is untouched
    (void)copy;
}

// BY POINTER: `addr` holds the caller's n address. *addr = ... STOREs into the
// caller's actual cell.
static void take_by_pointer(int* addr) {
    std::printf("      [by-pointer]   got addr=%p, *addr=%d (SAME cell as caller's n)\n",
                (void*)addr, *addr);
    *addr = 200; // writes through the address -> mutates caller's n
}

// BY REFERENCE: `alias` is another name for the caller's n. &alias == &n. The
// compiler implements this as a hidden pointer, but the syntax stays clean.
static void take_by_reference(int& alias) {
    std::printf("      [by-reference] &alias=%p (SAME cell as caller's n)\n", (void*)&alias);
    alias = 300; // plain assignment, but it lands in the caller's cell
}
