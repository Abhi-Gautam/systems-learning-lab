# AI Infrastructure

A long-horizon systems lab for understanding and eventually building the compute
platforms and workloads behind hosted models, GPU clouds, private AI deployments,
and agent/RL infrastructure. The goal is infrastructure-engineering depth, not
becoming an ML researcher or collecting framework tutorials.

## Scope (direction, not a fixed syllabus)

| Layer | Questions to investigate |
|---|---|
| GPU machine | How do execution, memory, synchronization, and CPU–GPU interaction determine performance? |
| Accelerator stack | What do drivers, runtimes, kernels, profilers, CUDA, Metal, and libraries each own? |
| Model execution | How do transformer inference, KV caches, batching, and training workloads use the machine? |
| Serving and distributed compute | How do placement, parallelism, networking, scheduling, admission, latency, throughput, and cost interact? |
| Cloud and private platform | How are GPU nodes provisioned, exposed, isolated, monitored, shared, upgraded, and recovered? |
| Agent and RL execution | How do sandboxes, environments, rollouts, verifiers, data pipelines, and evaluation run safely at scale? |

Explore the connections between layers, but do not reduce the lab to a single
agent-to-inference demo. Distinguish experiments that explain a mechanism from
proof that a production-scale platform works.

## Learning environments

- Apple Silicon + Metal/MLX: local experiments in transferable GPU principles
  and model execution; not a CUDA replacement.
- Rented NVIDIA GPUs (Modal already used by the learner): CUDA, profiling,
  inference, and multi-GPU experiments when the question requires real hardware.
- Linux and other suitable environments: process isolation, networking,
  containers, GPU node mechanics, and platform control-plane experiments.
- Additional providers or machines when managed GPU rental hides a mechanism
  we need to inspect. Access is a prerequisite to verify, not an assumption.

Set a spending guardrail before paid runs, record hardware/software versions and
measurements, and release resources after experiments. No paid workload is
implied by this README.

## Relationship to the other labs

`cs-from-silicone/` supplies the machine, OS, concurrency, and networking model.
`db-from-scratch/` supplies storage, indexing, and recovery intuition. Apply
abstraction and LLD practice inside these systems and this lab instead of
requiring a separate LLD assignment queue.

## Active experiment

`shopkeeper/` is the first agent experiment: a DuckDB shelf exposed
over MCP, and a counter whose model may only guide or buy. Hosted
free-tier inference. It does not replace the layers above, and
finishing it does not complete A06.

## Working rule

Choose one concrete question, predict what an experiment will show, run it on
appropriate hardware, compare observation with prediction, and trace the
mechanism. Read production code and documentation to understand what the
experiment omits. Grow the structure and select specialties as evidence and
curiosity develop; there is no locked schedule or predetermined project list.
