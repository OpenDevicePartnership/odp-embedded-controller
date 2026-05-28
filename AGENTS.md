# AGENTS.md — odp-embedded-controller

Operational guide for AI coding agents working in this repository. This
file complements `.github/copilot-instructions.md` (deep
architecture/conventions) with the *workflow* an agent must follow:
build, lint, test, commit, and push.

If you are a human contributor, read `README.md` and
`.github/copilot-instructions.md` first; this file repeats their
essentials so an autonomous agent has a single entry point.

---

## 1. Repository shape

- Language: Rust, `#![no_std]` / `#![no_main]` embedded firmware.
- **No workspace root.** Each platform under `platform/` is a
  standalone crate built from its own directory. `cargo` commands
  *must* be run inside `platform/<crate>/` so the per-crate
  `.cargo/config.toml` (target triple, linker, runner) is honoured.
- Crates:
  - `platform/platform-common` — shared `no_std` library (no build
    target; fmt/clippy/check only).
  - `platform/dev-imxrt` — NXP i.MXRT685S, `thumbv8m.main-none-eabihf`
    (requires `flip-link`).
  - `platform/dev-mcxa` — NXP MCXA266, `thumbv8m.main-none-eabihf`
    (requires `flip-link`).
  - `platform/dev-npcx` — Nuvoton NPCX498M, `thumbv7em-none-eabihf`
    (requires `flip-link`).
  - `platform/dev-qemu` — QEMU `virt`, `riscv32imac-unknown-none-elf`
    (no `flip-link`).
- Toolchain pinned in `rust-toolchain.toml` (stable + all three
  targets + `rust-src`, `rustfmt`, `clippy`, `llvm-tools-preview`).
- MSRV: `1.83`.

## 2. Setup

One-time host setup (only what isn't auto-installed by `rustup`):

```sh
cargo install flip-link --locked        # required by dev-imxrt / dev-mcxa / dev-npcx
cargo install --locked cargo-deny       # optional but used by check-all.sh
```

Targets and components install automatically the first time `cargo` is
invoked inside the repo.

## 3. Build / lint / test commands

There are **no unit or integration tests in-tree**. Validation is:
formatting, clippy, build, doc, feature-powerset check, cargo-deny,
cargo-machete, no_std check, and an out-of-tree QEMU integration test
driven by `scripts/integration-test.sh`.

### 3.1 Per-crate gates (run from `platform/<crate>/`)

| Gate              | Command                                                  | Crates                                        |
|-------------------|----------------------------------------------------------|-----------------------------------------------|
| Format            | `cargo fmt --check`                                      | `platform-common`, all `dev-*`                |
| Build             | `cargo build --locked`                                   | all `dev-*`                                   |
| Clippy            | `cargo clippy --locked -- -D warnings`                   | all `dev-*` (CI uses `cargo clippy --locked`) |
| Doc               | `RUSTDOCFLAGS=--cfg docsrs cargo doc --locked --no-deps --all-features` | all `dev-*`                |
| Feature powerset  | `cargo hack --locked --feature-powerset check`           | all `dev-*`                                   |
| Dependency policy | `cargo deny --locked --all-features check`               | all `dev-*`                                   |
| Unused deps       | `cargo machete`                                          | all `dev-*`                                   |
| no_std check      | `cargo check --locked --all-features`                    | all `dev-*`                                   |
| MSRV check        | `cargo +1.83 check --locked`                             | all `dev-*`                                   |

`cargo clippy --locked` on its own is what CI runs (`check.yml`); the
local `-- -D warnings` form mirrors `scripts/check-all.sh` and is the
stricter gate to use locally because the crates set
`warnings = "deny"` in `[lints.rust]` anyway.

### 3.2 Full local quality gate (recommended before every push)

```sh
bash scripts/check-all.sh
```

Runs `fmt → build → clippy -D warnings → cargo deny` across
`platform-common` and every `dev-*`, mirroring `.github/workflows/check.yml`.
Skips `cargo-deny` with a warning if it isn't installed; warns (but
proceeds) if `flip-link` is missing.

### 3.3 QEMU integration test

```sh
./scripts/integration-test.sh
```

Requires `qemu-system-riscv32` (Debian/Ubuntu: `qemu-system-misc`),
`libudev-dev`, and the pinned `ec-test-cli` binary
(`cargo install --git https://github.com/OpenDevicePartnership/odp-platform-common --locked --rev <pin from check.yml> ec-test-cli`).
Runs `dev-qemu` and exercises every `ec-test-cli` command against it.

### 3.4 Common pitfalls

- Running `cargo` from the repo root with `--manifest-path` bypasses
  the per-platform `.cargo/config.toml` and fails (notably `dev-qemu`
  cannot find its target). Always `cd platform/<crate>/` first.
- `linker 'flip-link' not found` → install `flip-link`.
- `can't find crate for 'core'` → toolchain targets weren't installed;
  run `rustup show` from repo root, or `rustup target add <triple>`.

## 4. Code conventions (agent-relevant subset)

See `.github/copilot-instructions.md` for the full version. Key rules
an agent must respect:

- **Static allocation only.** No heap, no `alloc`. Use `StaticCell<T>`
  for owned init-once values and `OnceLock<T>` for lazily shared
  references.
- **Embassy async executor.** Entry point is
  `#[embassy_executor::main] async fn main(spawner: Spawner)`. All
  concurrency is cooperative async tasks.
- **Inter-task comms** go through a typed
  `PubSubChannel<ThreadModeRawMutex, Message, ...>` per platform.
- **Shared I2C** via `Mutex<ThreadModeRawMutex, I2cMaster<'static, Async>>`
  inside a `StaticCell`, accessed through `I2cDevice::new(...)`.
- **Logging:** `defmt` only, transported over RTT. Derive
  `defmt::Format` on logged types.
- **Panic handlers:** `panic_halt` in release, `panic_probe` in debug
  (gated on `debug_assertions`).
- **Errors:** `.expect("descriptive message")` for init paths that
  must succeed; `Result<_, _>` for recoverable runtime ops; minimal
  custom error enums.
- **Lints:** `warnings = "deny"` plus
  `correctness/perf/suspicious/style = "deny"` for clippy in every
  crate. Do not weaken these.
- **rustfmt:** `max_width = 120` (see `rustfmt.toml`). Always
  `cargo fmt` before committing.
- **Module layout:** subsystems are directory modules with `mod.rs`
  re-exporting children.

When adding a new platform, mirror the matrix entries in
`.github/workflows/check.yml`, `nostd.yml`, and `scripts/check-all.sh`
(the workflow comment explicitly calls this out — there is no single
source of truth for the platform list).

## 5. Commit & PR workflow

### 5.1 Commit messages

- Subject line: capitalised, ≤ 50 chars, imperative mood ("Add foo",
  not "Added foo" / "Adds foo").
- Blank line, then body wrapped at 72 chars explaining *what* and
  *why*, not *how*.
- Reference issues/PRs in the body where relevant.

### 5.2 AI attribution (mandatory for any AI-assisted commit)

Every commit produced with AI assistance **must** carry an
`Assisted-by` trailer:

```
Assisted-by: AGENT_NAME:MODEL_VERSION [TOOL1] [TOOL2]
```

- `AGENT_NAME` — e.g. `GitHub Copilot`.
- `MODEL_VERSION` — the *actual* model the agent is running as. The
  agent **must verify its own identity** before composing the
  trailer; never hard-code a previous session's model.
- Optional bracketed tools: specialised analysis tools used
  (`coccinelle`, `clang-tidy`, …). Do **not** list editors, `git`, or
  `cargo`.
- Agents **must not** add `Signed-off-by`. Only humans can certify
  the DCO.

### 5.3 Author identity (agent runs)

When committing on behalf of a human, set author per-commit; never
mutate global git config:

```sh
git -c user.name="Felipe Balbi" \
    -c user.email="felipe.balbi@microsoft.com" \
    commit -m "..." -m "Assisted-by: GitHub Copilot:<model>"
```

### 5.4 Branch / push policy

- Work on a topic branch off `main` (e.g.
  `improve-agentic-workflow`). Never commit directly to `main`.
- Push to the contributor's **fork** remote, not `origin`. Never
  force-push to a shared branch.
- Open PRs against `OpenDevicePartnership/odp-embedded-controller`
  `main`. Code-review ownership is in `CODEOWNERS`.

## 6. Continuous Integration

Workflows in `.github/workflows/`:

- `check.yml` — per-platform fmt, clippy, doc, cargo-hack, cargo-deny,
  cargo-machete, MSRV, build, and the `dev-qemu` integration-test job.
- `nostd.yml` — `cargo check --locked --all-features` per platform
  to keep everything `no_std`-clean.
- `benchmark.yml` — binsize tracking (`dev-imxrt` / `dev-npcx`).
- `rolling.yml` — nightly dependency-update verification.
- `cargo-vet.yml`, `cargo-vet-pr-comment.yml` — supply-chain audit.

`scripts/check-all.sh` is the local mirror of `check.yml`. Run it (or
the equivalent per-crate commands above) before pushing.

## 7. Files & directories of interest

- `platform/<crate>/Cargo.toml` — pinned deps, lint config, MSRV.
- `platform/<crate>/.cargo/config.toml` — target, linker (`flip-link`
  where applicable), runner (`probe-rs` / `qemu-system-riscv32`).
- `platform/<crate>/src/main.rs` — entry point and task spawn graph.
- `platform/platform-common/src/` — shared HAL/service abstractions.
- `deny.toml` — license / source / advisory policy.
- `rustfmt.toml` — formatter config.
- `rust-toolchain.toml` — pinned toolchain + targets + components.
- `CODEOWNERS` — review routing.
- `.github/copilot-instructions.md` — deep convention reference.

## Model selection & cost discipline

Premium models (Opus, GPT-5 family, "high"/"xhigh" reasoning variants)
cost an order of magnitude more than standard models (Sonnet, Haiku,
mini). Most steps in a typical task do not need premium reasoning,
and over-using premium models wastes credits without improving
outcomes. The rules below apply to *all* model selection: your own
session, sub-agents launched via the `task` tool, and parallel work
launched via `/fleet`.

### Default posture

- **Default to the cheapest model that can do the job.** Reach for a
  premium model only when one of the escalation triggers below is hit.
- **Plan with premium, execute with cheap.** Spend at most one or two
  premium turns on design / planning, then downshift to a cheaper
  model for mechanical execution of the plan.
- **Never bump the model "just in case."** If you cannot articulate
  *why* a cheaper model would fail, use the cheaper model.

### Escalation triggers (use a premium model)

Reach for a premium model when *any* of these are true:

- Cross-module refactor, architectural design, or API design from
  scratch.
- Subtle correctness reasoning: concurrency, lifetimes, `unsafe`,
  FFI ABI, cryptography, safety-critical control paths.
- Debugging a failure that survived one prior cheap-model attempt.
- Reviewing code on a safety-, security-, or money-critical path.
- The diff cannot be predicted in advance — i.e. there is genuine
  creative or design work to do, not just typing.

### De-escalation triggers (use a cheap model)

Use the cheapest available model when *any* of these are true:

- Searching, reading, summarising files or docs.
- Single-file mechanical edits: rename, format, lint fix, dependency
  bump, boilerplate, scaffolding from a known template.
- Generating tests for code that already works.
- Running builds, tests, linters, or other commands where the model
  only needs to report success/failure.
- Routine commits, PR descriptions, changelog entries.
- The diff is essentially predictable before generation.

### Sub-agent routing (the `task` tool)

When delegating with the `task` tool, set `model:` explicitly. Do not
let sub-agents inherit a premium default for cheap work.

| Sub-agent type    | Default model             | Override to                                     |
|-------------------|---------------------------|-------------------------------------------------|
| `explore`         | cheap                     | keep cheap (`claude-haiku-4.5` or `gpt-5-mini`) |
| `task` (run cmd)  | cheap                     | keep cheap                                      |
| `research`        | cheap for breadth         | premium only for the final synthesis            |
| `general-purpose` | match task                | cheap for mechanical work; premium for design   |
| `rubber-duck`     | premium                   | keep premium — this is where reasoning pays off |
| `code-review`     | premium on critical paths | cheap on cosmetic / mechanical diffs            |

### `/fleet` (parallel sub-agents) rules

- Fleet mode multiplies cost by the fleet width. Apply the rules
  above *per worker*, not in aggregate.
- Split a fleet job along complexity lines: route the cheap,
  parallelisable workers (file edits, test runs, doc updates) to a
  cheap model; reserve premium models for the small number of
  workers that need real reasoning.
- If every worker in a fleet would need a premium model, the work is
  probably not a good fit for fleet mode — reconsider the
  decomposition before paying N× premium.

### Session hygiene

- Keep sessions short and focused. Long premium sessions are the
  single largest source of waste because every turn re-processes the
  full history.
- Use `/compact` when the conversation grows long, and `/new` for
  unrelated work.
- Prefer `/ask` for one-off side questions so they don't extend the
  main session.

### When in doubt

Ask: *"If a cheaper model produced the wrong answer here, would I
catch it in seconds (compiler, tests, my own review) or in
weeks (production incident)?"* If the former, use the cheap model
and let the feedback loop do its job.
