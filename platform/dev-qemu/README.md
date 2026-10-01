# dev-qemu
A platform targeting QEMU RISCV using mock embedded-services.

It runs on the custom ODP `ec` machine, which exposes the EC's I2C-target and GPIO lines as
sockets that external programs (such as another QEMU instance) can connect to. UART uses a PTY by
default and can instead use a stable socket for QEMU co-simulation.

## Prerequisites
- [Docker](https://docs.docker.com/get-docker/) — `qemu-ec.sh` pulls a prebuilt
  `qemu-system-riscv32` (with `ec` machine support) from the
  [`odp-qemu-builder`](https://github.com/OpenDevicePartnership/odp-qemu-builder)
  GHCR image and caches it under `target/qemu-ec/`. To use a local QEMU instead,
  set `QEMU=/path/to/qemu-system-riscv32`.
- `defmt-print`: `cargo install defmt-print`

## Run
`cargo run --release`

On first run the QEMU binary is pulled from GHCR. The PTY virtual serial port
path is displayed, and this can be used to connect over serial.

E.g. to connect with [ec-test-app](https://github.com/OpenDevicePartnership/odp-platform-common/tree/main/ec-test-app) built with the `serial` feature:
`./ec-test-app /dev/pts/<N> none`

To run without logging (skips `defmt-print`):
`cargo run-headless`

## HID startup fixture

The mock `relay::hid::HidDevice` runs through `hidi2c-target-service` at I2C
address `0x2c`. It retains VID/PID `045e:0002`, version `0100`, and the
vendor-defined one-byte input report descriptor, without generating input
events. The transport owns active-low GPIO0: RESET asserts it until the
host completes its reset-sentinel read. SET_POWER remains a no-op.

`Cargo.lock` pins the service crates to published upstream `fddcad16`.
The current transport includes the two-byte length prefix in
`wMaxOutputLength` even with no output payload (`2`, formerly `0`).
GET_REPORT for input ID zero returns the correctly framed zero byte
(`03 00 00`); reset and idle input reads still return zeros.

To check the descriptors, reset handshake, and GPIO0 lifetime without Windows
(Python 3 standard library only):

```console
cargo build --locked --release
python3 ../../scripts/test-hid-handshake.py /path/to/qemu-system-riscv32 target/riscv32imac-unknown-none-elf/release/dev-qemu
```

The test boots its own EC instance with private sockets. It does not replace
Windows enumeration or full host wake integration testing. A back-to-back
SET_POWER/RESET pair also checks that the HAL preserves transaction boundaries
without an inter-command delay.

## CPU retention-wake fixture

Build with `ODP_WAKE_SOURCE=ac cargo build --release --features time-alarm-wake`
or select `dc` explicitly. The feature requires one of those build-time
values; a missing or invalid selection fails at startup rather than assuming
a power source. Ordinary builds are unchanged.

This test-only source selection is not hardware AC/DC detection. GPIO1 is an
active-high wake-request level driven by the real TimeAlarm service; GPIO0
remains the HID interrupt. Clear or disable the corresponding timers through
the existing relay commands to release their wake latches. The CPU retention
test harness connects the separate `ec-gpio1` channel to the host GPIO1 and
verifies actual standby return; the EC log alone does not prove host resume.

## Emulated runtime AC/DC input

`cargo build --release --features time-alarm-power-input` includes the wake
output and uses GPIO2 (high = AC, low = DC) instead of `ODP_WAKE_SOURCE`.
The input must be explicitly initialized by the QEMU model before startup;
the firmware rejects an unset validity bit rather than assuming DC. Source
selection happens before the TimeAlarm runner starts. Socket bytes alone do
not establish validity.

Use the GPIO2-enabled QEMU model with `input-reset-mask=4` and `input-reset=4`
for cold AC, or `input-reset=0` for cold DC, and its `ec-gpio2` control socket.
Raw bytes `01` and `00` select AC and DC at runtime. The dedicated input task
resamples after opposite-level waits on the shared GPIO interrupt. This is a
level interface, not an event counter: a transient that returns before the
task samples it is not a recorded power-source change.

The model retains the last valid source across control-socket disconnects
and EC warm resets, and diagnoses malformed bytes. GPIO1 remains the wake
output and GPIO0 remains the HID interrupt. The default build and the explicit
construction-time `time-alarm-wake` fixture are unchanged. This emulated
external input neither detects physical power nor reports Windows power state.

## Sockets
While `dev-qemu` is running, the `ec` machine exposes two sockets that external
programs (such as another QEMU instance) can connect to:

- I2C target: `/tmp/qemu-ec-i2c.sock`
- GPIO: `/tmp/qemu-ec-gpio.sock`

Set `EC_UART_SOCK` to replace the UART PTY with another socket, for example
`EC_UART_SOCK=/tmp/qemu-ec-uart.sock cargo run --release`.

## Configuration
`qemu-ec.sh` reads the following environment variables:

| Variable       | Default                  | Description                                     |
|----------------|--------------------------|-------------------------------------------------|
| `QEMU`         | (pulled from GHCR)       | Override the `qemu-system-riscv32` binary.      |
| `ODP_QEMU_TAG` | (pinned in `qemu-ec.sh`) | Tag of the odp-qemu-builder GHCR image to pull. |
| `EC_I2C_SOCK`  | `/tmp/qemu-ec-i2c.sock`  | Path for the I2C-target socket.                 |
| `EC_GPIO_SOCK` | `/tmp/qemu-ec-gpio.sock` | Path for the GPIO socket.                       |
| `EC_UART_SOCK` | (unset)                  | UART socket path; when unset, use a PTY.        |
