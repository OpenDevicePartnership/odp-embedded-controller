# dev-qemu
A platform targeting QEMU RISCV using mock embedded-services.

It runs on the custom ODP `ec` machine, which exposes the EC's I2C-target and GPIO lines as
sockets that external programs (such as another QEMU instance) can connect to, alongside a PTY for
the UART.

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

## Sockets
While `dev-qemu` is running, the `ec` machine exposes two sockets that external
programs (such as another QEMU instance) can connect to:

- I2C target: `/tmp/qemu-ec-i2c.sock`
- GPIO: `/tmp/qemu-ec-gpio.sock`

## Configuration
`qemu-ec.sh` reads the following environment variables:

| Variable       | Default                  | Description                                     |
|----------------|--------------------------|-------------------------------------------------|
| `QEMU`         | (pulled from GHCR)       | Override the `qemu-system-riscv32` binary.      |
| `ODP_QEMU_TAG` | (pinned in `qemu-ec.sh`) | Tag of the odp-qemu-builder GHCR image to pull. |
| `EC_I2C_SOCK`  | `/tmp/qemu-ec-i2c.sock`  | Path for the I2C-target socket.                 |
| `EC_GPIO_SOCK` | `/tmp/qemu-ec-gpio.sock` | Path for the GPIO socket.                       |
