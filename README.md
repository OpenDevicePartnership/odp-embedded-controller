# odp-embedded-controller

Public reference and demo firmware for Embedded Controllers (EC) built on
[Open Device Partnership](https://github.com/OpenDevicePartnership) components.
This repository contains development targets suitable for experimentation,
integration testing, and as a starting point for downstream EC projects.

## Scope

This repository hosts the four public `dev-*` development targets and their
shared `platform-common` library. Vendor- and silicon-specific production
platforms are maintained separately and are not in scope here.

## Platforms

| Crate | Role | Target |
|-------|------|--------|
| `platform-common` | Shared `no_std` library crate — HAL traits, board abstractions, common services | (library, no build target) |
| `dev-imxrt` | Development target on NXP i.MXRT685S (Cortex-M33) | `thumbv8m.main-none-eabihf` |
| `dev-mcxa` | Development target on NXP MCXA266 (Cortex-M33) | `thumbv8m.main-none-eabihf` |
| `dev-npcx` | Development target on Nuvoton NPCX498M (Cortex-M4F) | `thumbv7em-none-eabihf` |
| `dev-qemu` | Development target under QEMU `virt` machine (RISC-V 32-bit) | `riscv32imac-unknown-none-elf` |

`platform-common` is consumed by each `dev-*` crate and contains no platform-specific code.

## Toolchain

Toolchain channel and targets are pinned in `rust-toolchain.toml`:

- Channel: `stable`
- Pre-installed targets: `thumbv8m.main-none-eabihf`, `thumbv7em-none-eabihf`, `riscv32imac-unknown-none-elf`
- Components: `rust-src`, `rustfmt`, `llvm-tools-preview`, `clippy`

All three `dev-*` targets are installed automatically the first time `cargo` is
invoked inside this repo; no manual `rustup target add` is required.

`dev-imxrt`, `dev-mcxa`, and `dev-npcx` link via [`flip-link`](https://github.com/knurling-rs/flip-link)
for stack-overflow protection. Install it once:

```
cargo install flip-link --locked
```

## Build

Build and lint a single platform:

```
cd platform/<name>
cargo build --locked
cargo clippy --locked -- -D warnings
```

For example, to build `dev-qemu`:

```
cd platform/dev-qemu
cargo build --locked
```

Format checks are run per crate:

```
cd platform/<name>
cargo fmt --check
```

Dependency policy (licenses, sources, advisories) is enforced by
[cargo-deny](https://github.com/EmbarkStudios/cargo-deny) using `deny.toml`:

```
cd platform/<name>
cargo deny --locked check
```

### Full local quality gate

`scripts/check-all.sh` runs every gate (fmt + build + clippy -D warnings +
cargo-deny) across all three dev-* platforms — the same checks CI runs:

```
bash scripts/check-all.sh
```

It installs `flip-link` on demand and skips `cargo-deny` gracefully if it
isn't installed. Run it before opening a PR to reproduce CI locally.

### Troubleshooting

- **`linker 'flip-link' not found`** — building `dev-imxrt` or `dev-npcx`
  requires `flip-link`. Install with `cargo install flip-link --locked`.
- **`unsupported target` from `semihosting`** — make sure you're running
  `cargo` from inside a `platform/<name>/` directory so `.cargo/config.toml`
  is picked up (it sets the build target). Building with
  `--manifest-path` from the repo root bypasses the per-platform target
  config and will fail for `dev-qemu`.
- **`can't find crate for 'core'`** — the toolchain file targets didn't
  install. Run `rustup show` from the repo root to force installation, or
  `rustup target add <target>` explicitly.

## Continuous Integration

CI lives under `.github/workflows/`:

- `check.yml` — per-platform fmt, clippy, build, doc, cargo-hack, cargo-deny,
  cargo-machete, and msrv across the public matrix (`dev-imxrt`, `dev-npcx`, `dev-qemu`).
- `nostd.yml` — verifies `platform-common` and `dev-*` remain `no_std`-clean.
- `benchmark.yml`, `rolling.yml` — secondary lanes (binsize tracking on
  `dev-imxrt`/`dev-npcx`; nightly dep update verification).

## Contributing

Code review ownership is defined in `CODEOWNERS`. Please open issues and PRs
against `main`.

## License

Licensed under the MIT License. See [LICENSE](./LICENSE).
