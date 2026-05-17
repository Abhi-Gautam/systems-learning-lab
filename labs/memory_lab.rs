use std::env;
use std::fmt::Write as _;
use std::hint::black_box;
use std::mem::{align_of, size_of};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    RunDay(u8),
    RunAll,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ArenaBlock {
    offset: usize,
    size: usize,
    addr: usize,
}

#[derive(Debug)]
struct BumpArena {
    bytes: Vec<u8>,
    cursor: usize,
}

impl BumpArena {
    fn new(capacity: usize) -> Self {
        Self {
            bytes: vec![0; capacity],
            cursor: 0,
        }
    }

    fn alloc(&mut self, size: usize, align: usize) -> Option<ArenaBlock> {
        let align = align.max(1);
        let start = align_up(self.cursor, align);
        let end = start.checked_add(size)?;
        if end > self.bytes.len() {
            return None;
        }
        self.cursor = end;
        Some(ArenaBlock {
            offset: start,
            size,
            addr: self.bytes.as_ptr() as usize + start,
        })
    }

    fn reset(&mut self) {
        self.cursor = 0;
    }

    fn used(&self) -> usize {
        self.cursor
    }

    fn capacity(&self) -> usize {
        self.bytes.len()
    }
}

#[derive(Debug)]
struct Node {
    value: u64,
    next: Option<Box<Node>>,
}

#[derive(Debug)]
struct PackedRecord {
    tag: u8,
    id: u64,
    flag: bool,
}

#[derive(Debug)]
struct ReorderedRecord {
    id: u64,
    tag: u8,
    flag: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TinyRam {
    cells: Vec<u8>,
}

impl TinyRam {
    fn new(size: usize) -> Self {
        Self {
            cells: vec![0; size],
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            cells: bytes.to_vec(),
        }
    }

    fn load(&self, address: usize) -> Option<u8> {
        self.cells.get(address).copied()
    }

    fn store(&mut self, address: usize, value: u8) -> Option<()> {
        let cell = self.cells.get_mut(address)?;
        *cell = value;
        Some(())
    }

    fn dump(&self) -> String {
        dump_bytes(&self.cells)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TinyCpu {
    registers: [u8; 3],
    ram: TinyRam,
}

impl TinyCpu {
    fn new(ram: TinyRam) -> Self {
        Self {
            registers: [0; 3],
            ram,
        }
    }

    fn load(&mut self, register: usize, address: usize) -> Option<()> {
        let value = self.ram.load(address)?;
        *self.registers.get_mut(register)? = value;
        Some(())
    }

    fn add(&mut self, dest: usize, left: usize, right: usize) {
        self.registers[dest] = self.registers[left].wrapping_add(self.registers[right]);
    }

    fn store(&mut self, register: usize, address: usize) -> Option<()> {
        self.ram.store(address, self.registers[register])
    }
}

fn main() {
    match parse_command(env::args()) {
        Command::RunDay(day) => run_day(day),
        Command::RunAll => {
            for day in 1..=30 {
                run_day(day);
            }
        }
        Command::Help => print_help(),
    }
}

fn parse_command<I, S>(args: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut iter = args.into_iter();
    let _program = iter.next();
    let Some(arg) = iter.next() else {
        return Command::Help;
    };
    let arg = arg.as_ref();
    if arg == "all" {
        return Command::RunAll;
    }
    let Some(rest) = arg.strip_prefix("day") else {
        return Command::Help;
    };
    match rest.parse::<u8>() {
        Ok(day @ 1..=30) => Command::RunDay(day),
        _ => Command::Help,
    }
}

fn print_help() {
    println!("memory_lab: heap, stack, cache, virtual memory, and allocator labs");
    println!();
    println!("Build:");
    println!("  rustc labs/memory_lab.rs -O -o /tmp/memory_lab");
    println!();
    println!("Run:");
    println!("  /tmp/memory_lab day01");
    println!("  /tmp/memory_lab day12");
    println!("  /tmp/memory_lab all");
    println!();
    println!("Notes:");
    println!("  macOS runs the core labs.");
    println!("  Raspberry Pi/Linux unlocks /proc maps, RSS, page faults, and perf-friendly labs.");
}

fn run_day(day: u8) {
    match day {
        1 => day01_process_memory_map(),
        2 => day02_stack_frames(),
        3 => day03_heap_basics(),
        4 => day04_static_storage(),
        5 => day05_c_contrast_basics(),
        6 => day06_layout_padding(),
        7 => day07_moves_and_copies(),
        8 => day08_references_and_raw_pointers(),
        9 => day09_slices_and_fat_pointers(),
        10 => day10_lifetimes_access_discipline(),
        11 => day11_memory_hierarchy(),
        12 => day12_cache_lines_and_strides(),
        13 => day13_temporal_locality(),
        14 => day14_pointer_chasing(),
        15 => day15_c_contrast_locality(),
        16 => day16_virtual_addresses(),
        17 => day17_pages_and_page_size(),
        18 => day18_demand_paging(),
        19 => day19_tlb_pressure(),
        20 => day20_mmap(),
        21 => day21_allocator_patterns(),
        22 => day22_drop_vs_return_to_os(),
        23 => day23_fragmentation(),
        24 => day24_arena_model(),
        25 => day25_pool_allocator(),
        26 => day26_kernel_stack_context(),
        27 => day27_pinned_dma_page_cache(),
        28 => day28_false_sharing(),
        29 => day29_allocator_case_study(),
        30 => day30_capstone(),
        _ => print_help(),
    }
}

fn section(day: u8, title: &str) {
    println!();
    println!("================================================================");
    println!("Day {:02}: {}", day, title);
    println!("================================================================");
}

fn concept(text: &str) {
    println!();
    println!("Concept:");
    println!("{}", text);
}

fn observe(text: &str) {
    println!();
    println!("Observe:");
    println!("{}", text);
}

fn reading(text: &str) {
    println!();
    println!("Read next:");
    println!("{}", text);
}

fn c_contrast(text: &str) {
    println!();
    println!("C contrast prompt:");
    println!("{}", text);
}

fn layout_report<T>(name: &str, value: &T) -> String {
    format!(
        "{:<28} addr=0x{:016x} size={} align={}",
        name,
        value as *const T as usize,
        size_of::<T>(),
        align_of::<T>()
    )
}

fn format_bits_u8(value: u8) -> String {
    let raw = format!("{:08b}", value);
    format!("{}_{}", &raw[..4], &raw[4..])
}

fn dump_bytes(bytes: &[u8]) -> String {
    let mut out = String::from("address  hex   bits\n");
    for (address, byte) in bytes.iter().enumerate() {
        let _ = writeln!(
            out,
            "0x{:02x}    0x{:02x}  {}",
            address,
            byte,
            format_bits_u8(*byte)
        );
    }
    out
}

fn align_up(value: usize, align: usize) -> usize {
    let align = align.max(1);
    let rem = value % align;
    if rem == 0 {
        value
    } else {
        value + (align - rem)
    }
}

fn stride_indexes(len: usize, stride: usize) -> Vec<usize> {
    if stride == 0 {
        return Vec::new();
    }
    (0..len).step_by(stride).collect()
}

fn page_size() -> usize {
    #[cfg(unix)]
    {
        #[cfg(target_os = "macos")]
        const SC_PAGESIZE: i32 = 29;
        #[cfg(target_os = "linux")]
        const SC_PAGESIZE: i32 = 30;
        unsafe extern "C" {
            fn sysconf(name: i32) -> isize;
        }
        let size = unsafe { sysconf(SC_PAGESIZE) };
        if size > 0 {
            return size as usize;
        }
    }
    4096
}

fn current_rss_kb() -> Option<usize> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("VmRSS:") {
                return rest
                    .split_whitespace()
                    .next()
                    .and_then(|n| n.parse::<usize>().ok());
            }
        }
        None
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

fn print_linux_memory_map_hint() {
    #[cfg(target_os = "linux")]
    {
        match std::fs::read_to_string("/proc/self/maps") {
            Ok(maps) => {
                println!("First 12 lines of /proc/self/maps:");
                for line in maps.lines().take(12) {
                    println!("  {}", line);
                }
            }
            Err(err) => println!("Could not read /proc/self/maps: {}", err),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        println!("Linux-only: run on Raspberry Pi and inspect /proc/self/maps.");
        println!("macOS equivalent to explore manually: vmmap <pid>");
    }
}

fn time_it<F>(label: &str, mut f: F) -> u128
where
    F: FnMut() -> u64,
{
    let start = Instant::now();
    let value = f();
    let elapsed = start.elapsed().as_micros();
    println!("{:<34} {:>10} us  checksum={}", label, elapsed, value);
    elapsed
}

fn make_shuffled_indexes(len: usize) -> Vec<usize> {
    let mut indexes: Vec<usize> = (0..len).collect();
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    for i in (1..len).rev() {
        state ^= state << 7;
        state ^= state >> 9;
        let j = (state as usize) % (i + 1);
        indexes.swap(i, j);
    }
    indexes
}

fn sequential_sum(data: &[u64]) -> u64 {
    let mut sum = 0_u64;
    for value in data {
        sum = sum.wrapping_add(black_box(*value));
    }
    sum
}

fn strided_sum(data: &[u64], stride: usize) -> u64 {
    let mut sum = 0_u64;
    for i in stride_indexes(data.len(), stride) {
        sum = sum.wrapping_add(black_box(data[i]));
    }
    sum
}

fn strided_sum_fixed_visits(data: &[u64], stride: usize, visits: usize) -> u64 {
    if data.is_empty() || stride == 0 {
        return 0;
    }
    let mut sum = 0_u64;
    let mut index = 0_usize;
    for _ in 0..visits {
        sum = sum.wrapping_add(black_box(data[index]));
        index = (index + stride) % data.len();
    }
    sum
}

fn random_index_sum(data: &[u64], indexes: &[usize]) -> u64 {
    let mut sum = 0_u64;
    for &i in indexes {
        sum = sum.wrapping_add(black_box(data[i]));
    }
    sum
}

fn build_list(len: usize) -> Option<Box<Node>> {
    let mut head = None;
    for value in (0..len as u64).rev() {
        head = Some(Box::new(Node { value, next: head }));
    }
    head
}

fn list_sum(head: &Option<Box<Node>>) -> u64 {
    let mut sum = 0_u64;
    let mut current = head.as_ref();
    while let Some(node) = current {
        sum = sum.wrapping_add(black_box(node.value));
        current = node.next.as_ref();
    }
    sum
}

fn recursive_stack_probe(depth: usize, max: usize, base: usize) {
    let local = depth as u64;
    let addr = &local as *const u64 as usize;
    if depth == 0 {
        println!("depth {:>2}: local addr=0x{:016x}", depth, addr);
    } else {
        let direction = if addr < base { "down" } else { "up" };
        println!(
            "depth {:>2}: local addr=0x{:016x} stack moved {} by {} bytes",
            depth,
            addr,
            direction,
            base.abs_diff(addr)
        );
    }
    if depth < max {
        recursive_stack_probe(depth + 1, max, base);
    }
    black_box(local);
}

fn day01_process_memory_map() {
    section(1, "Bits, bytes, words, and addressable storage");
    concept(
        "Before stack, heap, or operating systems, memory is addressable storage. A bit stores one 0/1 signal, \
         a byte is 8 bits, and an address is the number used to select a storage location.",
    );

    let byte = 0b1010_1100_u8;
    println!("one byte value: 0x{:02x}", byte);
    println!("bit positions:  7 6 5 4 3 2 1 0");
    println!("bit values:     {}", format_bits_u8(byte).replace('_', " "));
    println!();
    println!("Common word sizes:");
    println!(
        "u8  = {} byte  = {} bits",
        size_of::<u8>(),
        size_of::<u8>() * 8
    );
    println!(
        "u16 = {} bytes = {} bits",
        size_of::<u16>(),
        size_of::<u16>() * 8
    );
    println!(
        "u32 = {} bytes = {} bits",
        size_of::<u32>(),
        size_of::<u32>() * 8
    );
    println!(
        "u64 = {} bytes = {} bits",
        size_of::<u64>(),
        size_of::<u64>() * 8
    );

    let ram = [0x12_u8, 0x34, 0x56, 0x78];
    println!();
    println!("Tiny addressable storage:");
    print!("{}", dump_bytes(&ram));

    observe(
        "Do not think about Rust variables yet. Think like a memory chip: address 0 selects one byte, \
         address 1 selects the next byte, and a word is several neighboring bytes interpreted together.",
    );
    reading("N2T: registers/RAM chips. COD: data representation basics.");
}

fn day02_stack_frames() {
    section(2, "Registers vs memory");
    concept(
        "The ALU works on tiny fast storage called registers. Larger memory stores many values, but load/store \
         operations must move values between memory cells and registers before arithmetic happens.",
    );
    let mut cpu = TinyCpu::new(TinyRam::from_bytes(&[7, 5, 0, 0]));
    println!("Initial RAM:");
    print!("{}", cpu.ram.dump());
    println!(
        "Initial registers: R0={} R1={} R2={}",
        cpu.registers[0], cpu.registers[1], cpu.registers[2]
    );

    cpu.load(0, 0).unwrap();
    println!("LOAD  R0, [0]     -> R0={}", cpu.registers[0]);
    cpu.load(1, 1).unwrap();
    println!("LOAD  R1, [1]     -> R1={}", cpu.registers[1]);
    cpu.add(2, 0, 1);
    println!("ADD   R2, R0, R1  -> R2={}", cpu.registers[2]);
    cpu.store(2, 2).unwrap();
    println!("STORE [2], R2     -> RAM[2]={}", cpu.ram.load(2).unwrap());
    println!();
    println!("Final RAM:");
    print!("{}", cpu.ram.dump());

    observe(
        "Memory kept the values. Registers fed the ALU. Store wrote the result back to memory.",
    );
    reading("N2T: A register, D register, RAM. COD: datapath/register file/load-store idea.");
}

fn day03_heap_basics() {
    section(3, "SRAM, DRAM, and why memory has levels");
    concept(
        "There is no single physical thing called memory. Registers are tiny and fastest. SRAM is fast and costly, \
         so it becomes cache. DRAM is denser and slower, so it becomes main memory. The hierarchy is physics plus economics.",
    );
    println!("Hierarchy intuition:");
    println!("registers  : flip-flops, per-core, fastest, tiny");
    println!("L1/L2 cache: SRAM, on chip, fast, expensive per bit");
    println!("DRAM       : capacitor cells, off core/package, dense, slower, refreshed");
    println!("SSD/disk   : persistent storage, much slower, not CPU load/store memory");

    let small: Vec<u64> = (0..8_192).collect();
    let large: Vec<u64> = (0..4_000_000).collect();
    time_it("small working set repeated 200x", || {
        let mut sum = 0;
        for _ in 0..200 {
            sum ^= sequential_sum(&small);
        }
        sum
    });
    time_it("large working set streamed 3x", || {
        let mut sum = 0;
        for _ in 0..3 {
            sum ^= sequential_sum(&large);
        }
        sum
    });
    observe(
        "The timing is only a symptom. The cause is that faster storage costs more area and power per bit, \
         so machines use several smaller/faster levels in front of larger/slower levels.",
    );
    reading("COD Ch.5 memory hierarchy. Existing COD notes on hierarchy.");
}

fn day04_static_storage() {
    section(4, "Cache lines and locality");
    concept(
        "A program can ask for one byte or word, but the hardware usually moves a whole cache line between memory \
         levels. Nearby data gets cheaper because it rode along in the same line.",
    );
    println!("Mental model for a 64-byte line:");
    println!("ask for address 100 -> hardware may fetch addresses 64..127 into cache");
    println!("then address 101, 102, ... are likely already nearby");

    let data: Vec<u64> = (0..2_000_000).collect();
    let visits = 1_000_000;
    for stride in [1, 2, 4, 8, 16, 64, 512] {
        time_it(&format!("stride {:>3}, fixed loads", stride), || {
            strided_sum_fixed_visits(&data, stride, visits)
        });
    }
    observe(
        "Stride 1 consumes neighboring values. Large strides throw away most of each fetched cache line. \
         This is the first reason layout matters before we ever say heap or stack.",
    );
    reading("COD Ch.5 cache blocks/lines. DBI notes on row vs column locality.");
}

fn day05_c_contrast_basics() {
    section(5, "Address, data, and control signals: fake RAM chip");
    concept(
        "A memory operation is a protocol. Address lines select which cell, a control signal says read or write, \
         and data lines carry the byte being returned or stored.",
    );

    let mut ram = TinyRam::new(16);
    let address = 3;
    let value = 0xaa;
    println!("WRITE address={} data=0x{:02x}", address, value);
    println!("Address lines: {:04b}", address);
    println!("Data lines:    {}", format_bits_u8(value));
    println!("Control:       WRITE");
    ram.store(address, value).unwrap();

    println!();
    println!("READ  address={}", address);
    println!("Address lines: {:04b}", address);
    println!("Control:       READ");
    let read_back = ram.load(address).unwrap();
    println!(
        "Data lines:    {} = 0x{:02x}",
        format_bits_u8(read_back),
        read_back
    );
    println!();
    println!("RAM dump:");
    print!("{}", ram.dump());

    observe(
        "This fake chip is not OS memory yet. It is the lower abstraction: numbered cells plus a read/write protocol.",
    );
    reading("N2T RAM/register chapters. COD memory/datapath basics.");
}

fn day06_layout_padding() {
    section(6, "Value layout, alignment, and padding");
    concept(
        "CPUs prefer naturally aligned loads. Compilers insert padding so fields start at addresses the target \
         can load efficiently.",
    );
    let packed = PackedRecord {
        tag: 1,
        id: 42,
        flag: true,
    };
    let reordered = ReorderedRecord {
        id: 42,
        tag: 1,
        flag: true,
    };
    println!("{}", layout_report("PackedRecord value", &packed));
    println!("{}", layout_report("ReorderedRecord value", &reordered));
    println!(
        "PackedRecord fields: tag={}, id={}, flag={}",
        packed.tag, packed.id, packed.flag
    );
    println!(
        "ReorderedRecord fields: id={}, tag={}, flag={}",
        reordered.id, reordered.tag, reordered.flag
    );
    println!("size_of::<PackedRecord>() = {}", size_of::<PackedRecord>());
    println!(
        "size_of::<ReorderedRecord>() = {}",
        size_of::<ReorderedRecord>()
    );
    observe("Rust's default repr is not a stable C ABI. Use repr(C) when C layout is part of the contract.");
    reading("COD Ch.2: data representation and alignment.");
}

fn day07_moves_and_copies() {
    section(
        7,
        "Moves: owner metadata moves, heap payload usually does not",
    );
    let vec = vec![10_u64, 20, 30];
    let before_buffer = vec.as_ptr() as usize;
    println!("{}", layout_report("vec owner before move", &vec));
    println!("buffer before move=0x{:016x}", before_buffer);
    let moved = vec;
    println!("{}", layout_report("vec owner after move", &moved));
    println!("buffer after move =0x{:016x}", moved.as_ptr() as usize);
    observe("The stack owner changed location. The heap buffer pointer stayed the same.");

    let copied = 123_u64;
    let copied2 = copied;
    println!("{}", layout_report("copied u64 original", &copied));
    println!("{}", layout_report("copied u64 copy", &copied2));
    reading("Rust Book: move vs Copy. COD: memory operands vs register values.");
}

fn day08_references_and_raw_pointers() {
    section(8, "References and raw pointers");
    let mut value = 55_u64;
    let shared = &value;
    println!("{}", layout_report("&value binding", &shared));
    println!(
        "shared points to 0x{:016x}",
        *&shared as *const u64 as usize
    );
    let raw = &mut value as *mut u64;
    println!("raw mutable pointer=0x{:016x}", raw as usize);
    unsafe {
        *raw += 1;
        println!("unsafe dereference read value={}", *raw);
    }
    observe("The unsafe block is small on purpose: raw pointers are allowed to exist, dereferencing is the dangerous act.");
    reading("Rustonomicon: references, aliasing, raw pointers.");
}

fn day09_slices_and_fat_pointers() {
    section(9, "Slices and fat pointers");
    let array = [1_u32, 2, 3, 4, 5, 6];
    let slice = &array[1..5];
    let text = "hello slice";
    println!("{}", layout_report("array", &array));
    println!("{}", layout_report("slice reference", &slice));
    println!(
        "slice data ptr=0x{:016x} len={} first={}",
        slice.as_ptr() as usize,
        slice.len(),
        slice[0]
    );
    println!(
        "&str data ptr=0x{:016x} len={} text={}",
        text.as_ptr() as usize,
        text.len(),
        text
    );
    observe("A slice is pointer plus length. The elements are somewhere else.");
    reading("COD Ch.2: base+offset addressing. Rust Book: slices.");
}

fn day10_lifetimes_access_discipline() {
    section(10, "Lifetimes: access discipline, not storage location");
    let stack_value = 77_u64;
    let heap_value = Box::new(88_u64);
    let stack_ref = &stack_value;
    let heap_ref = &*heap_value;
    println!(
        "stack_ref points to 0x{:016x}",
        stack_ref as *const u64 as usize
    );
    println!(
        "heap_ref  points to 0x{:016x}",
        heap_ref as *const u64 as usize
    );
    observe(
        "Both are references with lifetimes. One pointee is stack storage, one is heap storage. \
         Lifetimes describe how long access is valid; they are not an address-space label.",
    );
    reading("Rust Book: lifetimes. OSTEP: address spaces for the separate OS-level meaning of lifetime.");
}

fn day11_memory_hierarchy() {
    section(11, "Memory hierarchy and working-set cliffs");
    let sizes = [4 * 1024, 64 * 1024, 1024 * 1024, 8 * 1024 * 1024];
    for bytes in sizes {
        let len = bytes / size_of::<u64>();
        let data: Vec<u64> = (0..len as u64).collect();
        time_it(&format!("sequential scan {:>8} bytes", bytes), || {
            sequential_sum(&data)
        });
    }
    observe("When the working set stops fitting a cache level, time usually jumps.");
    reading("COD Ch.5: memory hierarchy and locality.");
}

fn day12_cache_lines_and_strides() {
    section(12, "Cache lines, stride, and spatial locality");
    let data: Vec<u64> = (0..2_000_000).collect();
    for stride in [1, 2, 4, 8, 16, 64, 512] {
        time_it(&format!("stride {:>3}", stride), || {
            strided_sum(&data, stride)
        });
    }
    observe(
        "A typical cache line is 64 bytes. With u64 values, stride 1 consumes every loaded value; \
         stride 8 often uses only one value per line.",
    );
    reading("COD Ch.5: cache blocks/lines and spatial locality.");
}

fn day13_temporal_locality() {
    section(13, "Temporal locality and reuse distance");
    let small: Vec<u64> = (0..8_192).collect();
    let large: Vec<u64> = (0..4_000_000).collect();
    time_it("small array repeated 200x", || {
        let mut sum = 0;
        for _ in 0..200 {
            sum ^= sequential_sum(&small);
        }
        sum
    });
    time_it("large array repeated 3x", || {
        let mut sum = 0;
        for _ in 0..3 {
            sum ^= sequential_sum(&large);
        }
        sum
    });
    observe(
        "Temporal locality means reuse happens before the data is evicted from the relevant cache.",
    );
    reading("COD Ch.5: temporal locality.");
}

fn day14_pointer_chasing() {
    section(14, "Pointer chasing vs contiguous arrays");
    let len = 200_000;
    let data: Vec<u64> = (0..len as u64).collect();
    let list = build_list(len);
    let indexes = make_shuffled_indexes(len);
    time_it("contiguous Vec scan", || sequential_sum(&data));
    time_it("random index Vec scan", || {
        random_index_sum(&data, &indexes)
    });
    time_it("Box linked-list scan", || list_sum(&list));
    observe(
        "Pointer-rich structures are flexible, but they fight caches and hardware prefetchers.",
    );
    reading(
        "DBI: in-memory pointer-rich formats. CLRS: data structures with memory-layout awareness.",
    );
}

fn day15_c_contrast_locality() {
    section(15, "C contrast: arrays and linked lists");
    c_contrast(
        "Write a C benchmark that sums a malloc'd uint64_t array and a malloc'd linked list of the same length. \
         Compile with -O2. Compare timings with day14.",
    );
    reading("COD Ch.5 and DBI in-memory storage notes.");
}

fn day16_virtual_addresses() {
    section(16, "Virtual addresses and process maps");
    concept(
        "The pointer values printed by this program are virtual addresses. The OS and MMU translate them to \
         physical frames through page tables and TLBs.",
    );
    print_linux_memory_map_hint();
    reading("OSTEP: address translation and virtual memory overview.");
}

fn day17_pages_and_page_size() {
    section(17, "Pages and page-sized access");
    let page = page_size();
    println!("reported page size: {} bytes", page);
    let pages = 4096;
    let mut data = vec![0_u8; pages * page];
    time_it("touch one byte per page", || {
        let mut sum = 0_u64;
        for i in (0..data.len()).step_by(page) {
            data[i] = data[i].wrapping_add(1);
            sum += data[i] as u64;
        }
        sum
    });
    reading("OSTEP: paging. COD Ch.5: virtual memory.");
}

fn day18_demand_paging() {
    section(18, "Demand paging and resident memory");
    let before = current_rss_kb();
    let page = page_size();
    let mut data = vec![0_u8; 128 * 1024 * 1024];
    let after_alloc = current_rss_kb();
    for i in (0..data.len()).step_by(page) {
        data[i] = 1;
    }
    let after_touch = current_rss_kb();
    println!(
        "RSS before allocation: {:?}",
        before.map(|v| format!("{} KiB", v))
    );
    println!(
        "RSS after allocation before touch: {:?}",
        after_alloc.map(|v| format!("{} KiB", v))
    );
    println!(
        "RSS after touching every page: {:?}",
        after_touch.map(|v| format!("{} KiB", v))
    );
    observe("On Linux, allocation and physical commitment can be separated by demand paging and overcommit.");
    reading("OSTEP: page faults and demand paging.");
}

fn day19_tlb_pressure() {
    section(19, "TLB pressure: many pages vs one page");
    let page = page_size();
    let pages = 16_384;
    let data = vec![1_u8; pages * page];
    time_it("many pages, one byte each", || {
        let mut sum = 0_u64;
        for i in (0..data.len()).step_by(page) {
            sum += black_box(data[i] as u64);
        }
        sum
    });
    time_it("same byte count, compact region", || {
        let mut sum = 0_u64;
        for value in data.iter().take(pages) {
            sum += black_box(*value as u64);
        }
        sum
    });
    reading("OSTEP: TLBs. COD Ch.5: virtual memory and caches.");
}

fn day20_mmap() {
    section(
        20,
        "mmap: memory mapping is not the same abstraction as Vec allocation",
    );
    #[cfg(unix)]
    unsafe {
        #[cfg(target_os = "linux")]
        const MAP_ANON_FLAG: i32 = 0x20;
        #[cfg(target_os = "macos")]
        const MAP_ANON_FLAG: i32 = 0x1000;
        const PROT_READ: i32 = 0x1;
        const PROT_WRITE: i32 = 0x2;
        const MAP_PRIVATE: i32 = 0x02;
        const MAP_FAILED: isize = -1;
        unsafe extern "C" {
            fn mmap(
                addr: *mut std::ffi::c_void,
                len: usize,
                prot: i32,
                flags: i32,
                fd: i32,
                offset: isize,
            ) -> *mut std::ffi::c_void;
            fn munmap(addr: *mut std::ffi::c_void, len: usize) -> i32;
        }

        let len = page_size() * 4;
        let ptr = mmap(
            std::ptr::null_mut(),
            len,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANON_FLAG,
            -1,
            0,
        );
        if ptr as isize == MAP_FAILED {
            println!("mmap failed");
        } else {
            println!("mmap returned address 0x{:016x}, len={}", ptr as usize, len);
            let bytes = std::slice::from_raw_parts_mut(ptr as *mut u8, len);
            bytes[0] = 1;
            bytes[len - 1] = 2;
            println!("first={}, last={}", bytes[0], bytes[len - 1]);
            let rc = munmap(ptr, len);
            println!("munmap return code={}", rc);
        }
    }
    #[cfg(not(unix))]
    println!("mmap lab is Unix-only.");
    reading("OSTEP: mmap/VM APIs. Linux man pages: mmap(2), munmap(2).");
}

fn day21_allocator_patterns() {
    section(21, "Allocator patterns: sizes, alignment, reuse");
    let mut boxes = Vec::new();
    for size in [8_usize, 16, 24, 32, 64, 128, 1024] {
        let b = vec![0_u8; size].into_boxed_slice();
        println!(
            "allocated {:>4} bytes at 0x{:016x}",
            size,
            b.as_ptr() as usize
        );
        boxes.push(b);
    }
    drop(boxes);
    let again = Box::new(1234_u64);
    println!(
        "after drops, new Box<u64> at 0x{:016x}",
        &*again as *const u64 as usize
    );
    observe(
        "Allocators often reuse freed chunks. That is not the same as returning pages to the OS.",
    );
    reading("OSTEP: free-space management.");
}

fn day22_drop_vs_return_to_os() {
    section(22, "Drop/free vs returning memory to the OS");
    let before = current_rss_kb();
    let data = vec![0_u8; 256 * 1024 * 1024];
    let after_alloc = current_rss_kb();
    drop(data);
    let after_drop = current_rss_kb();
    println!(
        "RSS before:      {:?}",
        before.map(|v| format!("{} KiB", v))
    );
    println!(
        "RSS after alloc: {:?}",
        after_alloc.map(|v| format!("{} KiB", v))
    );
    println!(
        "RSS after drop:  {:?}",
        after_drop.map(|v| format!("{} KiB", v))
    );
    observe(
        "Drop makes memory reusable by Rust/allocator. The allocator may keep arenas for future allocations \
         instead of immediately returning them to the kernel.",
    );
    reading("OSTEP: free-space management. DBI: in-memory data structures.");
}

fn day23_fragmentation() {
    section(23, "Fragmentation: internal and external");
    let mut arena = BumpArena::new(128);
    for (size, align) in [(3, 1), (5, 8), (9, 8), (17, 16), (7, 4)] {
        let before = arena.used();
        let block = arena.alloc(size, align);
        println!(
            "request size={:>2} align={:>2} before={:>3} after={:>3} block={:?}",
            size,
            align,
            before,
            arena.used(),
            block
        );
    }
    observe("Padding for alignment is internal waste. Holes between live allocations are external waste.");
    reading("OSTEP: segmentation/free-space management.");
}

fn day24_arena_model() {
    section(24, "Arena allocation: bump now, reset later");
    let mut arena = BumpArena::new(256);
    for i in 0..8 {
        let block = arena.alloc(size_of::<u64>(), align_of::<u64>()).unwrap();
        println!(
            "object {} offset={:>3} addr=0x{:016x}",
            i, block.offset, block.addr
        );
    }
    println!("arena used {}/{} bytes", arena.used(), arena.capacity());
    arena.reset();
    println!(
        "after reset used {}/{} bytes",
        arena.used(),
        arena.capacity()
    );
    observe("Arena allocation trades per-object free for very cheap allocation and bulk reset.");
    reading("DBI: memory-native structures. OSTEP: allocation strategies.");
}

fn day25_pool_allocator() {
    section(25, "Fixed-size pool intuition");
    let object_size = 32;
    let mut arena = BumpArena::new(object_size * 8);
    let mut free_list = Vec::new();
    for _ in 0..8 {
        free_list.push(arena.alloc(object_size, object_size).unwrap());
    }
    println!("pool made {} fixed-size slots", free_list.len());
    let a = free_list.pop().unwrap();
    let b = free_list.pop().unwrap();
    println!("checked out slots at offsets {} and {}", a.offset, b.offset);
    free_list.push(a);
    println!("returned one slot; free slots={}", free_list.len());
    reading("Linux kernel docs: slab/slub allocator as a production version of this idea.");
}

fn day26_kernel_stack_context() {
    section(26, "Kernel stack context");
    concept(
        "Kernel stacks are intentionally small. Kernel code avoids deep recursion and large stack locals because \
         a stack overflow can corrupt privileged memory.",
    );
    recursive_stack_probe(0, 8, {
        let marker = 0_u64;
        &marker as *const u64 as usize
    });
    observe("User-space experiments are safer, but the discipline transfers: do not put huge objects on stacks.");
    reading("OSTEP: threads/processes. Linux kernel docs: kernel stacks and memory allocation.");
}

fn day27_pinned_dma_page_cache() {
    section(27, "Pinned memory, DMA, and page cache concepts");
    concept(
        "A user-space pointer is a virtual address, not a physical address. Devices doing DMA need stable physical \
         pages or IOMMU mappings. The kernel page cache is file data cached in memory, not your process heap.",
    );
    let mut buffer = vec![0_u8; page_size() * 2];
    println!("buffer virtual addr=0x{:016x}", buffer.as_ptr() as usize);
    println!("page size={} bytes", page_size());
    buffer[0] = 1;
    buffer[page_size()] = 2;
    observe("Alignment and page boundaries matter when crossing into OS/device interfaces.");
    reading("COD Ch.5. CUDA memory hierarchy and pinned host memory sections.");
}

#[repr(align(64))]
struct PaddedCounter(u64);

fn day28_false_sharing() {
    section(28, "False sharing and multicore memory");
    concept(
        "False sharing happens when independent values share one cache line and different cores keep invalidating \
         each other's copies.",
    );
    println!("size_of::<u64>() = {}", size_of::<u64>());
    println!(
        "size_of::<PaddedCounter>() = {}",
        size_of::<PaddedCounter>()
    );
    let counters = [0_u64; 2];
    let padded = [PaddedCounter(0), PaddedCounter(0)];
    println!(
        "adjacent u64 addresses: 0x{:x}, 0x{:x}",
        &counters[0] as *const u64 as usize, &counters[1] as *const u64 as usize
    );
    println!(
        "padded counter addresses: 0x{:x}, 0x{:x}; values={}, {}",
        &padded[0] as *const PaddedCounter as usize,
        &padded[1] as *const PaddedCounter as usize,
        padded[0].0,
        padded[1].0
    );
    reading("COD Ch.6 parallelism. CUDA: coalescing and memory hierarchy contrast.");
}

fn day29_allocator_case_study() {
    section(29, "Case study: records as boxes vs contiguous vs arena");
    let count = 200_000;
    time_it("Vec<Record> contiguous", || {
        let records: Vec<ReorderedRecord> = (0..count)
            .map(|i| ReorderedRecord {
                id: i as u64,
                tag: (i % 255) as u8,
                flag: i % 2 == 0,
            })
            .collect();
        records.iter().map(|r| black_box(r.id)).sum()
    });
    time_it("Vec<Box<Record>> scattered", || {
        let records: Vec<Box<ReorderedRecord>> = (0..count)
            .map(|i| {
                Box::new(ReorderedRecord {
                    id: i as u64,
                    tag: (i % 255) as u8,
                    flag: i % 2 == 0,
                })
            })
            .collect();
        records.iter().map(|r| black_box(r.id)).sum()
    });
    let mut arena = BumpArena::new(count * size_of::<ReorderedRecord>());
    for _ in 0..count {
        let _ = arena.alloc(size_of::<ReorderedRecord>(), align_of::<ReorderedRecord>());
    }
    println!(
        "arena allocated {} records using {} bytes",
        count,
        arena.used()
    );
    reading("DBI row/column and in-memory notes. DDIA storage engines.");
}

fn day30_capstone() {
    section(30, "Capstone: explain the full memory story");
    concept(
        "For every value, answer five questions: where is the owner, where are the bytes, who can access them, \
         what locality pattern do they create, and when can the storage be reused or returned?",
    );
    let mut out = String::new();
    let local = 1_u64;
    let heap = Box::new(2_u64);
    let vec = vec![3_u64, 4, 5, 6];
    let _ = writeln!(out, "{}", layout_report("local", &local));
    let _ = writeln!(out, "{}", layout_report("Box owner", &heap));
    let _ = writeln!(out, "Box pointee=0x{:016x}", &*heap as *const u64 as usize);
    let _ = writeln!(out, "{}", layout_report("Vec owner", &vec));
    let _ = writeln!(
        out,
        "Vec buffer=0x{:016x} len={} cap={}",
        vec.as_ptr() as usize,
        vec.len(),
        vec.capacity()
    );
    println!("{}", out);
    println!("Write one formal note: Heap, Stack, Caches, and Arenas - The Full Mental Model.");
    reading("Revisit OSTEP VM + allocator chapters, COD Ch.5, DBI in-memory layout, CUDA memory hierarchy.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_day_accepts_named_days_and_all() {
        assert_eq!(parse_command(["memory_lab", "day01"]), Command::RunDay(1));
        assert_eq!(parse_command(["memory_lab", "day30"]), Command::RunDay(30));
        assert_eq!(parse_command(["memory_lab", "all"]), Command::RunAll);
    }

    #[test]
    fn parse_day_rejects_out_of_range_days() {
        assert_eq!(parse_command(["memory_lab", "day00"]), Command::Help);
        assert_eq!(parse_command(["memory_lab", "day31"]), Command::Help);
        assert_eq!(parse_command(["memory_lab"]), Command::Help);
    }

    #[test]
    fn align_up_rounds_offsets_to_requested_alignment() {
        assert_eq!(align_up(0, 8), 0);
        assert_eq!(align_up(1, 8), 8);
        assert_eq!(align_up(8, 8), 8);
        assert_eq!(align_up(9, 8), 16);
    }

    #[test]
    fn bump_arena_allocates_aligned_regions_and_resets() {
        let mut arena = BumpArena::new(64);
        let a = arena.alloc(3, 1).unwrap();
        let b = arena.alloc(8, 8).unwrap();

        assert_eq!(a.offset, 0);
        assert_eq!(b.offset, 8);
        assert_eq!(arena.used(), 16);

        arena.reset();
        assert_eq!(arena.used(), 0);
        assert_eq!(arena.alloc(65, 1), None);
    }

    #[test]
    fn stride_plan_visits_expected_indexes() {
        assert_eq!(stride_indexes(10, 3), vec![0, 3, 6, 9]);
        assert_eq!(stride_indexes(0, 4), Vec::<usize>::new());
    }

    #[test]
    fn fixed_visit_strided_sum_keeps_work_count_constant() {
        let data = [1, 2, 3, 4];

        assert_eq!(strided_sum_fixed_visits(&data, 1, 4), 10);
        assert_eq!(strided_sum_fixed_visits(&data, 2, 4), 8);
        assert_eq!(strided_sum_fixed_visits(&data, 0, 4), 0);
    }

    #[test]
    fn layout_report_includes_size_alignment_and_address() {
        let value = 42_u64;
        let report = layout_report("u64 local", &value);

        assert!(report.contains("u64 local"));
        assert!(report.contains("size=8"));
        assert!(report.contains("align=8"));
        assert!(report.contains("addr=0x"));
    }

    #[test]
    fn format_bits_u8_groups_bits_by_position() {
        assert_eq!(format_bits_u8(0b1010_1100), "1010_1100");
        assert_eq!(format_bits_u8(0), "0000_0000");
        assert_eq!(format_bits_u8(255), "1111_1111");
    }

    #[test]
    fn dump_bytes_numbers_each_addressed_cell() {
        let dump = dump_bytes(&[0x12, 0x34, 0xab]);

        assert!(dump.contains("address"));
        assert!(dump.contains("0x00"));
        assert!(dump.contains("0x12"));
        assert!(dump.contains("0x02"));
        assert!(dump.contains("1010_1011"));
    }

    #[test]
    fn tiny_ram_loads_and_stores_addressed_bytes() {
        let mut ram = TinyRam::new(4);

        assert_eq!(ram.load(2), Some(0));
        assert_eq!(ram.store(2, 0xaa), Some(()));
        assert_eq!(ram.load(2), Some(0xaa));
        assert_eq!(ram.load(4), None);
        assert_eq!(ram.store(4, 1), None);
    }

    #[test]
    fn tiny_cpu_load_add_store_moves_data_between_ram_and_registers() {
        let mut cpu = TinyCpu::new(TinyRam::from_bytes(&[7, 5, 0, 0]));

        cpu.load(0, 0).unwrap();
        cpu.load(1, 1).unwrap();
        cpu.add(2, 0, 1);
        cpu.store(2, 2).unwrap();

        assert_eq!(cpu.registers, [7, 5, 12]);
        assert_eq!(cpu.ram.load(2), Some(12));
    }
}
