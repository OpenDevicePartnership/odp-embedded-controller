#!/usr/bin/env bash
#
# Unified QEMU launcher for `dev-qemu`.
#
# This is the single source of truth used by `cargo run`, `cargo run-headless`,
# `scripts/integration-test.sh`, and CI. It:
#
#   1. Resolves a `qemu-system-riscv32` binary that supports the `ec` machine
#      (GPIO + I2C target sockets). The binary is pulled once per image tag from
#      the `odp-qemu-builder` image published on GHCR and cached under
#      `target/qemu-ec/<tag>/`.
#   2. Launches QEMU on the `ec` machine, exposing the EC's I2C-target and GPIO
#      lines as UNIX-domain sockets that external programs can connect to. UART
#      uses a PTY by default or a stable socket when EC_UART_SOCK is set. When
#      GPIO2 is initialized as the AC/DC input (see EC_POWER_SOURCE), it also
#      gets its own control socket.
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
#   EC_UART_SOCK  Optional path for a UART socket (default: unset, use a PTY).
#   EC_POWER_SOURCE
#                 Initial GPIO2 AC/DC input, `ac` or `dc`. Defaults to `dc` for
#                 firmware built with `time-alarm-power-input` (build.rs marks
#                 it); otherwise unset, leaving GPIO2 uninitialized.
#   EC_POWER_SOCK Path for the GPIO2 control socket when GPIO2 is initialized
#                 (default: /tmp/qemu-ec-power.sock). Write the raw byte 0x01
#                 for AC or 0x00 for DC.

set -euo pipefail

MODE="run"
if [[ "${1:-}" == "--prepare" ]]; then
    MODE="prepare"
    ELF=""
else
    ELF="${1:?usage: qemu-ec.sh <elf> | qemu-ec.sh --prepare}"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ODP_QEMU_TAG="${ODP_QEMU_TAG:-sha-f8af056}"
EC_I2C_SOCK="${EC_I2C_SOCK:-/tmp/qemu-ec-i2c.sock}"
EC_GPIO_SOCK="${EC_GPIO_SOCK:-/tmp/qemu-ec-gpio.sock}"
EC_UART_SOCK="${EC_UART_SOCK:-}"
EC_POWER_SOURCE="${EC_POWER_SOURCE:-}"
EC_POWER_SOCK="${EC_POWER_SOCK:-/tmp/qemu-ec-power.sock}"

# Firmware built with `time-alarm-power-input` stops at startup unless GPIO2 is
# initialized. Its build.rs leaves this marker symbol, so start it on DC.
if [[ -z "$EC_POWER_SOURCE" && -n "$ELF" ]] && grep -qaF __odp_qemu_ec_power_input "$ELF"; then
    EC_POWER_SOURCE=dc
fi

# GPIO2 (bit 2) initial level: high is AC, low is DC.
case "$EC_POWER_SOURCE" in
    "") ;;
    ac) EC_POWER_LEVEL=0x4 ;;
    dc) EC_POWER_LEVEL=0x0 ;;
    *)
        echo "error: EC_POWER_SOURCE must be 'ac' or 'dc', got '$EC_POWER_SOURCE'" >&2
        exit 1
        ;;
esac

# GHCR image that publishes the prebuilt QEMU (with `ec` machine support).
QEMU_IMAGE="ghcr.io/opendevicepartnership/odp-qemu-builder/qemu:${ODP_QEMU_TAG}"
# Location of the binary inside that image.
QEMU_IMAGE_BIN="/usr/local/bin/qemu-system-riscv32"
# Where we cache the extracted binary on the host. Caching per tag means bumping
# ODP_QEMU_TAG pulls the new binary instead of reusing an older one.
QEMU_CACHE_DIR="${SCRIPT_DIR}/target/qemu-ec/${ODP_QEMU_TAG}"
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
    # `resolve_qemu` runs in a command-substitution subshell, so an EXIT trap is
    # scoped to that subshell and fires on both success and an errexit abort
    # (e.g. a failing `docker cp`). RETURN would be skipped on errexit, leaking
    # the temporary container.
    # shellcheck disable=SC2064
    trap "docker rm -f '$cid' >/dev/null 2>&1 || true" EXIT
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
# - `-serial pty`            By default, UART0 uses a PTY for terminal/ec-test-cli.
# - `-chardev socket,...`    The I2C-target, GPIO, and optional UART lines as UNIX
#                            sockets that external programs can connect to.
# - `-global odp-gpio.*`     With EC_POWER_SOURCE, initialize GPIO2 (the AC/DC input)
#                            for its control socket, which QEMU otherwise rejects.
QEMU_ARGS=(
    -machine ec
    -bios none
    -nographic
    -monitor none
)

if [[ -n "$EC_UART_SOCK" ]]; then
    if [[ -e "$EC_UART_SOCK" && ! -S "$EC_UART_SOCK" ]]; then
        echo "error: EC_UART_SOCK points to an existing non-socket: $EC_UART_SOCK" >&2
        exit 1
    fi
    rm -f -- "$EC_UART_SOCK"
    QEMU_ARGS+=(
        -chardev "socket,id=ec-uart0,path=${EC_UART_SOCK},server=on,wait=off"
        -serial "chardev:ec-uart0"
    )
else
    QEMU_ARGS+=(-serial pty)
fi

QEMU_ARGS+=(
    -chardev "socket,id=ec-i2c-target,path=${EC_I2C_SOCK},server=on,wait=off"
    -chardev "socket,id=ec-gpio0,path=${EC_GPIO_SOCK},server=on,wait=off"
    -kernel "$ELF"
)

if [[ -n "$EC_POWER_SOURCE" ]]; then
    QEMU_ARGS+=(
        -global odp-gpio.input-reset-mask=0x4
        -global "odp-gpio.input-reset=${EC_POWER_LEVEL}"
        -chardev "socket,id=ec-gpio2,path=${EC_POWER_SOCK},server=on,wait=off"
    )
    echo "GPIO2 power input starts on ${EC_POWER_SOURCE}; write 0x01 (AC) or 0x00 (DC) to ${EC_POWER_SOCK}" >&2
    echo "Expected: QEMU warns \"gpio2 source control disconnected; retaining input level N\" at startup" \
        "and after each client disconnects (N: 1 = AC, 0 = DC)." >&2
fi

# Headless path: no defmt logging, so semihosting is left disabled and QEMU runs
# raw. The "char device redirected to /dev/pts/N" line goes to stdout where
# callers (integration-test.sh) grep it.
if [[ "${DEFMT_LOG:-}" == "off" ]]; then
    if [[ -n "$EC_UART_SOCK" ]]; then
        printf 'char device redirected to %s (label serial0)\n' "$EC_UART_SOCK"
    fi
    exec "$QEMU_BIN" "${QEMU_ARGS[@]}"
fi

# Interactive path enables semihosting so defmt can route its log stream to
# QEMU's own stdout (`target=native`), keeping it separate from UART0.
QEMU_ARGS+=(-semihosting-config enable=on,target=native)

# Interactive path: defmt-print decodes the semihosting log stream.
if ! command -v defmt-print >/dev/null 2>&1; then
    echo "error: defmt-print is required for the interactive run path" >&2
    echo "       install it with: cargo install defmt-print" >&2
    echo "       (or use 'cargo run-headless' to disable logging)" >&2
    exit 1
fi

if [[ -n "$EC_UART_SOCK" ]]; then
    printf 'char device redirected to %s (label serial0)\n' "$EC_UART_SOCK" >&2
    "$QEMU_BIN" "${QEMU_ARGS[@]}" | defmt-print -e "$ELF"
else
    # With `-serial pty`, QEMU prints one announcement line before any
    # semihosting data. Keep the PTY visible and decode the remaining bytes.
    "$QEMU_BIN" "${QEMU_ARGS[@]}" | {
        IFS= read -r ptsline
        printf '%s\n' "$ptsline" >&2
        defmt-print -e "$ELF"
    }
fi
