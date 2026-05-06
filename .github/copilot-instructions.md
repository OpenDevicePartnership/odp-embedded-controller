# Copilot Instructions — odp-embedded-controller

## Build & Lint

There is no workspace root `Cargo.toml`. Each platform is a standalone `#![no_std]` / `#![no_main]` binary crate built independently from its own directory.

```sh
# Build a specific platform (from repo root)
cd platform/<name>        # dev-imxrt | dev-npcx | dev-qemu
cargo build               # debug
cargo build --release     # release (LTO, opt-level "z")

# Lint / format (CI runs these per-platform)
cargo fmt --check         # in platform/<name>/
cargo clippy --locked     # in platform/<name>/

# Flash to hardware via probe-rs (configured in .cargo/config.toml)
cargo run                 # debug, flashes and runs
cargo run --release       # release
```

There are no unit tests or integration tests in this repository — it is a `no_std` embedded firmware project targeting bare-metal ARM Cortex-M MCUs.

## Architecture

### Platform targets

| Platform | MCU | Cortex | Target | Role |
|---|---|---|---|---|
| `dev-imxrt` | i.MXRT685S | M33 | `thumbv8m.main-none-eabihf` | Minimal dev board |
| `dev-npcx` | NPCX498M | M4F | `thumbv7em-none-eabihf` | NPCX dev board |
| `dev-qemu` | QEMU RISC-V virt | — | `riscv32imac-unknown-none-elf` | QEMU dev board |

### Crate dependency graph

```
platform/<target>/     ← standalone binary crate (no workspace)
  └─ platform-common/  ← shared no_std library (local path dep)
       └─ OpenDevicePartnership service crates (git, branch v0.2.0)
            └─ embassy-* HAL forks (git, per-MCU family)
```

All OpenDevicePartnership service crates (`battery-service`, `thermal-service`, `power-policy-service`, `time-alarm-service`, `espi-service`, `type-c-service`, etc.) are git dependencies pinned to the `v0.2.0` branch. Embassy HAL crates use per-MCU custom forks.

### Runtime model

The firmware runs on the **Embassy async executor** (single-threaded, cooperative). Entry point is `#[embassy_executor::main] async fn main(spawner: Spawner)`. All concurrency is via async tasks, not threads or interrupts-as-tasks.

Initialization follows a fixed sequence:
1. HAL init (`embassy_<hal>::init()`)
2. Global service init (`embedded_services::init().await`)
3. Hardware configuration (GPIO, I2C buses, peripherals)
4. Task spawning via `spawner.must_spawn()` / `spawner.spawn()`
5. Main event loop (subscribes to `MESSAGE_BUS`)

### Inter-task communication

Tasks communicate through `PubSubChannel<ThreadModeRawMutex, Message, ...>` — a typed publish/subscribe bus. Each platform defines its own `Message` enum.

## Key Conventions

### Static allocation

Everything is statically allocated — no heap, no `alloc`. Use:
- **`StaticCell<T>`** for owned values initialized once at startup (services, drivers, bus wrappers)
- **`OnceLock<T>`** for lazily initialized shared references (configs, service handles via `Service::init`)

```rust
use embassy_sync::once_lock::OnceLock;

static UART_SERVICE: StaticCell<uart_service::Service> = StaticCell::new();
let service = UART_SERVICE.init(uart_service::Service::new(...));

static TIME_SERVICE: OnceLock<time_alarm_service::Service> = OnceLock::new();
let service = time_alarm_service::Service::init(&TIME_SERVICE, ...).await;
```

### I2C bus sharing

I2C buses are shared via `Mutex<ThreadModeRawMutex, I2cMaster<'static, Async>>` wrapped in a `StaticCell`, then accessed through `I2cDevice::new(locked_bus)` from `embassy-embedded-hal::shared_bus`. Multiple peripherals on the same bus each get their own `I2cDevice` handle.

### Error handling

- `.expect("descriptive message")` for initialization failures that should never happen at runtime
- `StaticCell::init()` is called directly (it panics on double-init, does not return `Result`)
- `.expect()` / `.unwrap()` for `OnceLock::init()` and infallible conversions
- `Result<(), Error>` returns for recoverable operations
- Custom error types are minimal (e.g., `enum BatteryError { Failed }`)

### Logging

Use `defmt` for all runtime logging. Derive `defmt::Format` on types that need formatting. Transport is RTT (Real-Time Transfer) via `defmt-rtt`.

```rust
use defmt::{info, error, trace};
info!("Sensor initialized");
```

### Panic handlers

```rust
#[cfg(not(debug_assertions))]
use panic_halt as _;        // Silent halt in release
#[cfg(debug_assertions)]
use panic_probe as _;       // Probe-rs aware panic in debug
```

### Lint strictness

All platforms enforce in `Cargo.toml`:
```toml
[lints.rust]
warnings = "deny"

[lints.clippy]
correctness = "deny"
perf = "deny"
suspicious = "deny"
style = "deny"
```

### Module structure

Subsystems use directory modules with `mod.rs`:
```
power/
  mod.rs       # pub mod charger; pub mod pd;
  charger.rs
  pd.rs
```

### Commit Messages
- Subject line: capitalized, 50 characters or less, imperative mood (e.g., "Fix bug" not "Fixed bug")
- Separate subject from body with a blank line
- Wrap body text at 72 characters
- Use the body to explain *what* and *why*, not *how*

### AI Attribution
Every commit that includes AI-generated or AI-assisted work **must** contain an `Assisted-by` trailer in the commit message:
```
Assisted-by: AGENT_NAME:MODEL_VERSION [TOOL1] [TOOL2]
```
Where:
- `AGENT_NAME` is the name of the AI tool or framework (e.g., `GitHub Copilot`)
- `MODEL_VERSION` is the specific model version used (e.g., `claude-opus-4.6`)
- `[TOOL1] [TOOL2]` are optional specialized analysis tools used (e.g., `coccinelle`, `sparse`, `smatch`, `clang-tidy`)
Basic development tools (git, cargo, editors) should not be listed.
AI agents **must** verify their own identity (agent name and model version) before composing the `Assisted-by` trailer — do not assume or hard-code a model name from a previous session.
AI agents **MUST NOT** add `Signed-off-by` tags. Only humans can certify the Developer Certificate of Origin.

### Formatting

`rustfmt.toml` sets `max_width = 120`.


