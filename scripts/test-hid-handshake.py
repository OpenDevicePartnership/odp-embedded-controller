#!/usr/bin/env python3
"""Exercise the dev-qemu HID startup contract over the EC's real sockets."""

import argparse
from pathlib import Path
import select
import socket
import struct
import subprocess
import tempfile


REPORT_DESCRIPTOR = bytes.fromhex(
    "06 00 ff 09 01 a1 01 09 01 15 00 26 ff 00 75 08 95 01 81 02 c0"
)
DEVICE_DESCRIPTOR = struct.pack(
    "<15H", 30, 0x0100, len(REPORT_DESCRIPTOR), 2, 3, 3, 4, 2, 5, 6,
    0x045E, 0x0002, 0x0100, 0, 0,
)


def receive(sock, count):
    data = bytearray()
    while len(data) < count:
        chunk = sock.recv(count - len(data))
        if not chunk:
            raise EOFError("QEMU closed the socket")
        data.extend(chunk)
    return bytes(data)


def no_gpio_edge(gpio):
    assert not select.select([gpio], [], [], 0.05)[0], "Unexpected GPIO0 transition"


class I2cHost:
    # odp-i2c-target socket opcodes: START=42, STOP=77, DATA=da, ACK=01, NAK=00.
    def __init__(self, sock):
        self.sock = sock

    def start(self, read=False):
        self.sock.sendall(bytes([0x42, (0x2C << 1) | read]))
        assert receive(self.sock, 1) == b"\x01", "I2C address not acknowledged"

    def write(self, data, stop=True):
        self.start()
        for byte in data:
            self.sock.sendall(bytes([0xDA, byte]))
            assert receive(self.sock, 1) == b"\x01", "I2C data not acknowledged"
        if stop:
            self.sock.sendall(b"\x77")

    def read(self, count, finish=True):
        self.start(read=True)
        data = bytearray()
        for index in range(count):
            opcode, byte = receive(self.sock, 2)
            assert opcode == 0xDA, f"Unexpected I2C opcode: {opcode:#x}"
            data.append(byte)
            if index + 1 < count:
                self.sock.sendall(b"\x01")
        if finish:
            self.finish_read()
        return bytes(data)

    def finish_read(self):
        self.sock.sendall(b"\x00\x77")

    def register(self, register, count):
        self.write(struct.pack("<H", register), stop=False)
        return self.read(count)


def handshake(i2c, gpio):
    assert receive(gpio, 1) == b"\x01", "GPIO0 must start deasserted"
    assert i2c.register(1, 30) == DEVICE_DESCRIPTOR
    no_gpio_edge(gpio)

    i2c.write(bytes.fromhex("05 00 00 08"))  # SET_POWER On
    no_gpio_edge(gpio)
    i2c.write(bytes.fromhex("05 00 00 01"))  # RESET
    assert receive(gpio, 1) == b"\x00", "RESET must assert GPIO0"
    assert i2c.register(1, 30) == DEVICE_DESCRIPTOR
    assert i2c.register(2, len(REPORT_DESCRIPTOR)) == REPORT_DESCRIPTOR
    no_gpio_edge(gpio)

    # Keep the final byte unacknowledged: GPIO0 belongs to the whole transfer.
    assert i2c.read(3, finish=False) == b"\x00\x00\x00"
    no_gpio_edge(gpio)
    i2c.finish_read()
    assert receive(gpio, 1) == b"\x01", "Reset sentinel must release GPIO0"

    i2c.write(bytes.fromhex("05 00 00 01"))
    assert receive(gpio, 1) == b"\x00"
    assert i2c.register(3, 2) == b"\x00\x00"
    assert receive(gpio, 1) == b"\x01"
    assert i2c.read(3) == b"\x00\x00\x00"

    i2c.write(bytes.fromhex("05 00 10 02 06 00"), stop=False)  # GET_REPORT Input, ID 0
    assert i2c.read(3) == b"\x03\x00\x00"
    i2c.write(bytes.fromhex("05 00 01 08"))  # SET_POWER Sleep
    no_gpio_edge(gpio)
    i2c.write(bytes.fromhex("05 00 00 08"))
    no_gpio_edge(gpio)
    assert i2c.register(2, len(REPORT_DESCRIPTOR)) == REPORT_DESCRIPTOR
    no_gpio_edge(gpio)

    # No inter-command wait: STOP and the next RX byte can be latched together.
    i2c.write(bytes.fromhex("05 00 00 08"))
    i2c.write(bytes.fromhex("05 00 00 01"))
    assert receive(gpio, 1) == b"\x00", "Back-to-back RESET must assert GPIO0"
    assert i2c.read(3) == b"\x00\x00\x00"
    assert receive(gpio, 1) == b"\x01", "Reset sentinel must release GPIO0"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("qemu", help="qemu-system-riscv32 with the ODP ec machine")
    parser.add_argument("elf", type=Path, help="built dev-qemu ELF")
    args = parser.parse_args()

    with tempfile.TemporaryDirectory(prefix="ec-hid-") as directory:
        with socket.socket(socket.AF_UNIX) as i2c_server, socket.socket(socket.AF_UNIX) as gpio_server:
            for server, name in [(i2c_server, "i2c"), (gpio_server, "gpio")]:
                server.bind(f"{directory}/{name}")
                server.listen(1)
                server.settimeout(10)
            # Connect QEMU to listening peers before boot so GPIO0's initial level isn't lost.
            with subprocess.Popen(
                [
                    args.qemu, "-machine", "ec", "-bios", "none", "-nographic",
                    "-monitor", "none", "-serial", "null",
                    "-semihosting-config", "enable=on,target=native",
                    "-chardev", f"socket,id=ec-i2c-target,path={directory}/i2c",
                    "-chardev", f"socket,id=ec-gpio0,path={directory}/gpio",
                    "-kernel", str(args.elf.resolve()),
                ],
                stdout=subprocess.DEVNULL,
            ) as process:
                try:
                    with i2c_server.accept()[0] as i2c, gpio_server.accept()[0] as gpio:
                        i2c.settimeout(3)
                        gpio.settimeout(3)
                        handshake(I2cHost(i2c), gpio)
                        assert process.poll() is None, "Firmware exited during handshake"
                finally:
                    process.terminate()
                    process.wait(timeout=5)
    print("PASS: HID identity, descriptors, reset/read, GPIO0 lifetime, GET_REPORT, SET_POWER")


if __name__ == "__main__":
    main()
