# dev-qemu
A platform targeting QEMU RISCV using mock embedded-services.

It runs on the custom ODP `ec` machine, which exposes the EC's I2C-target and GPIO lines as
sockets that external programs (such as another QEMU instance) can connect to. UART uses a PTY by
default and can instead use a stable socket for QEMU co-simulation.

## Prerequisites
- [Docker](https://docs.docker.com/get-docker/) — `qemu-ec.sh` pulls a prebuilt
  `qemu-system-riscv32` (with `ec` machine support) from the
  [`odp-qemu-builder`](https://github.com/OpenDevicePartnership/odp-qemu-builder)
  GHCR image and caches it under `target/qemu-ec/<tag>/`. To use a local QEMU
  instead, set `QEMU=/path/to/qemu-system-riscv32`.
- QEMU's runtime libraries (Ubuntu: `sudo apt install libfdt1 libjpeg-turbo8 libpng16-16t64`).
- `defmt-print`: `cargo install defmt-print`

## Run
`cargo run --release`

On first run the QEMU binary is pulled from GHCR. The PTY virtual serial port
path is displayed, and this can be used to connect over serial.

E.g. to connect with [ec-test-app](https://github.com/OpenDevicePartnership/odp-platform-common/tree/main/ec-test-app) built with the `serial` feature:
`./ec-test-app /dev/pts/<N> none`

To run without logging (skips `defmt-print`):
`cargo run-headless`

## CPU retention-wake fixture

Build with `ODP_WAKE_SOURCE=ac cargo build --locked --release --features time-alarm-wake`
or select `dc` explicitly. The feature requires one of those build-time
values; a missing or invalid selection fails at startup rather than assuming
a power source. Both alarms are disabled and the source is selected before
the TimeAlarm runner starts. Ordinary builds retain their initialization policy.

This test-only source selection is not hardware AC/DC detection. GPIO1 is an
active-high wake-request level driven by the real TimeAlarm service; GPIO0
remains the HID interrupt. HID-over-I2C and UART/MCTP continue to share that
same TimeAlarm service. Clear or disable the corresponding timers through
the existing relay commands to release their wake latches. The CPU retention
test harness connects the separate `ec-gpio1` channel to the host GPIO1 and
verifies actual standby return; the EC log alone does not prove host resume.

## Emulated runtime AC/DC input

Build with `--features time-alarm-power-input` to take the power source from
GPIO2 (high = AC, low = DC) instead of `ODP_WAKE_SOURCE`. `cargo run` starts it
on DC (`EC_POWER_SOURCE=ac` for AC); switch it by writing a raw byte to the
control socket:

```sh
cargo run --release --features time-alarm-power-input
printf '\001' | nc -NU /tmp/qemu-ec-power.sock   # AC
printf '\000' | nc -NU /tmp/qemu-ec-power.sock   # DC
```

Sending AC sets TimeAlarm's source to AC and connects a mock PSU (a 20 V / 3 A
wall adapter) to the power policy, so battery 0's `_PSR` reports online about
0.8 s later. Sending DC reverses both.

QEMU's `gpio2 source control disconnected; retaining input level N` warning is
expected at startup and after each write; `N` is the current level.

## Sockets
While `dev-qemu` is running, the `ec` machine exposes these sockets that external
programs (such as another QEMU instance) can connect to:

- I2C target: `/tmp/qemu-ec-i2c.sock`
- GPIO: `/tmp/qemu-ec-gpio.sock`
- GPIO2 AC/DC control (power-input builds): `/tmp/qemu-ec-power.sock`

Set `EC_UART_SOCK` to replace the UART PTY with another socket, for example
`EC_UART_SOCK=/tmp/qemu-ec-uart.sock cargo run --release`.

## Configuration
`qemu-ec.sh` reads the following environment variables:

| Variable          | Default                   | Description                                     |
|-------------------|---------------------------|-------------------------------------------------|
| `QEMU`            | (pulled from GHCR)        | Override the `qemu-system-riscv32` binary.      |
| `ODP_QEMU_TAG`    | (pinned in `qemu-ec.sh`)  | Tag of the odp-qemu-builder GHCR image to pull. |
| `EC_I2C_SOCK`     | `/tmp/qemu-ec-i2c.sock`   | Path for the I2C-target socket.                 |
| `EC_GPIO_SOCK`    | `/tmp/qemu-ec-gpio.sock`  | Path for the GPIO socket.                       |
| `EC_UART_SOCK`    | (unset)                   | UART socket path; when unset, use a PTY.        |
| `EC_POWER_SOURCE` | `dc` (power-input builds) | Initial GPIO2 level: `ac` or `dc`.              |
| `EC_POWER_SOCK`   | `/tmp/qemu-ec-power.sock` | Path for the GPIO2 control socket.              |
