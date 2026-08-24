# Secure EC Overview

The Open Device Partnership (ODP) Secure Embedded Controller (Secure EC)
project strives to build an open-source, microcontroller-agnostic embedded controller
platform for the PC ecosystem. It combines secure firmware, standardized host
interfaces, and reusable Rust components so device makers can reduce effort by building
on a standardized platform that is secure by design and focusing on their unique value added.

## Goals

- Security by default leveraging memory-safe Rust
- Standardize communication between the host operating system and the EC across
  hardware architectures and operating systems.
- Provide reusable and customizable firmware services for battery, thermal, power, USB-C, HID,
  time alarms, and other common EC functions.
- Support collaboration among device makers, silicon vendors, operating-system
  vendors, and the open-source community.
- Reduce platform bring-up effort through shared abstractions, reference
  implementations, virtual platforms, and test tooling.

## Architecture

Secure EC separates portable service logic from platform-specific hardware
support:

1. **MCU HALs and board support** provide access to clocks, GPIO, I2C, eSPI,
   I3C, and other peripherals.
2. **Subsystem abstractions** connect hardware implementations to subsystem abstractions for battery, thermal, power, USB-C, HID, and other common EC subsystems.
3. **Embedded services** implement reference application logic for EC subsystems.
4. **Host transports and protocols** expose those services to the host through standardized
   interfaces

## Security Direction

The project is working toward an EC that is an attestable platform component
rather than an implicitly trusted boundary:

- **Secure boot** verifies that firmware is authorized before execution.
- **DICE** derives device identity from hardware secrets and firmware
  measurements.
- **Attestation** allows host OS to assess the identity and state of the
  EC.
- **Encrypted communication** provides a standardized mechanism for secure data exchange between the EC and the host OS.

## Current Work

Current areas of investment include:

- Expanding portable Rust services and drivers.
- Supporting reference MCU platforms from multiple silicon vendors.
- Developing standardized HID communication protocol over eSPI and I3C transport with OS support.
- Starting with secure boot and working toward device identity + attestation.
- Using QEMU-based virtualized platforms for development, integration testing, and ecosystem
  onboarding.
- Exploring how Rust based Secure EC components and APIs can interoperate with Zephyr-based
  systems.

## Key Repositories

| Repository | Purpose |
| ---------- | ------- |
| [`odp-embedded-controller`](https://github.com/OpenDevicePartnership/odp-embedded-controller) | Reference EC firmware including hardware platforms and virtualized platforms. Refer to its README to get started. Secure EC top-level documentation is also included. |
| [`odp-platform-qemu-arm-virt`](https://github.com/OpenDevicePartnership/odp-platform-qemu-arm-virt) | End to End virtualized Secure EC testing platform using virtualized EC RISC-V platform <-> QEMU ARM virtual platform running Windows Validation OS. |
| [`embedded-services`](https://github.com/OpenDevicePartnership/embedded-services) | Reusable EC service implementations and interfaces, including battery, thermal, HID, time and alarms, and UART services. |

The broader set of projects and community activity is available in the
[Open Device Partnership GitHub organization](https://github.com/OpenDevicePartnership).
