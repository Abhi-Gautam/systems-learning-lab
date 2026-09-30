# Systems & AI Infrastructure

A dependency-led curriculum for handwritten systems experiments and technical design study. This is a map of possible learning stops, not a fixed calendar or a claim of completed work.

## Navigation

- [Visual roadmap](roadmap.html): prerequisite graphs and expandable scope cards.
- [Curriculum data](curriculum.json): portable node IDs, topics and dependencies.
- [AI infrastructure charter](../../labs/ai-infrastructure/README.md): scope and environment boundaries.

## Learning loop

Choose one question → predict → implement → run/inspect → compare → explain → change one condition. Keep one active experiment; use supplementary reading to explain the mechanism rather than adding parallel book queues.

## Shared systems spine

- Ownership and layout → compilation/linking/loading → processes and kernel boundaries → virtual memory.
- Concurrency → scheduling and parallelism; storage and networking branch from their relevant foundations.
- Measure throughout; consolidate performance analysis, isolation and accelerator work when the experiment requires them.
- Integrate mechanisms into one product-shaped systems component rather than several unfinished projects.

## AI infrastructure branches

| Branch | Foundation |
|---|---|
| Tensor workloads | Layout, numerical representation and a CPU baseline |
| Accelerator execution | Memory, concurrency and the actual runtime/driver/kernel stack |
| Model execution | Tensor and accelerator foundations; durable checkpoints for training |
| Serving | Inference, concurrency and networking |
| Distributed compute | Model execution, parallelism, networking and measurement |
| GPU platform | Device stack, scheduling, networking and isolation |
| Agent execution | Process lifecycle, concurrency, storage, networking and isolation |
| Rollout/evaluation | Agent execution, model/policy mechanics and scheduling |

Hosted inference can support agent-execution experiments without building a serving stack. Distributed training additionally needs the training/checkpoint branch; training coupled to rollouts also needs the relevant distributed execution checkpoint.

## Experiment boundaries

- Prerequisite arrows refer to the relevant checkpoint, not every advanced topic in the preceding stop.
- Raspberry Pi/Linux is suitable for many OS experiments, not CUDA computation.
- Apple Silicon/Metal/MLX can demonstrate transferable accelerator principles, but is not equivalent to CUDA.
- Verify actual NVIDIA/multi-device/node access before hardware-specific experiments.
- Approve a spending limit before paid runs, record versions, and release resources afterward.
- Simulation and manifests can test protocols but do not prove GPU compute, production scale or secure isolation.

## Stops

### S00 · Ownership & invariants

**Prerequisites:** entry checkpoint

- addresses, object lifetime, layout, aliasing
- state transitions and invariant-preserving updates
- Rust ownership/borrowing introduced through actual bytes and resources

**Supplementary reading:** CSAPP: data representation and machine-level programs; COD: instructions and memory representation

### S01 · Executable → running program

**Prerequisites:** S00

- calling convention, stack/register use and compiler output
- translation units, defined/undefined symbols and relocations
- static/shared libraries, entrypoint, dynamic loader and mappings

**Supplementary reading:** COD §2.12–2.13; CSAPP: machine-level programs and linking

### S02 · Processes & the kernel boundary

**Prerequisites:** S01

- user/kernel mode, syscall vs ordinary call
- fork/exec/wait, resource ownership and exit status
- file descriptors, pipes, signals and cleanup

**Supplementary reading:** CSAPP: exceptional control flow and system-level I/O

### S03 · Virtual memory & allocation

**Prerequisites:** S00, S02

- virtual vs physical address, page tables and TLB
- demand faults, permissions, mmap and copy-on-write
- allocation/free lists, fragmentation and lifetime

**Supplementary reading:** CSAPP: virtual memory; COD: memory hierarchy

### S04 · Concurrency correctness

**Prerequisites:** S00, S02, S03

- threads/shared memory, races and legal interleavings
- mutex/condition-variable predicate, sleep/wake and futex
- language memory models, acquire/release and Rust Send/Sync
- cancellation, resource lifetimes and structured shutdown

**Supplementary reading:** CSAPP: concurrent programming

### S05 · Scheduling & parallel execution

**Prerequisites:** S04

- preemption, context-switch cost, CPU affinity and runnable vs blocked
- parallel work partitioning, queue policy and work stealing
- oversubscription, fairness, throughput and tail latency

**Supplementary reading:** COD: parallelism and performance

### S06 · Storage, filesystems & recovery

**Prerequisites:** S02, S03

- file API, VFS/inodes and page cache
- persistent page I/O, buffer pool, eviction and full tree splits
- WAL ordering, durable reopen, crash recovery and later transaction visibility

**Supplementary reading:** Database Internals: storage structures and recovery; CSAPP: system-level I/O

### S07 · Networking & event-driven I/O

**Prerequisites:** S02, S04

- socket state, TCP stream framing, partial reads/writes and shutdown
- DNS/TLS and NIC-to-userspace data path
- nonblocking descriptors, readiness/epoll, backpressure and async runtime
- timeouts, retries and request identity

**Supplementary reading:** CSAPP: network and concurrent programming

### S08 · Measure → explain → optimize

**Prerequisites:** S03, S04, S05

- Linux perf/stat/record, tracing, CPU vs off-CPU time
- cache locality, coherence/false sharing, bandwidth and NUMA
- allocation, sharding, batching, vectorization and workload validity

**Supplementary reading:** COD: memory hierarchy and performance; CSAPP: optimizing program performance

### S09 · Isolation & virtualization

**Prerequisites:** S02, S03, S05, S07

- namespaces, cgroups, capabilities and syscall restrictions
- resource isolation vs hostile-code security
- VM privilege/translation/device model and containers vs VMs

**Supplementary reading:** Linux/kernel primary documentation selected for the experiment

### S10 · Accelerator execution & stack

**Prerequisites:** S00, S03, S04

- host/device memory and execution groups; SIMT/SIMD-group model as hardware permits
- memory hierarchy, coalescing, synchronization and transfer/computation boundaries
- what driver, runtime, compiler, kernel, profiler and library each own
- CUDA vs Metal/MLX vs AMD stack: transferable principles and non-equivalences

**Supplementary reading:** Primary documentation for the actual accelerator hardware and software stack

### S11 · Integrated systems capstone

**Prerequisites:** S04, S06, S07, S08

- one coherent runtime/storage/networking component
- explicit resource, durability and concurrency contracts
- fault injection, observability, benchmarks and design evolution

**Supplementary reading:** Relevant sections only; no additional mandatory book queue

### A00 · Tensor & workload foundations

**Prerequisites:** S00

- tensor shape, stride, dtype and numerical precision
- matrix multiplication, operation count vs bytes moved, CPU baseline
- minimal transformer/autograd vocabulary only as needed to understand execution

**Supplementary reading:** Primary documentation and implementation selected for the active experiment

### A01 · Transformer inference execution

**Prerequisites:** A00, S10

- trace tokens through embedding, attention, normalization and projection
- KV-cache ownership, sizes, lifetime and layout
- prefill/decode, precision/quantization, fused kernels and memory limits

**Supplementary reading:** Primary documentation and implementation selected for the active experiment

### A02 · Training & checkpoint execution

**Prerequisites:** A00, S10, S04, S06

- training forward/backward and optimizer-state resource costs
- data loading, checkpoint format/order and restart correctness
- precision, accumulation and execution/communication overlap at bounded scale

**Supplementary reading:** Primary documentation and implementation selected for the active experiment

### A03 · Single-device serving

**Prerequisites:** A01, S04, S07

- request lifecycle and continuous batching
- KV capacity, admission, cancellation and backpressure
- latency distributions, throughput, workload mix and cost

**Supplementary reading:** Primary documentation and implementation selected for the active experiment

### A04 · Distributed accelerator compute

**Prerequisites:** A01, S05, S07, S08

- data/tensor/pipeline/expert parallelism when the workload forces them
- communication collectives, topology and overlap
- placement, stragglers, failure/restart and bandwidth costs

**Supplementary reading:** Primary documentation and implementation selected for the active experiment

### A05 · GPU cloud & private platform

**Prerequisites:** S10, S05, S07, S09

- node provisioning, drivers, device exposure and compatibility
- resource isolation/sharing and scheduler admission
- observability, private-network boundaries, upgrades and recovery

**Supplementary reading:** Primary documentation and implementation selected for the active experiment

### A06 · Agent sandbox & execution platform

**Prerequisites:** S02, S04, S06, S07, S09

- sandbox threat model, tool permissions and resource limits
- workspace/artifact ownership, durable jobs and retry-safe effects
- environment lifecycle, verifiers, traces and evaluation data

**Supplementary reading:** Primary documentation and implementation selected for the active experiment

### A07 · RL rollout & evaluation infrastructure

**Prerequisites:** A06, A01, S05

- rollout scheduling, environment/version identity and backpressure
- verifiers, evaluation validity and data provenance
- policy-version boundaries, failures and durable experiment results

**Supplementary reading:** Primary documentation and implementation selected for the active experiment

## Publication boundary

This directory contains curriculum scope only. Personal checkpoints, source-origin discussions, session references, local machine paths and progress evidence are kept outside the public projection.
