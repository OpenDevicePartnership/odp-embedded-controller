# HID EC Interface

This section is a collection of the HID definitions for each HID class driver in the OS that is loaded on a Top Level Collection. 

Today HIDClass, HIDI2C and HIDI3C are all inboxed drivers in the OS and support input devices. Here we outline the specs for the remaining missing pieces for all the HID class drivers and HIDeSPI.

## HID Transport Drivers

HID supports I3C inbox in the OS without requiring any vendor or OEM drivers to be installed if the HID device is HCI compliant. For I2C and SPI drivers the stack is also present, but requires a Silicon Vendor driver for the underlying I2C interface as I2C controllers are not fully standardized.

For eSPI we first need to standardize and have a more efficient eSPI interface. As part of that we are working on updating the EC specification based on PCC interface defined in ACPI. You can find the current draft spec here:

[ESPI PCC Specification](../espi/espi_pcc_draft.md)

## HID Class Drivers

For each top level collection there is a corresponding HID Class driver that gets loaded for that TLC. The HID Class driver implements the miniport driver interface from the OS based on the HID specification for that class.

Below you can find links to the HUTRR documents for HID Class drivers as they are created an draft specifications to cover all the EC services:

- [HID Time and Alarm](HUTRR120-SystemWakeTimerAndRTC.md)
- [HID Battery](HUTRR-DRAFT-Battery.md)
