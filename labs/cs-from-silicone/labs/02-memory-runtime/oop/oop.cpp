// =====================================================================================
//  oop.cpp  —  OBJECT-ORIENTED PROGRAMMING, FROM THE SILICON UP
// =====================================================================================
//
//  This is a LESSON, not a library. Read it top to bottom, then run it. Every section
//  prints something you can SEE so the abstract ideas (vtables, hidden `this`, object
//  layout) become concrete bytes and addresses on your screen.
//
//  ------------------------------------------------------------------------------------
//  THE CONCEPT LADDER (we climb it for every feature, in THIS order):
//
//    1) PHYSICAL BYTES   — an object is just a run of contiguous bytes in memory.
//                          Its members sit at fixed offsets from the object's address.
//    2) CPU / ISA VIEW   — a "method call" is an ordinary function call with one extra
//                          hidden argument: a pointer to the object (`this`). A "virtual
//                          call" is an INDIRECT jump: the CPU loads a function-pointer
//                          from a table (the vtable) and jumps to whatever address is
//                          stored there.
//    3) OS / RUNTIME     — who allocates the bytes (stack vs heap), when constructors
//                          and destructors run, when memory is freed.
//    4) LANGUAGE CONCEPT — only NOW do we name it: "encapsulation", "polymorphism",
//                          "abstract class". The name is the last thing, not the first.
//
//  ------------------------------------------------------------------------------------
//  BUILD & RUN (macOS arm64, Apple clang):
//
//    clang++ -std=c++17 -O0 -g -Wall -Wextra oop.cpp -o /tmp/oop && /tmp/oop
//
//  -O0 keeps the compiler from optimizing away the layout tricks we want to observe.
//
//  ------------------------------------------------------------------------------------
//  WHAT YOU'LL BE ABLE TO ARGUE AFTER THIS:
//
//    You will be able to walk up to a hardcore Java/OOP person and explain, at the level
//    of bytes and machine instructions: that an object is just a struct of data; that a
//    method is a free function with a hidden `this` pointer; that "dynamic dispatch" is
//    literally "load a function pointer out of a per-class table and do an indirect call";
//    that adding the FIRST virtual function silently grows every object by 8 bytes (the
//    vptr); why deleting through a base pointer without a virtual destructor leaks; why
//    "prefer composition over inheritance" is about coupling, layout, and the fragile
//    base class problem — not taste. You will be able to derive these, not recite them.
// =====================================================================================

#include <cstdio>      // std::printf — we use printf so addresses/sizes print cleanly
#include <cstdint>     // uintptr_t — an integer type wide enough to hold a pointer
#include <string>      // std::string for human-readable names
#include <vector>      // std::vector for the polymorphism demo
#include <memory>      // std::unique_ptr — owning smart pointer
#include <cmath>       // M_PI etc. for shape areas

// ------------------------------------------------------------------------------------
//  Tiny helpers so every section prints a clean, labeled banner.
// ------------------------------------------------------------------------------------
static void section(const char* title) {
    std::printf("\n==== %s ====\n", title);
}

// Print an address as a plain hex number. We cast pointer -> uintptr_t (an integer the
// same width as a pointer) so printf can format it. An address is just a NUMBER: which
// byte in the process's memory we're talking about.
static unsigned long long addr(const void* p) {
    return (unsigned long long)(uintptr_t)p;
}

// =====================================================================================
//  01. FROM struct TO class  —  a class is a struct with manners
// =====================================================================================
//
//  In C, a `struct` is purely a layout: "lay these fields out next to each other in
//  memory." It has no functions and no access control — everything is public.
//
//  A C++ `class` is THE SAME THING in memory, plus two conveniences the compiler adds:
//    (a) members default to PRIVATE instead of public, and
//    (b) you may bundle functions ("methods") alongside the data.
//
//  The crucial silicon truth: a method is NOT stored inside the object. The object holds
//  only DATA. The function lives once, in the code segment, shared by all objects. When
//  you write `p.move(2,3)`, the compiler rewrites it to `Point_move(&p, 2, 3)` — it
//  passes the object's address as a hidden first argument called `this`.

struct PointStruct {       // a C-style struct: both fields public by default
    int x;
    int y;
};

class PointClass {         // identical bytes, but members are private by default
    int x;                 // <- private (class default). Outsiders cannot name `x`.
    int y;
public:
    PointClass(int x_, int y_) : x(x_), y(y_) {}

    // This method takes NO visible arguments, yet it reads x and y. How? There is a
    // hidden parameter: `this`, a `PointClass*` pointing at the object you called it on.
    // `x` is shorthand for `this->x`, i.e. "the int at (this + offset_of_x)".
    void move(int dx, int dy) {
        x += dx;
        y += dy;
    }

    // We can print `this` to prove the object's address is what the method received.
    void report(const char* name) const {
        std::printf("  %s: this=0x%llx  x=%d  y=%d\n", name, addr(this), x, y);
    }
};

static void demo_struct_to_class() {
    section("01. FROM struct TO class (the hidden `this`)");

    PointStruct ps{1, 2};
    std::printf("  C-style struct: sizeof(PointStruct)=%zu, &ps=0x%llx\n",
                sizeof(PointStruct), addr(&ps));

    PointClass pc(10, 20);
    std::printf("  C++ class:      sizeof(PointClass)=%zu, &pc=0x%llx\n",
                sizeof(PointClass), addr(&pc));

    // The address printed as `this` inside report() will EQUAL &pc below. That proves
    // the method is just a function receiving the object's address.
    std::printf("  &pc (taken from the outside)      =0x%llx\n", addr(&pc));
    pc.report("pc before move");        // pc.report(&pc, ...)
    pc.move(5, 5);                       // pc.move(&pc, 5, 5)  -> mutates via this->x/y
    pc.report("pc after  move");

    std::printf("  TAKEAWAY: same bytes as a struct; `move` is move(&pc, dx, dy).\n");
    std::printf("            `this` inside the method == &pc outside. A method is a\n");
    std::printf("            free function with one hidden pointer argument.\n");
}

// =====================================================================================
//  02. OBJECT = BYTES IN MEMORY  —  members at fixed offsets from `this`
// =====================================================================================
//
//  The compiler decides, at compile time, where each field lives RELATIVE to the start
//  of the object. Field access is pointer arithmetic: address_of_field = this + offset.
//  Offsets can include PADDING so each field is aligned (hardware reads aligned data
//  faster). Below we print each member's address and subtract the object address to
//  reveal the offsets the compiler chose.

class Mixed {
public:
    char  c;     // 1 byte
    int   i;     // 4 bytes, but the compiler will pad after `c` to align `i` to 4
    double d;    // 8 bytes, aligned to 8

    void layout() const {
        unsigned long long base = addr(this);
        std::printf("  object base (this) = 0x%llx\n", base);
        std::printf("  &c = 0x%llx  -> offset %llu\n", addr(&c), addr(&c) - base);
        std::printf("  &i = 0x%llx  -> offset %llu\n", addr(&i), addr(&i) - base);
        std::printf("  &d = 0x%llx  -> offset %llu\n", addr(&d), addr(&d) - base);
        std::printf("  sizeof(Mixed) = %zu  (includes padding for alignment)\n",
                    sizeof(Mixed));
    }
};

static void demo_object_bytes() {
    section("02. OBJECT = BYTES IN MEMORY (offsets from `this`)");
    Mixed m{};
    m.layout();
    std::printf("  TAKEAWAY: a field read is *(this + offset). The padding between c and\n");
    std::printf("            i exists so the CPU reads `i` from a 4-byte-aligned address.\n");
}

// =====================================================================================
//  03. ENCAPSULATION  —  public / private / protected (a COMPILE-TIME fence)
// =====================================================================================
//
//  Access specifiers are checked ONLY by the compiler. They do not encrypt, hide, or
//  move the bytes. A private field sits at the same predictable offset as any other
//  field — anyone with the raw address can read it. (Debater ammo: "private" is a rule
//  the compiler enforces on the source code, NOT a runtime protection. We prove it by
//  reading a private member through a raw pointer below.)
//
//    public:    anyone may access (the object's API)
//    protected: this class AND its derived classes may access (not outsiders)
//    private:   only this class (and its `friend`s) may access

class BankAccount {
private:
    long balance_cents;            // private: the API must mediate all access

protected:
    long overdraft_limit_cents;    // protected: subclasses may tweak policy

public:
    BankAccount(long initial) : balance_cents(initial), overdraft_limit_cents(0) {}

    long  getBalance() const { return balance_cents; }       // public getter
    void  deposit(long c)    { if (c > 0) balance_cents += c; } // public setter w/ a rule
};

static void demo_encapsulation() {
    section("03. ENCAPSULATION (access control is compile-time only)");
    BankAccount acct(1000);
    acct.deposit(500);
    std::printf("  via public API getBalance() = %ld cents\n", acct.getBalance());

    // The bytes are right there. `balance_cents` is the FIRST member, so it sits at
    // offset 0. We reinterpret the object's address as a `long*` and read it directly.
    // The compiler would reject `acct.balance_cents`, but the MEMORY does not care.
    long* raw = reinterpret_cast<long*>(&acct);
    std::printf("  reading the SAME bytes via raw pointer = %ld cents\n", *raw);
    std::printf("  TAKEAWAY: `private` is a source-code rule the compiler enforces.\n");
    std::printf("            The bytes are unprotected; the address exposes everything.\n");
}

// =====================================================================================
//  04. CONSTRUCTORS & DESTRUCTORS  —  the object lifecycle, printed
// =====================================================================================
//
//  A constructor runs right after the bytes are allocated; it initializes them. A
//  destructor runs right before the bytes are released; it cleans up (frees heap memory,
//  closes files, etc.). The compiler INSERTS these calls for you at scope boundaries.
//
//  ORDER RULES (memorize — they come from layout, see §05/§04):
//    * Construction: base subobjects first, then members IN DECLARATION ORDER, then the
//      constructor body.
//    * Destruction: exact REVERSE — body, then members in reverse, then bases. (Last
//      built is first torn down, like a stack.)

class Tracer {
    std::string tag;
public:
    Tracer(std::string t) : tag(std::move(t)) {                 // parameterized ctor
        std::printf("  [ctor    ] Tracer(\"%s\")\n", tag.c_str());
    }
    Tracer() : tag("default") {                                  // default ctor
        std::printf("  [ctor    ] Tracer() default\n");
    }
    Tracer(const Tracer& other) : tag(other.tag + "-copy") {     // copy ctor
        std::printf("  [copyctor] Tracer(const Tracer& \"%s\")\n", other.tag.c_str());
    }
    ~Tracer() {                                                  // destructor
        std::printf("  [dtor    ] ~Tracer(\"%s\")\n", tag.c_str());
    }
};

static void demo_ctor_dtor() {
    section("04. CONSTRUCTORS & DESTRUCTORS (lifecycle, printed)");
    std::printf("  -- entering inner scope --\n");
    {
        Tracer a("A");          // parameterized ctor
        Tracer b;               // default ctor
        Tracer c = a;           // COPY ctor (a is copied into c), NOT assignment
        (void)c;
        std::printf("  -- leaving inner scope (watch reverse-order destruction) --\n");
    } // <- destructors fire HERE, in reverse construction order: c, then b, then a
    std::printf("  TAKEAWAY: built A,B,C -> destroyed C,B,A. Lifecycle is a stack.\n");
}

// =====================================================================================
//  05. INHERITANCE ("extends")  —  the base lives INSIDE the derived object
// =====================================================================================
//
//  `Derived : public Base` says "a Derived IS-A Base". In memory, a Base subobject is
//  embedded at the FRONT of every Derived object (offset 0). The Derived's own members
//  come AFTER it. So a `Derived*` can be handed to anything expecting a `Base*` — the
//  Base part is right there at offset 0, no conversion needed.
//
//  The derived constructor's initializer list calls the base constructor explicitly
//  (`: Base(args)`). You reach a base's version of a method with `Base::method()` —
//  that is C++'s equivalent of Java's `super`.

class Animal {
protected:
    std::string name;
public:
    Animal(std::string n) : name(std::move(n)) {
        std::printf("  [ctor] Animal(\"%s\")\n", name.c_str());
    }
    void describe() const { std::printf("  Animal named %s\n", name.c_str()); }
    void breathe() const { std::printf("  %s breathes (base behavior)\n", name.c_str()); }
    ~Animal() { std::printf("  [dtor] ~Animal(\"%s\")\n", name.c_str()); }
};

class Dog : public Animal {       // Dog IS-A Animal
    int good_boy_points;
public:
    Dog(std::string n, int pts)
        : Animal(std::move(n)),    // <- call base ctor first (super)
          good_boy_points(pts) {   // <- then our own member
        std::printf("  [ctor] Dog (pts=%d)\n", good_boy_points);
    }
    void bark() const { std::printf("  %s says woof!\n", name.c_str()); }

    // Use Base::method() to reach the base version explicitly (C++'s "super").
    void breatheLikeAnimal() const { Animal::breathe(); }

    ~Dog() { std::printf("  [dtor] ~Dog\n"); }
};

static void demo_inheritance() {
    section("05. INHERITANCE (base subobject sits FIRST inside derived)");
    Dog d("Rex", 100);

    // Prove the base subobject is at offset 0: the Dog's address and its Animal-part's
    // address are the same. We upcast Dog* -> Animal*; the value doesn't change here.
    Animal* asAnimal = &d;
    std::printf("  &d (Dog*)            = 0x%llx\n", addr(&d));
    std::printf("  (Animal*)&d          = 0x%llx  (same address: base is at offset 0)\n",
                addr(asAnimal));
    std::printf("  sizeof(Animal)=%zu, sizeof(Dog)=%zu  (Dog = Animal-part + its int)\n",
                sizeof(Animal), sizeof(Dog));
    d.bark();
    d.breatheLikeAnimal();   // explicitly calls Animal::breathe — the "super" call
    std::printf("  TAKEAWAY: Dog embeds an Animal at offset 0, so Dog* IS-A Animal*\n");
    std::printf("            for free. Ctor order: Animal then Dog; dtor: Dog then Animal.\n");
}

// =====================================================================================
//  06. VIRTUAL FUNCTIONS, POLYMORPHISM, override  —  THE VTABLE (climax #1)
// =====================================================================================
//
//  Problem: with §05's non-virtual `describe()`, if a Base* points at a Derived, calling
//  base.describe() runs the BASE version — the compiler picked the function at COMPILE
//  TIME based on the pointer's static type. That is "static dispatch".
//
//  We often want the opposite: the call should run whatever the OBJECT actually is, at
//  RUN TIME. That is "dynamic dispatch", and C++ implements it with a VTABLE.
//
//  THE MECHANISM (this is the whole secret of OOP polymorphism):
//    * For each class that has any virtual function, the compiler builds ONE static
//      table of function pointers — the "vtable" — one entry per virtual function,
//      filled with the addresses of THAT class's versions.
//    * Each OBJECT of such a class gets a hidden pointer member, the "vptr", placed at
//      offset 0 (the first 8 bytes on a 64-bit machine). It points at its class's vtable.
//    * A virtual call `p->speak()` compiles to roughly:
//          vptr  = *(void**)p;        // load the hidden pointer at offset 0
//          fn    = vptr[ index_of_speak ];   // index into the table
//          fn(p);                     // INDIRECT call through that pointer
//      The CPU does an indirect jump to an address it only learns at runtime.
//
//  Cost: a virtual call is two loads + an indirect branch, and it can't be inlined (the
//  compiler doesn't know the target). A non-virtual call is a direct call the compiler
//  can often inline to nothing. That gap is the price of runtime polymorphism.
//
//  PROOF the vptr exists: two structurally identical classes, one with a virtual method,
//  one without. The virtual one is 8 bytes larger — that 8 is the vptr.

struct PlainTwin {            // NO virtual functions
    int a;
    int b;
    void f() const {}         // ordinary method: no vptr needed
};

struct VirtualTwin {          // identical data, but ONE virtual function
    int a;
    int b;
    virtual void f() const {} // <- forces a vtable; object gains a hidden vptr
    virtual ~VirtualTwin() = default;
};

// A small polymorphic hierarchy to demonstrate dynamic dispatch.
class Shape {                 // base with virtual behavior
public:
    virtual void speak() const { std::printf("  I am a generic Shape\n"); }
    virtual ~Shape() = default;   // (virtual dtor — explained fully in §07)
};
class Circle : public Shape {
public:
    void speak() const override { std::printf("  I am a Circle (override ran)\n"); }
};
class Square : public Shape {
public:
    void speak() const override { std::printf("  I am a Square (override ran)\n"); }
};

static void demo_virtual_dispatch() {
    section("04. VIRTUAL FUNCTIONS & DYNAMIC DISPATCH");

    // (1) Prove the hidden vptr by comparing sizes of the twins.
    std::printf("  sizeof(PlainTwin)   = %zu  (two ints, no vptr)\n", sizeof(PlainTwin));
    std::printf("  sizeof(VirtualTwin) = %zu  (two ints + hidden vptr + padding)\n",
                sizeof(VirtualTwin));
    std::printf("  -> adding the FIRST virtual function grew the object by %zu bytes:\n",
                sizeof(VirtualTwin) - sizeof(PlainTwin));
    std::printf("     that is the vptr (8 bytes on 64-bit) the compiler injected.\n");

    // (2) See the vptr itself: on the common Itanium/AArch64 ABI it sits at offset 0,
    // so the first 8 bytes of the object ARE the vtable pointer. We read them raw.
    Circle c;
    void** vptr_slot = reinterpret_cast<void**>(&c);
    std::printf("  &c                  = 0x%llx\n", addr(&c));
    std::printf("  *(void**)&c (vptr)  = 0x%llx  <- points at Circle's vtable\n",
                addr(*vptr_slot));

    // (3) Dynamic dispatch in action: a Shape* aimed at a Circle calls Circle::speak.
    Shape  generic;
    Circle circle;
    Square square;
    Shape* p = &generic;
    std::printf("  Shape* -> Shape:   "); p->speak();   // runs Shape::speak
    p = &circle;
    std::printf("  Shape* -> Circle:  "); p->speak();   // runs Circle::speak (!)
    p = &square;
    std::printf("  Shape* -> Square:  "); p->speak();   // runs Square::speak (!)

    // (4) Contrast: a NON-virtual call binds to the pointer's static type at compile
    // time. We demonstrate the difference with explicit qualification, which forces
    // static dispatch even on a virtual function (the compiler calls Shape::speak
    // directly, bypassing the vtable — proving the two dispatch styles coexist).
    p = &circle;
    std::printf("  STATIC dispatch p->Shape::speak() forces base: ");
    p->Shape::speak();   // direct call, NOT through the vtable

    std::printf("  TAKEAWAY: virtual call = load vptr (offset 0) -> index vtable ->\n");
    std::printf("            indirect jump. Resolved at RUNTIME, can't inline. A\n");
    std::printf("            non-virtual/qualified call is a direct, inlinable jump.\n");
}

// =====================================================================================
//  07. VIRTUAL DESTRUCTOR  —  why deleting through a Base* needs one
// =====================================================================================
//
//  When you `delete basePtr;` and basePtr actually points at a Derived, the runtime must
//  run ~Derived AND ~Base, in that order. But `delete` calls the destructor named by the
//  POINTER's type. If ~Base is NOT virtual, the call is statically bound: only ~Base
//  runs. ~Derived is skipped — any resource the Derived owned (heap memory, file handle)
//  LEAKS.
//
//  THE BROKEN CASE (do not ship this): if Base had `~Base()` non-virtual, then
//      Base* p = new Derived();  delete p;   // runs ONLY ~Base. ~Derived never fires.
//  We don't write that broken delete here (it's a real bug); instead we make the dtor
//  VIRTUAL so the vtable carries the destructor too, and `delete p` does an indirect
//  call to the CORRECT ~Derived, which then chains to ~Base. Watch both fire below.

class Resource {
public:
    Resource() { std::printf("  [ctor] Resource (base) acquired\n"); }
    virtual ~Resource() { std::printf("  [dtor] ~Resource (base) released\n"); }
    // ^ virtual: `delete Resource*` will dispatch to the real derived destructor first.
};

class FileResource : public Resource {
    int* buffer;   // pretend this owns heap memory that MUST be freed
public:
    FileResource() : buffer(new int[64]) {
        std::printf("  [ctor] FileResource allocated 64-int buffer at 0x%llx\n",
                    addr(buffer));
    }
    ~FileResource() override {
        delete[] buffer;   // if ~Resource weren't virtual, this line would be SKIPPED
        std::printf("  [dtor] ~FileResource freed its buffer (no leak)\n");
    }
};

static void demo_virtual_destructor() {
    section("07. VIRTUAL DESTRUCTOR (delete through Base* runs ~Derived)");
    Resource* p = new FileResource();   // base pointer, derived object
    std::printf("  -- calling delete through Resource* --\n");
    delete p;   // virtual dtor => ~FileResource runs (frees buffer) THEN ~Resource
    std::printf("  TAKEAWAY: with a virtual dtor, `delete Base*` dispatches to ~Derived\n");
    std::printf("            via the vtable, then chains to ~Base. Without it: leak.\n");
}

// =====================================================================================
//  08. ABSTRACT CLASSES & PURE VIRTUAL FUNCTIONS (= 0)
// =====================================================================================
//
//  A pure virtual function (`virtual T f() = 0;`) has a vtable slot but NO body to put
//  there. A class with at least one pure virtual is ABSTRACT: you cannot create an object
//  of it, because its vtable would contain a hole — there'd be nothing to call. The
//  compiler rejects `AbstractBase b;` for this reason. (We do NOT write that line; it
//  would be a compile error. This comment is the demonstration.)
//
//  A subclass that provides bodies for ALL inherited pure virtuals becomes CONCRETE and
//  can be instantiated. This is exactly C++'s notion of an "interface" + an implementer.

class Logger {                       // ABSTRACT: cannot be instantiated
public:
    virtual void log(const std::string& msg) = 0;   // pure virtual: no body, slot only
    virtual ~Logger() = default;
};

class ConsoleLogger : public Logger {                // CONCRETE: implements log()
public:
    void log(const std::string& msg) override {
        std::printf("  [console] %s\n", msg.c_str());
    }
};

static void demo_abstract() {
    section("08. ABSTRACT CLASSES & PURE VIRTUAL (= 0)");
    // Logger l;                  // <- WOULD NOT COMPILE: "variable type 'Logger' is an
    //                                  abstract class". The vtable slot for log() is empty.
    ConsoleLogger cl;             // OK: provides the missing body, so the slot is filled
    cl.log("concrete subclass can be created");
    Logger* lp = &cl;             // use it through the abstract interface type
    lp->log("called through Logger* (the interface)");
    std::printf("  TAKEAWAY: `= 0` leaves a vtable slot empty -> abstract -> no objects.\n");
    std::printf("            A subclass that fills every slot becomes concrete.\n");
}

// =====================================================================================
//  09. INTERFACES IN C++  —  all-pure-virtual abstract class + virtual dtor
// =====================================================================================
//
//  C++ has no `interface` keyword. The IDIOM is: an abstract class whose methods are ALL
//  pure virtual, plus a virtual destructor (so owners can delete through the interface
//  pointer safely — see §07). Multiple unrelated concrete classes implement it, and you
//  store them behind the interface pointer. Iterating and calling is classic RUNTIME
//  polymorphism: one loop, one virtual call, many different machine targets.

class Drawable {                                   // the "interface"
public:
    virtual void draw()  const = 0;
    virtual double area() const = 0;
    virtual ~Drawable() = default;                 // so `delete Drawable*` is safe
};

class CircleShape : public Drawable {
    double r;
public:
    explicit CircleShape(double radius) : r(radius) {}
    void   draw() const override { std::printf("  Circle  r=%.1f  area=%.2f\n", r, area()); }
    double area() const override { return M_PI * r * r; }
};

class RectShape : public Drawable {
    double w, h;
public:
    RectShape(double width, double height) : w(width), h(height) {}
    void   draw() const override { std::printf("  Rect %.1fx%.1f area=%.2f\n", w, h, area()); }
    double area() const override { return w * h; }
};

static void demo_interfaces() {
    section("09. INTERFACES (vector<unique_ptr<Drawable>> + virtual calls)");
    // unique_ptr OWNS the heap object and runs delete (via the virtual dtor) automatically
    // when the vector is destroyed. No manual delete, no leak.
    std::vector<std::unique_ptr<Drawable>> shapes;
    shapes.push_back(std::make_unique<CircleShape>(2.0));
    shapes.push_back(std::make_unique<RectShape>(3.0, 4.0));
    shapes.push_back(std::make_unique<CircleShape>(1.0));

    double total = 0.0;
    for (const auto& s : shapes) {   // ONE loop body...
        s->draw();                   // ...but each draw()/area() jumps to a DIFFERENT
        total += s->area();          //    function via that object's vtable.
    }
    std::printf("  total area = %.2f\n", total);
    std::printf("  TAKEAWAY: heterogeneous objects behind one interface pointer; the\n");
    std::printf("            same source line dispatches to different code per object.\n");
}

// =====================================================================================
//  10. COMPOSITION vs INHERITANCE  —  "has-a" vs "is-a"
// =====================================================================================
//
//  We give a Car an Engine TWO ways and compare.
//
//  INHERITANCE (is-a): `Car : public Engine` would say a Car IS-A Engine — nonsense, and
//  it welds Car to every public detail of Engine. Change Engine's interface and Car may
//  break (the "fragile base class" problem). The Engine bytes are forced into Car's
//  layout; Engine's whole public API leaks out of Car.
//
//  COMPOSITION (has-a): `Car { Engine engine; }` — a Car HAS-A Engine. Car only depends
//  on the slice of Engine it chooses to call, and re-exports nothing it doesn't want to.
//  Swapping the engine, or hiding parts of it, is a local change.
//
//  MODERN GUIDANCE: prefer composition over inheritance. WHY (real ammo):
//    * Coupling: inheritance binds you to the base's ENTIRE public/protected surface.
//    * Fragile base class: a change in the base silently alters all derived classes.
//    * Layout/ABI: the base subobject is baked into the derived layout; adding a base
//      member shifts derived offsets and can break binary compatibility.
//    * "is-a" is rare; "has-a"/"uses-a" is the common, honest relationship.
//  Use inheritance for genuine substitutability (Liskov: a Derived must work ANYWHERE a
//  Base is expected) and to share a virtual interface — not merely to reuse code.

class Engine {
public:
    void start() const { std::printf("  Engine: vroom\n"); }
};

// COMPOSITION: Car HAS-A Engine. Car decides exactly what to expose.
class CarComposed {
    Engine engine;                 // a member: an Engine lives INSIDE this Car
public:
    void start() const {
        std::printf("  CarComposed starting (delegates to its Engine member):\n");
        engine.start();            // explicit delegation — Car controls the surface
    }
};

// INHERITANCE: Car IS-A Engine (modeling smell). Engine's API leaks straight onto Car.
class CarInherited : public Engine {
public:
    void honk() const { std::printf("  CarInherited: beep (and it also IS-A Engine)\n"); }
};

static void demo_composition_vs_inheritance() {
    section("10. COMPOSITION vs INHERITANCE (has-a vs is-a)");
    CarComposed cc;
    cc.start();

    CarInherited ci;
    ci.honk();
    ci.start();   // works, but only because Car "is" an Engine — a leaky model
    std::printf("  sizeof(Engine)=%zu  sizeof(CarComposed)=%zu  sizeof(CarInherited)=%zu\n",
                sizeof(Engine), sizeof(CarComposed), sizeof(CarInherited));
    std::printf("  TAKEAWAY: composition exposes only what you delegate and decouples you\n");
    std::printf("            from the base's full surface; prefer it unless you truly\n");
    std::printf("            need substitutability (Liskov) or a shared virtual interface.\n");
}

// =====================================================================================
//  11. MULTIPLE INHERITANCE, THE DIAMOND, & VIRTUAL INHERITANCE
// =====================================================================================
//
//  C++ lets a class inherit from MORE THAN ONE base. That creates the "diamond":
//
//          A
//         / \         B and C each inherit A.  D inherits both B and C.
//        B   C        Question: how many A subobjects does a D contain?
//         \ /
//          D
//
//  NON-virtual inheritance: D physically contains B (which contains an A) AND C (which
//  contains ANOTHER A). So D has TWO complete A subobjects — A's data is DUPLICATED, and
//  naming `d.a_value` is AMBIGUOUS (which A?). You'd have to write d.B::a_value.
//
//  VIRTUAL inheritance (`class B : virtual public A`): B and C declare they're willing to
//  SHARE one A. Now D contains exactly ONE A subobject, reached through hidden offset
//  pointers the compiler adds. Cost: extra indirection and a bigger object, but a single
//  unambiguous A.
//
//  Java/Go DODGE this: a class may extend only one class (Java) or has no class
//  inheritance at all (Go); both allow multiple INTERFACES (pure behavior, no data), so
//  there's no duplicated state to fight over. C++ pays the complexity to allow data
//  inheritance from multiple bases.
//
//  Below: a non-virtual diamond (two A's) and a virtual diamond (one A). We avoid writing
//  the ambiguous access; the sizes alone tell the story.

struct A_base {
    long a_value = 0;   // 8 bytes of state we can count
};

// ---- Non-virtual diamond: D_dup ends up with TWO A_base subobjects ----
struct B_dup : public A_base { long b_value = 0; };
struct C_dup : public A_base { long c_value = 0; };
struct D_dup : public B_dup, public C_dup {
    long d_value = 0;
    // d.a_value here is AMBIGUOUS (B's A or C's A?). To use it you must disambiguate:
    void setBoth(long x) { B_dup::a_value = x; C_dup::a_value = x; }   // two distinct A's!
};

// ---- Virtual diamond: D_shared has exactly ONE shared A_base ----
struct B_v : virtual public A_base { long b_value = 0; };
struct C_v : virtual public A_base { long c_value = 0; };
struct D_shared : public B_v, public C_v {
    long d_value = 0;
    // Now a_value is unambiguous: there is only one A. No disambiguation needed.
    void setA(long x) { a_value = x; }
};

static void demo_diamond() {
    section("11. MULTIPLE INHERITANCE, DIAMOND, & VIRTUAL INHERITANCE");
    std::printf("  sizeof(A_base) = %zu\n", sizeof(A_base));

    D_dup dd{};
    dd.setBoth(7);   // sets TWO separate a_value fields
    std::printf("  NON-virtual diamond: sizeof(D_dup)    = %zu\n", sizeof(D_dup));
    std::printf("    contains B's A AND C's A: B::a_value=%ld  C::a_value=%ld\n",
                dd.B_dup::a_value, dd.C_dup::a_value);
    std::printf("    -> A_base's bytes are DUPLICATED; plain `d.a_value` is ambiguous.\n");

    D_shared ds{};
    ds.setA(9);
    std::printf("  VIRTUAL diamond:     sizeof(D_shared) = %zu\n", sizeof(D_shared));
    std::printf("    ONE shared A: a_value=%ld (unambiguous). Larger due to hidden\n",
                ds.a_value);
    std::printf("    base-offset pointers the compiler adds to locate the shared A.\n");
    std::printf("  TAKEAWAY: non-virtual => duplicated base + ambiguity; `virtual public`\n");
    std::printf("            => one shared base via indirection. Java/Go sidestep this by\n");
    std::printf("            allowing only single class inheritance + stateless interfaces.\n");
}

// =====================================================================================
//  12. STATIC vs DYNAMIC, AND THE OBJECT-LAYOUT RECAP TABLE
// =====================================================================================
//
//  Static dispatch  : target chosen at COMPILE time from the static type. Direct call,
//                      inlinable, zero runtime overhead. (Non-virtual methods; calls made
//                      on a concrete object; Base::f() qualified calls.)
//  Dynamic dispatch : target chosen at RUN time from the object's vptr->vtable. Indirect
//                      call, not inlinable, one extra load + an indirect branch. (Virtual
//                      calls through a base pointer/reference.)
//
//  The recap table reprints the sizeof story so the silicon lands: a plain struct, the
//  same struct +1 virtual (gains a vptr), and an inheritance case.

// (public fields here only to keep -Wextra quiet; these classes exist solely so we can
//  print their sizeof. The bytes are what we're teaching, not the access level.)
struct NoVirt   { int x; int y; void f() {} };               // no vptr
struct OneVirt  { int x; int y; virtual void f() {} virtual ~OneVirt() = default; };
struct DerivedV : public OneVirt { int z; };                 // reuses base's vptr slot

static void demo_recap() {
    section("12. STATIC vs DYNAMIC  +  OBJECT-LAYOUT RECAP TABLE");
    std::printf("  +-------------------------------+--------+----------------------------+\n");
    std::printf("  | class                         | sizeof | what the bytes are         |\n");
    std::printf("  +-------------------------------+--------+----------------------------+\n");
    std::printf("  | NoVirt   (2 ints, no virtual) | %5zu  | x,y only                   |\n",
                sizeof(NoVirt));
    std::printf("  | OneVirt  (2 ints + 1 virtual) | %5zu  | vptr + x,y (+padding)      |\n",
                sizeof(OneVirt));
    std::printf("  | DerivedV (OneVirt + 1 int)    | %5zu  | reuses base vptr + x,y,z   |\n",
                sizeof(DerivedV));
    std::printf("  +-------------------------------+--------+----------------------------+\n");
    std::printf("  Note: OneVirt is %zu bytes bigger than NoVirt -> that delta is the vptr.\n",
                sizeof(OneVirt) - sizeof(NoVirt));
    std::printf("  Note: DerivedV does NOT add a second vptr; it shares the base's.\n");
}

// =====================================================================================
//  MAIN  —  run the lesson top to bottom
// =====================================================================================
int main() {
    std::printf("################################################################\n");
    std::printf("#  OOP FROM THE SILICON UP  —  run order = teaching order      #\n");
    std::printf("################################################################\n");

    demo_struct_to_class();           // 01
    demo_object_bytes();              // 02
    demo_encapsulation();             // 03
    demo_ctor_dtor();                 // 04
    demo_inheritance();               // 05
    demo_virtual_dispatch();          // 06 (printed as "04. VIRTUAL..." per spec)
    demo_virtual_destructor();        // 07
    demo_abstract();                  // 08
    demo_interfaces();                // 09
    demo_composition_vs_inheritance();// 10
    demo_diamond();                   // 11
    demo_recap();                     // 12

    // -------------------------------------------------------------------------------
    //  MENTAL-MODEL SUMMARY — the things to remember (and to argue with)
    // -------------------------------------------------------------------------------
    section("MENTAL MODEL — what to carry away");
    std::printf("  1. An object IS its data bytes. Methods are NOT stored in the object;\n");
    std::printf("     they live once in code. obj.f() == f(&obj): the hidden `this`.\n");
    std::printf("  2. Members sit at fixed compile-time offsets from `this`; field access\n");
    std::printf("     is *(this + offset). Padding aligns fields for the hardware.\n");
    std::printf("  3. public/private/protected are COMPILE-TIME rules. The bytes are still\n");
    std::printf("     right there at known offsets — `private` is not runtime protection.\n");
    std::printf("  4. Construction: bases, then members in declaration order, then body.\n");
    std::printf("     Destruction is the exact reverse. The lifecycle is a stack.\n");
    std::printf("  5. Inheritance embeds the base subobject at offset 0, so Derived* IS-A\n");
    std::printf("     Base* for free. Base::f() is C++'s `super`.\n");
    std::printf("  6. VIRTUAL = vptr + vtable. A virtual call = load vptr (offset 0) ->\n");
    std::printf("     index the vtable -> INDIRECT jump. Decided at runtime; can't inline.\n");
    std::printf("     The first virtual function grows every object by one vptr (8 bytes).\n");
    std::printf("  7. Delete through a Base* needs a VIRTUAL destructor, or only ~Base\n");
    std::printf("     runs and the Derived's resources leak.\n");
    std::printf("  8. Pure virtual (=0) leaves a vtable slot empty -> abstract -> no\n");
    std::printf("     instances. A C++ `interface` = all-pure-virtual class + virtual dtor.\n");
    std::printf("  9. PREFER COMPOSITION (has-a) over inheritance (is-a): less coupling,\n");
    std::printf("     no fragile base class, cleaner layout/ABI. Inherit only for genuine\n");
    std::printf("     substitutability or a shared virtual interface.\n");
    std::printf(" 10. The diamond duplicates the base unless you use `virtual` inheritance;\n");
    std::printf("     Java/Go avoid it via single inheritance + stateless interfaces.\n");

    std::printf("\n(end of lesson — re-run any time; sizes/addresses vary per run/host)\n");
    return 0;
}
