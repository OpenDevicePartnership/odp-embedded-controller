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
