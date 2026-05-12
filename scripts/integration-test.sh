#!/usr/bin/env bash
#
# This script spawns an instance of `dev-qemu` and runs the `ec-test-cli` commands against it.
# Mainly used within CI, but can be run locally assuming `ec-test-cli` is installed with:
# `cargo install --git https://github.com/OpenDevicePartnership/odp-platform-common ec-test-cli --locked`
#
# Currently this only checks the status code of the commands, not the actual returned result.
# So this won't catch if requests are responding successfully but with garbage data.
# Will need to add support to `ec-test-cli` to accept a range of expected outputs to check against
# for that.

# Causes script to exit non-zero on the first test failure,
# which will then cause entire script to return non-zero and fail CI
set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel)

# Dump all `dev-qemu` output here so we can extract info or print it on failure
QEMU_LOG=$(mktemp)
QEMU_PID=""

# The dev-qemu process is launched in the background, so we need to ensure it gets killed when this
# script exits regardless of whether it succeeded or failed
cleanup() {
    # Print the dev-qemu log if we encountered error
    if [[ $? -ne 0 && -f "$QEMU_LOG" ]]; then
        echo "dev-qemu output:"
        cat "$QEMU_LOG"
    fi
    if [[ -n "$QEMU_PID" ]]; then
        kill "$QEMU_PID" 2>/dev/null || true
        wait "$QEMU_PID" 2>/dev/null || true
    fi
    rm -f "$QEMU_LOG"
}
trap cleanup EXIT

# Build `dev-qemu` (with DEFMT disabled since we don't need logs)
cd "$REPO_ROOT/platform/dev-qemu"
echo "Building dev-qemu..."
cargo build --locked --release --config 'env.DEFMT_LOG="off"'

# Then launch it in "headless mode" (again, DEFMT disabled),
# and poll until serial comms are ready
echo "Starting dev-qemu..."
cargo run-headless --locked > "$QEMU_LOG" 2>&1 &
QEMU_PID=$!

# Poll for the PTY path to appear in dev-qemu output, indicating serial is ready
# Note: We do this poll-sleep loop in case the CI is under heavy load
MAX_RETRIES=10
WAIT_INTERVAL=1
PTY_PATH=""
for i in $(seq 1 $MAX_RETRIES); do
    if ! kill -0 "$QEMU_PID" 2>/dev/null; then
        echo "ERROR: dev-qemu exited unexpectedly"
        exit 1
    fi

    PTY_PATH=$(grep -oE '/dev/pts/[0-9]+' "$QEMU_LOG" || true)
    if [[ -n "$PTY_PATH" ]]; then
        break
    fi

    echo "Waiting for dev-qemu to be ready... ($i/$MAX_RETRIES)"
    sleep $WAIT_INTERVAL
done

if [[ -z "$PTY_PATH" ]]; then
    echo "ERROR: Could not find PTY path in dev-qemu output after ${MAX_RETRIES}s"
    exit 1
fi
echo "dev-qemu serial port: $PTY_PATH"
CLI="ec-test-cli --port $PTY_PATH"

# Helper to run a ec-test-cli request and print the result
run_test() {
    echo "$*"
    $CLI "$@"
    # Note: Sleep is needed since otherwise we seem to get a timeout error on next request
    sleep 0.01
}

# Thermal
echo "Running thermal tests..."
run_test thermal get-temperature
run_test thermal get-rpm
run_test thermal get-min-rpm
run_test thermal get-max-rpm
run_test thermal get-threshold on
run_test thermal get-threshold ramping
run_test thermal get-threshold max
run_test thermal set-rpm 1000

# Battery
echo "Running battery tests..."
run_test battery get-bst
run_test battery get-bix
run_test battery set-btp 50

# RTC
echo "Running RTC tests..."
run_test rtc get-capabilities
run_test rtc get-real-time
run_test rtc get-wake-status ac
run_test rtc get-wake-status dc
run_test rtc get-expired-timer-wake-policy ac
run_test rtc get-expired-timer-wake-policy dc
run_test rtc get-timer-value ac
run_test rtc get-timer-value dc

echo "SUCCESS: Integration test passed!"
