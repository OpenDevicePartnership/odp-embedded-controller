# Embedded Controller Interface Specification

Embedded Controller(EC) Interface Specification describes base set of requirements to interface to core Windows features.

It covers the following areas:
- Firmware Management
- Battery
- Time and Alarm
- UCSI
- Thermal
- Power
- Input Devices
- Customization

## Protocols and Stacks

There are 3 ways to communicate with the EC:

- ACPI specification for Legacy EC controller (eSPI) based interface on x86 devices, directly accessed through OperationRegion in ACPI
- ACPI through FFA driver and in secure world an SV or OEM customer driver to communicate with the EC on ARM devices
- Directly through HID class driver for bus based devices, I2C, I3C and eSPI

This specification is organized accordingly and you can find the details for each protocol in the sub folder.

[Legacy EC Interface](legacy/README.md)

[FFA EC Interface](ffa/README.md)

[HID EC Interface](hid/README.md)


## Transports

While other transports can be wired up to work with OEM or SV drivers we are focused on supporting I2C, I3C and eSPI natively both through direct ACPI access and the HID stacks.

Today there is partial support for I2C and eSPI in ACPI and support for I2C and I3C on HID. To support eSPI natively work is being done to better standardize eSPI based on PCC specification.

The following is a visual of the driver stacks planed to interface with the EC from the OS.

# CPU-to-Embedded-Controller Architecture

```mermaid
flowchart TB
    subgraph CPU["CPU / PC"]
        direction LR

        subgraph NSW["Non-Secure World"]
            direction TB

            subgraph STACK["Top-Level Stack"]
                direction LR
                INPUT["Input"]
                BATTERY["Battery"]
                TAD["TAD"]
                MPTF["MPTF"]
                OEM["OEM"]
            end

            subgraph HID_DRIVERS["HID Class Drivers"]
                direction LR
                INPUT_HID["Input HID Driver"]
                BATTERY_HID["Battery HID Driver"]
                TAD_HID["TAD HID Driver"]
                MPTF_HID["MPTF HID Driver"]
                OEM_HID["OEM HID Driver"]
            end

            INPUT --> INPUT_HID
            BATTERY --> BATTERY_HID
            TAD --> TAD_HID
            MPTF --> MPTF_HID
            OEM --> OEM_HID

            subgraph OS_DRIVERS["OS Driver Layer"]
                direction LR
                HIDCLASS["HIDClass Driver"]
                ACPI["ACPI Driver"]
            end

            INPUT_HID --> HIDCLASS
            BATTERY_HID --> HIDCLASS
            TAD_HID --> HIDCLASS
            MPTF_HID --> HIDCLASS
            OEM_HID --> HIDCLASS

            subgraph HID_SUPPORT["HID Transport Support"]
                direction LR
                HIDSPBCX["HIDSpbCx"]
                HIDI3C["HIDI3C"]
                HIDESPI["HIDeSPI"]
            end

            HIDCLASS --> HIDSPBCX
            HIDCLASS --> HIDI3C
            HIDCLASS --> HIDESPI

            I2C["I2C Transport"]
            I3C["I3C Transport"]
            ESPI["eSPI Transport"]

            HIDSPBCX --> I2C
            HIDI3C --> I3C
            HIDESPI --> ESPI

            FFA["FF-A"]
            I2CI3C["I2C/I3C"]
            ESPIA["ESPI"]
            ACPI --> FFA
            ACPI --> I2CI3C
            ACPI --> ESPIA
        end

        subgraph SW["Secure World"]
            direction TB
            HAFNIUM["Hafnium"]
            ECSP["EC SP"]
            CUSTOM["Custom Transport Driver"]

            HAFNIUM --> ECSP
            ECSP --> CUSTOM
        end

        FFA --> HAFNIUM
    end

    subgraph EC["Embedded Controller"]
        direction TB
        EC_IF["EC Transport Interfaces"]
        EC_FW["EC Firmware / Services"]
        EC_IF --> EC_FW
    end

    I2C -->|I2C| EC_IF
    I3C -->|I3C| EC_IF
    ESPI -->|eSPI| EC_IF
    ESPIA -->|eSPI| EC_IF
    I2CI3C --> |ACPI| EC_IF
    CUSTOM -->|Custom secure transport| EC_IF

    classDef nonsecure fill:#dbeafe,stroke:#2563eb,color:#111827;
    classDef secure fill:#fef3c7,stroke:#d97706,color:#111827;
    classDef controller fill:#dcfce7,stroke:#16a34a,color:#111827;
    class INPUT,BATTERY,TAD,MPTF,OEM,INPUT_HID,BATTERY_HID,TAD_HID,MPTF_HID,OEM_HID,HIDCLASS,ACPI,HIDSPBCX,HIDI3C,HIDESPI,I2C,I3C,ESPI,FFA nonsecure;
    class HAFNIUM,ECSP,CUSTOM secure;
    class EC_IF,EC_FW controller;
```
