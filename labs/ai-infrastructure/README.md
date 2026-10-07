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

## Experiments

Each experiment is its own git repo and GitHub repo, checked out inside this
folder and ignored by the parent repo (see `.gitignore`). Every experiment repo
is also added to the same GitHub app installation as `systems-learning-lab`, so
agents working from here can reach all of them.

| Folder | Repo | Question | State |
|---|---|---|---|
| `shopkeeper/` | [Abhi-Gautam/shopkeeper](https://github.com/Abhi-Gautam/shopkeeper) | Evals: make a model-run grocery counter more correct, faster and cheaper, one change at a time on the same shelf and customers | Done; article published |

Shopkeeper does not replace the layers above, and finishing it does not
complete A06.

## How an experiment is run

This is the loop shopkeeper established. Follow it for the next one.

1. **Align first, in this folder.** Agree on the problem statement and the
   learning goal (which layer above it teaches, what we predict) before any
   code. Then create the folder and the repo.
2. **New repo.** `mkdir <name> && cd <name> && git init`, create
   `github.com/Abhi-Gautam/<name>`, add it to `.gitignore` here and to the
   table above, and add it to the GitHub app installation.
3. **Local setup.** Everything runs locally from a `Makefile` (`make db`,
   `make run`, ...), Python in `.venv` from `requirements.txt`, local services
   (e.g. Phoenix on :6006) via `docker compose`. Secrets live in `.env`, never
   committed; `.env.example` lists every variable with a comment.
4. **Model access.** Hosted models through OpenAI-compatible endpoints: OpenAI
   directly, or OpenRouter (one key, many models, e.g. Jev via
   `JEV_OPENROUTER_KEY`). Use recent models, not old open-weight ones. Make the
   base URL and model name env vars so a run can switch provider without code.
5. **Measure the same way every time.** Fixed inputs (shelf, customers), every
   run traced (Phoenix), every change is one commit, and every result is tied
   to the commit that produced it.
6. **Write it up.** The blog post lives in the experiment repo at
   `docs/public/article.md` with images in `docs/public/assets/`. Frontmatter
   carries `title`, `description`, `published`, `project`, `repository`, and
   `sourceCommit`, which is pinned to the commit the reported runs used and
   updated whenever new runs are added. Plain sentences, the story of each
   version, ending with where the experiment stands.
7. **README in plain language**: what it is, what it is for, how to run it.

## Working rule

Choose one concrete question, predict what an experiment will show, run it on
appropriate hardware, compare observation with prediction, and trace the
mechanism. Read production code and documentation to understand what the
experiment omits. Grow the structure and select specialties as evidence and
curiosity develop; there is no locked schedule or predetermined project list.
