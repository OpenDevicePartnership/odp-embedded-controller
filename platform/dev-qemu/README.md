# dev-qemu
A platform targeting QEMU RISCV using mock embedded-services.

It runs on the custom ODP `ec` machine, which exposes the EC's I2C-target, GPIO, and eSPI interfaces as
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

## Sockets
While `dev-qemu` is running, the `ec` machine exposes three sockets that external
programs (such as another QEMU instance) can connect to:

- I2C target: `/tmp/qemu-ec-i2c.sock`
- GPIO: `/tmp/qemu-ec-gpio.sock`
- eSPI target: `/tmp/qemu-ec-espi.sock`

Set `EC_UART_SOCK` to replace the UART PTY with another socket, for example
`EC_UART_SOCK=/tmp/qemu-ec-uart.sock cargo run --release`.

## PCC PING/PONG

The firmware responds to command `1` on eSPI mailbox 0 (PCC Type 3 subspace 0).
Start the ARM host and EC QEMU instances with the same `EC_ESPI_SOCK`, using
QEMU builds that support the mailbox-only eSPI interface and publish PCCT.
In the Windows ARM64 guest, use `ec-test-cli` from `odp-platform-common`:

```text
ec-test-cli --source acpi pcc probe
ec-test-cli --source acpi pcc ping --sequence 42
```

This requires a Windows ARM64 image with the native-PCC-enabled `ectest` driver.
Probe queries interface metadata only; ping must print
`PCC response: PONG sequence=42` and exit successfully.

The fixed eight-byte payload is ASCII `PING` followed by a little-endian `u32`
sequence; the reply is `PONG` followed by the same sequence. It starts at mailbox
offset 16, after the extended PCC header. Request PCC Length is not used because
the inspected Windows provider leaves it unset; response Length is 12 bytes
(command plus payload). Unsupported commands or markers complete with an error.
Mailbox 0 starts with command-complete set so Windows can acquire the idle
channel before sending its first command. No Type 4 notification is implemented.

## Configuration
`qemu-ec.sh` reads the following environment variables:

| Variable       | Default                  | Description                                     |
|----------------|--------------------------|-------------------------------------------------|
| `QEMU`         | (pulled from GHCR)       | Override the `qemu-system-riscv32` binary.      |
| `ODP_QEMU_TAG` | (pinned in `qemu-ec.sh`) | Tag of the odp-qemu-builder GHCR image to pull. |
| `EC_I2C_SOCK`  | `/tmp/qemu-ec-i2c.sock`  | Path for the I2C-target socket.                 |
| `EC_GPIO_SOCK` | `/tmp/qemu-ec-gpio.sock` | Path for the GPIO socket.                       |
| `EC_UART_SOCK` | (unset)                  | UART socket path; when unset, use a PTY.        |
| `EC_ESPI_SOCK` | `/tmp/qemu-ec-espi.sock` | eSPI target socket path; set empty to disable. |
