#!/usr/bin/env bash
#
# Unified QEMU launcher for `dev-qemu`.
#
# This is the single source of truth used by `cargo run`, `cargo run-headless`,
# `scripts/integration-test.sh`, and CI. It:
#
#   1. Resolves a `qemu-system-riscv32` binary that supports the `ec` machine
#      (GPIO + I2C target sockets). The binary is pulled once from the
#      `odp-qemu-builder` image published on GHCR and cached under `target/`.
#   2. Launches QEMU on the `ec` machine, exposing the EC's I2C-target and GPIO
#      lines as UNIX-domain sockets that external programs can connect to, plus
#      a PTY for the UART.
#   3. Routes the defmt log stream: in the normal (interactive) path the
#      semihosting output is piped straight into `defmt-print`; in the headless
#      path (`DEFMT_LOG=off`) QEMU is run raw with no logging.
#
# Invoked by cargo as: `./qemu-ec.sh <PATH_TO_ELF>`.
#
# It can also be invoked as `./qemu-ec.sh --prepare` to only resolve (and, on
# first use, pull) the QEMU binary into the cache and exit, without launching.
# This is useful for warming the cache before a time-sensitive launch so a cold
# `docker pull` doesn't count against a startup timeout.
#
# Environment knobs (all optional):
#   QEMU          Override the QEMU binary entirely (skips the GHCR pull).
#   ODP_QEMU_TAG  Tag of the GHCR image to pull.
#   EC_I2C_SOCK   Path for the I2C-target socket (default: /tmp/qemu-ec-i2c.sock).
#   EC_GPIO_SOCK  Path for the GPIO socket (default: /tmp/qemu-ec-gpio.sock).

set -euo pipefail

MODE="run"
if [[ "${1:-}" == "--prepare" ]]; then
    MODE="prepare"
    ELF=""
else
    ELF="${1:?usage: qemu-ec.sh <elf> | qemu-ec.sh --prepare}"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ODP_QEMU_TAG="${ODP_QEMU_TAG:-sha-7e461b3}"
EC_I2C_SOCK="${EC_I2C_SOCK:-/tmp/qemu-ec-i2c.sock}"
EC_GPIO_SOCK="${EC_GPIO_SOCK:-/tmp/qemu-ec-gpio.sock}"

# GHCR image that publishes the prebuilt QEMU (with `ec` machine support).
QEMU_IMAGE="ghcr.io/opendevicepartnership/odp-qemu-builder/qemu:${ODP_QEMU_TAG}"
# Location of the binary inside that image.
QEMU_IMAGE_BIN="/usr/local/bin/qemu-system-riscv32"
# Where we cache the extracted binary on the host.
QEMU_CACHE_DIR="${SCRIPT_DIR}/target/qemu-ec"
QEMU_CACHE_BIN="${QEMU_CACHE_DIR}/qemu-system-riscv32"

# Resolve the QEMU binary, pulling it from GHCR on first use.
resolve_qemu() {
    # 1. Explicit override.
    if [[ -n "${QEMU:-}" ]]; then
        if [[ ! -x "$QEMU" ]]; then
            echo "error: \$QEMU is set to '$QEMU' but it is not executable" >&2
            exit 1
        fi
        echo "$QEMU"
        return
    fi

    # 2. Previously cached binary.
    if [[ -x "$QEMU_CACHE_BIN" ]]; then
        echo "$QEMU_CACHE_BIN"
        return
    fi

    # 3. Pull from the GHCR image.
    if ! command -v docker >/dev/null 2>&1; then
        echo "error: docker is required to fetch qemu-system-riscv32 from ${QEMU_IMAGE}" >&2
        echo "       (install docker, or set \$QEMU to a local qemu-system-riscv32 with 'ec' machine support)" >&2
        exit 1
    fi

    echo "Pulling QEMU from ${QEMU_IMAGE}..." >&2
    docker pull "$QEMU_IMAGE" >&2

    mkdir -p "$QEMU_CACHE_DIR"
    local cid
    cid="$(docker create "$QEMU_IMAGE")"
    # shellcheck disable=SC2064
    trap "docker rm -f '$cid' >/dev/null 2>&1 || true" RETURN
    docker cp "${cid}:${QEMU_IMAGE_BIN}" "$QEMU_CACHE_BIN" >&2
    chmod +x "$QEMU_CACHE_BIN"

    echo "$QEMU_CACHE_BIN"
}

QEMU_BIN="$(resolve_qemu)"

# `--prepare` only warms the cache; report the resolved binary and exit.
if [[ "$MODE" == "prepare" ]]; then
    echo "QEMU ready: $QEMU_BIN" >&2
    exit 0
fi

# QEMU arguments shared by both the interactive and headless paths.
#
# - `-machine ec`            EC board exposing the I2C-target and GPIO sockets.
# - `-bios none`             dev-qemu is a bare-metal kernel; no firmware needed.
# - `-serial pty`            UART0 is bridged to a PTY for terminal/ec-test-cli.
# - `-chardev socket,...`    The I2C-target and GPIO lines as UNIX sockets that
#                            external programs can connect to (server=on).
# - `-semihosting-config`    Routes defmt over semihosting to QEMU's own stdout
#                            (`target=native`) so it stays separate from UART0.
QEMU_ARGS=(
    -machine ec
    -bios none
    -nographic
    -monitor none
    -semihosting-config enable=on,target=native
    -serial pty
    -chardev "socket,id=ec-i2c-target,path=${EC_I2C_SOCK},server=on,wait=off"
    -chardev "socket,id=ec-gpio0,path=${EC_GPIO_SOCK},server=on,wait=off"
    -kernel "$ELF"
)

# Headless path: no defmt logging, run QEMU raw. The "char device redirected to
# /dev/pts/N" line goes to stdout where callers (integration-test.sh) grep it.
if [[ "${DEFMT_LOG:-}" == "off" ]]; then
    exec "$QEMU_BIN" "${QEMU_ARGS[@]}"
fi

# Interactive path: defmt-print decodes the semihosting log stream.
if ! command -v defmt-print >/dev/null 2>&1; then
    echo "error: defmt-print is required for the interactive run path" >&2
    echo "       install it with: cargo install defmt-print" >&2
    echo "       (or use 'cargo run-headless' to disable logging)" >&2
    exit 1
fi

# With `-serial pty`, QEMU prints a single "char device redirected to
# /dev/pts/N (label serial0)" line on stdout before any semihosting data. Peel
# that first line off to stderr (so the PTY path stays visible) and feed the
# remaining bytes to defmt-print.
"$QEMU_BIN" "${QEMU_ARGS[@]}" | {
    IFS= read -r ptsline
    printf '%s\n' "$ptsline" >&2
    defmt-print -e "$ELF"
}
