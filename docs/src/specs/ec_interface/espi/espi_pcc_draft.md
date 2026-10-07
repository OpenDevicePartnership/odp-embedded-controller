# eSPI ACPI Interface Update

## Purpose

The purpose of this document is to propose an update for the eSPI hardware interface that allows a more efficient and common interface to various eSPI hardware on x86 and ARM designs.

## Legacy eSPI Communication

### Current ACPI EC Interface

The current ACPI design exposes three register roles through two I/O port addresses, together with SCI and SMI signaling, as defined below.

[12. ACPI Embedded Controller Interface Specification — ACPI Specification 6.5 documentation](https://uefi.org/specs/ACPI/6.5/12_Embedded_Controller_Interface_Specification.html)

**I/O Ports**

| **Register** | **Access** | **Purpose**      |
|--------------|------------|------------------|
| EC_SC        | Read       | Status Register  |
| EC_SC        | Write      | Command Register |
| EC_DATA      | Read/Write | Data Register    |

**Status Register**

| **Bit** | **Name** | **Meaning**                    |
|---------|----------|--------------------------------|
| 0       | OBF      | Output Buffer Full             |
| 1       | IBF      | Input Buffer Full              |
| 2       | Reserved |                                |
| 3       | CMD      | Data register contains command |
| 4       | BURST    | EC in burst mode               |
| 5       | SCI_EVT  | SCI query pending              |
| 6       | SMI_EVT  | SMI query pending              |
| 7       | Reserved |                                |

**Command Register**

| **Command** | **Value** |
|-------------|-----------|
| RD_EC       | 0x80      |
| WR_EC       | 0x81      |
| BE_EC       | 0x82      |
| BD_EC       | 0x83      |
| QR_EC       | 0x84      |

### EC Operations

### EC Write Flow

1.  **Wait until ready:** Confirm **EC_SC\[IBF\] = 0**.
2.  **Issue write command:** Write **0x81 (WR_EC)** to **EC_SC**.
3.  **Wait for command acceptance:** Confirm **EC_SC\[IBF\] = 0**.
4.  **Provide address:** Write the EC address to **EC_DATA**.
5.  **Wait for address acceptance:** Confirm **EC_SC\[IBF\] = 0**.
6.  **Provide data:** Write the data byte to **EC_DATA**.
7.  **Complete transaction:** Wait until **EC_SC\[IBF\] = 0**.

### EC Read Flow

1.  **Wait until ready:** Confirm **EC_SC\[IBF\] = 0**.
2.  **Issue read command:** Write **0x80 (RD_EC)** to **EC_SC**.
3.  **Wait for command acceptance:** Confirm **EC_SC\[IBF\] = 0**.
4.  **Provide address:** Write the EC address to **EC_DATA**.
5.  **Wait for response:** Wait until **EC_SC\[OBF\] = 1**.
6.  **Read data:** Read the data byte from **EC_DATA**.

### EC Event Flow

1.  **Receive notification:** OSPM invokes the EC GPE handler.
2.  **Confirm pending SCI:** Verify **EC_SC\[SCI_EVT\] = 1**.
3.  **Wait until ready:** Confirm **EC_SC\[IBF\] = 0**.
4.  **Issue query command:** Write **0x84 (QR_EC)** to **EC_SC**.
5.  **Wait for command acceptance:** Confirm **EC_SC\[IBF\] = 0**.
6.  **Read event code:** Read **EVT = XX** from **EC_DATA**.
7.  **Dispatch handler:** Invoke the corresponding **\_QXX** control method.

> **Note:** The OEM defines the meaning of each event code and its corresponding **\_QXX** method.

### Legacy Architecture

The operating system sees the legacy `EC_SC`/`EC_CMD` and `EC_DATA` I/O-port interface, normally at ports `0x66` and `0x62`. On an eSPI system, the host controller converts these I/O accesses into eSPI Peripheral Channel transactions. Battery, thermal, lid, button, and OEM-defined operations all share the same command/data mailbox and are globally serialized by the ACPI EC driver.

```mermaid
flowchart LR
    subgraph Host["Host / x86"]
        BAT["Battery AML"]
        THM["Thermal AML"]
        LID["Lid / Button AML"]
        OEM["OEM AML"]
        DRV["ACPI EC Driver<br/>Global transaction serialization"]
        PORTS["Legacy EC I/O Ports<br/>0x66: Command / Status<br/>0x62: Data"]

        BAT --> DRV
        THM --> DRV
        LID --> DRV
        OEM --> DRV
        DRV --> PORTS
    end

    subgraph PCH["PCH / SoC"]
        DECODE["Legacy I/O Decode"]
        ESPI["eSPI Controller"]
        VWMAP["VWire to SCI/GPE Mapping"]
    end

    subgraph EC["Embedded Controller"]
        HIF["Legacy Host Interface<br/>IBF / OBF / SCI_EVT"]
        PARSER["Single Command Parser"]
        ERAM["OEM EC RAM Map"]
        TASKS["Battery / Thermal / Lid / OEM Tasks"]
        EVENTS["Internal Event Queue"]

        HIF --> PARSER
        PARSER --> ERAM
        PARSER --> TASKS
        TASKS --> EVENTS
        EVENTS --> HIF
    end

    PORTS --> DECODE
    DECODE --> ESPI
    ESPI -->|"eSPI Peripheral Channel<br/>I/O read/write cycles"| HIF
    HIF -->|"SCI VWire"| VWMAP
    VWMAP -->|"SCI / GPE"| DRV
```

### Legacy Read Transaction

```mermaid
sequenceDiagram
    participant AML as ACPI AML
    participant Driver as ACPI EC Driver
    participant PCH as PCH eSPI Controller
    participant EC as EC Host Interface
    participant Subsystem as EC Subsystem

    AML->>Driver: Read EC field at address A
    Driver->>Driver: Acquire global EC transaction lock
    Driver->>EC: Poll EC_SC.IBF until clear
    Driver->>PCH: OUT 0x66, RD_EC (0x80)
    PCH->>EC: eSPI Peripheral I/O write
    EC->>EC: Consume command and clear IBF
    Driver->>EC: Poll EC_SC.IBF until clear
    Driver->>PCH: OUT 0x62, address A
    PCH->>EC: eSPI Peripheral I/O write
    EC->>Subsystem: Read OEM-defined value at A
    Subsystem-->>EC: Return one byte
    EC->>EC: Place byte in output buffer and set OBF
    Driver->>EC: Poll EC_SC.OBF until set
    Driver->>PCH: IN 0x62
    PCH->>EC: eSPI Peripheral I/O read
    EC-->>PCH: Return data byte
    PCH-->>Driver: Complete IN instruction
    Driver->>Driver: Release global EC transaction lock
    Driver-->>AML: Return field value
```

### Legacy Unsolicited Event

```mermaid
sequenceDiagram
    participant Subsystem as EC Subsystem
    participant EC as EC Host Interface
    participant PCH as PCH / eSPI
    participant Driver as ACPI EC Driver
    participant AML as ACPI _Qxx Method

    Subsystem->>EC: Queue event code XX
    EC->>EC: Set SCI_EVT
    EC->>PCH: Assert SCI VWire
    PCH->>Driver: Raise SCI / GPE
    Driver->>EC: Write QR_EC (0x84) through 0x66
    EC-->>Driver: Return event code XX through 0x62
    Driver->>AML: Evaluate _QXX
```

The legacy transport has the following properties:

- All EC subsystems share one command register and one data register.
- OEM-defined EC RAM ranges separate subsystem data logically, but do not provide independent transport ownership or concurrency.
- Reads and writes are byte-oriented and require repeated `IBF`/`OBF` handshakes.
- The EC queues unsolicited events internally, and the host retrieves them one at a time with `QR_EC`.

## eSPI Features to Address

The following is a list of issues with the current ACPI eSPI EC definition that we are seeking to address:

1.  Current ACPI definition is based on I/O port definition only works on architectures that support I/O ports
2.  Only works for flat memory mapped layout, does not work well with packet-based transactions
3.  Very inefficient throughput for larger transfers of data, it doesn’t take advantage of the actual eSPI protocol ability to send larger packets
4.  Limited VWire support for SMI and SCI, want IRQ and GPIO expansion ability
5.  I/O port model does not allow for easy expansion to multiple channels and protection/securing of channels

## eSPI Proposal over PCC

Existing ACPI specifications are available for defining channel-based communication through PCC.

[14. Platform Communications Channel (PCC) — ACPI Specification 6.5 documentation](https://uefi.org/specs/ACPI/6.5/14_Platform_Communications_Channel.html)

Recommendation is to define a hardware interface that is compatible with PCC Type 3 and 4 that allows us to create inbox ACPI based or driver-based handling of communication with the channel.

Keep PCC Type 0 for a bi-directional flat mapped memory region as existing today and support backward compatible EC’s that are not packet based.

## Hardware Resources

Each eSPI controller consists of channel independent registers, VWire, Peripheral, OOB and Flash Channels. Each of these sections is optional and only present if defined via a resource entry.

### Channel Independent Config

An optional Memory32Fixed region can be defined in the ESPI \_CRS that identifies non-channel independent register configuration as defined in the eSPI specification.

If no resources are defined, then it is presumed that the BIOS or firmware will configure the channels as necessary to operate with the PCC regions defined.

| Start (Hex) | End (Hex) | Register Name                               |
|-------------|-----------|---------------------------------------------|
| 000         | 003       | Reserved                                    |
| 004         | 007       | Device Identification                       |
| 008         | 00B       | General Capabilities and Configurations     |
| 00C         | 00F       | Reserved                                    |
| 010         | 013       | Channel 0 Capabilities and Configurations   |
| 014         | 01F       | Reserved                                    |
| 020         | 023       | Channel 1 Capabilities and Configurations   |
| 024         | 02F       | Reserved                                    |
| 030         | 033       | Channel 2 Capabilities and Configurations   |
| 034         | 03F       | Reserved                                    |
| 040         | 043       | Channel 3 Capabilities and Configurations   |
| 044         | 047       | Channel 3 Capabilities and Configurations 2 |
| 048         | 04B       | Channel 3 Capabilities and Configurations 3 |
| 04C         | 04F       | Channel 3 Capabilities and Configurations 4 |
| 050         | 7FF       | Reserved                                    |
| 800         | FFF       | Platform Specific registers                 |

### Status Registers

The eSPI status register is processed directly by the controller hardware itself and as such isn’t directly exposed. However, the PCC specification requires a doorbell and command complete register for each PCC channel that allows flow control.

**Doorbell** – Requires a single bit in a register or I/O port to indicate when the host has finished writing the data and the device can read the data from the peripheral memory.

**Command Complete** – Requires a single bit in a register or I/O port to indicate that the device has finished processing the data and the host can overwrite it with new data.

**Error Status** – Optional, reports errors to the OS if error recovery isn’t implemented fully in the controller itself.

On existing platforms recommendation is to use an I/O port for each of these. Each bit within the I/O port represents the state for the corresponding channel allowing 8-channels. The mask and values can be programmed via PCC so this isn’t required but a suggested implementation to allow existing hardware to function.

### Global Reset Register

Optional, an MMIO or I/O port register and mask that can be written to that traps into FW to initiate an in-band reset and reconfigures the eSPI config space back to defaults. Any pending transactions are lost, and the controller is in a fresh state.

### VWire Channel

The eSPI controller should decode VWires and allow a configuration that maps at least 16 IRQ’s from index 0 to hard coded SIRQ’s or vectors. These may either be directed to ACPI or the OS may choose to directly register an ISR for these interrupts. The meaning behind these interrupts and what to do in response to them is implementation defined.

How many IRQ’s are supported and their mapping should be defined in the \_CRS of the eSPI device.

System events with indexes 2-7 should be supported via legacy ACPI interface with SCI and SMI support.

Optional support for GPIO expansion via index 128-255 can be defined as well. The details of how those are reported and defining structures need to be decided via MMIO or I/O definitions in future revision.

### Peripheral Channel

There can be multiple logical sub-channels exposed over the single physical eSPI Peripheral Channel. Each logical sub-channel has a corresponding PCCT subspace that defines the channel details and shared-memory region.

For a logical function that supports both host-initiated commands and platform-initiated notifications, define a Type 3/Type 4 pair:

**Type 3** - An Initiator subspace. OSPM initiates the transaction by writing a request and ringing the doorbell. The platform may write the corresponding response into the same shared-memory region before setting Command Complete.

**Type 4** - A Responder subspace. The platform initiates an unsolicited notification, raises the Platform Interrupt, and waits for OSPM to acknowledge and release the region.

A Type 3/Type 4 pair is a logical resource allocation, not an additional physical eSPI channel. A platform should allocate dedicated pairs where latency, isolation, independent reset, or security requires them. It is not necessary to allocate a pair to every EC subsystem. Related low-rate functions may share a logical pair or continue using the legacy EC or Type 0 interface.

For legacy flat mapped memory space which just uses direct memory updates and is not packet based, this region can be directly accessed as MMIO or SystemMemory. Type 0 PCC region may also be used to describe the memory if a doorbell mechanism or more signaling is required.

The doorbell register when written will trigger a peripheral channel transfer of the data length specified by the Length field in the PCC Shared Memory Region.

On systems where MMIO accesses automatically generate peripheral transactions, the doorbell can be set to 0. The command complete register will still need to point to an MMIO or I/O space location to indicate the state of data transfer.

## Sample ACPI Definition for the eSPI PCC Device

The sample separates the namespace device definition from the PCCT subspace records. PCC is exposed as independent subspaces, and the subspace ID is the index of each structure in the PCCT array. The addresses below are illustrative and must be replaced by the platform memory map.

### Illustrative Address Map

| Resource | Base Address | Length | Purpose |
|----|----|----|----|
| Channel-independent configuration | 0xFEDC0000 | 0x1000 | Read-only eSPI identification and channel capability/configuration registers. |
| Global Doorbell Register | 0xFEDC1000 | 0x4 | Doorbell status, allows 32-channels, bit 0-31 represents doorbell status for each channel. |
| Global Command Complete Register | 0xFEDC1004 | 0x4 | Command complete status for 32-channels, one bit per channel. |
| PCC Type 3 shared memory | 0xFEDC2000 | 0x400 | 1024-byte extended master subspace for OS-initiated bidirectional peripheral traffic. |
| PCC Type 4 shared memory | 0xFEDC2400 | 0x400 | 1024-byte extended slave subspace for platform-initiated bidirectional peripheral traffic. |

The 1024-byte shared-memory sizes are illustrative. Supported region and message sizes depend on the resolution of [PCC Region Size versus eSPI Maximum Payload Size](#pcc-region-size-versus-espi-maximum-payload-size).

### Sample SSDT Namespace Definition

DefinitionBlock ("", "SSDT", 2, "OEMID", "ESPIPCC", 0x00000001)  
{  
  Scope (\\SB)  
  {  
    Device (ESPI)  
    {  
      Name (\_HID, "OEM0001")  
      Name (\_UID, Zero)  
      Name (\_DDN, "eSPI PCC Controller")  
      Name (\_STA, 0x0F)  
  
      Name (\_CRS, ResourceTemplate ()  
      {  
        Memory32Fixed (ReadOnly, 0xFEDC0000, 0x00001000, CFG0)  
        Memory32Fixed (ReadWrite, 0xFEDC1000, 0x00000004, STS0)  
        Memory32Fixed (ReadWrite, 0xFEDC1004, 0x00000004, RST0)  
        Memory32Fixed (ReadWrite, 0xFEDC1008, 0x00000008, DBR0)  
        Memory32Fixed (ReadWrite, 0xFEDC1010, 0x00000004, VWF0)  
      })  
  
      OperationRegion (ECFG, SystemMemory, 0xFEDC0000, 0x1000)  
      OperationRegion (GCSR, SystemMemory, 0xFEDC1000, 0x0014)  
      Field (GCSR, DWordAcc, NoLock, Preserve)  
      {  
        GSTA, 32,  
        GRST, 1, Reserved, 31,  
        DB00, 1, Reserved, 31,  
        AK00, 1, Reserved, 31,  
        VWFR, 32  // VWire FIFO, offset 0x10  
      }  
  
      // Extended PCC shared-memory header: 16 bytes, followed by 1008-byte payload.  
      OperationRegion (PCC3, PCC, 0, 0x400)  // Host -\> EC, Type 3  
      Field (PCC3, DWordAcc, NoLock, Preserve)  
      {  
        C3SG, 32,  // Signature, offset 0x00  
        C3FL, 32,  // Flags, offset 0x04  
        C3LN, 32,  // Length, offset 0x08  
        C3CM, 32,  // Command, offset 0x0C  
        C3DT, 8064  // Payload, offset 0x10, 1008 bytes  
      }  
  
      OperationRegion (PCC4, PCC, 1, 0x400)  // EC -\> Host, Type 4  
      Field (PCC4, DWordAcc, NoLock, Preserve)  
      {  
        C4SG, 32,  // Signature, offset 0x00  
        C4FL, 32,  // Flags, offset 0x04  
        C4LN, 32,  // Length, offset 0x08  
        C4CM, 32,  // Command, offset 0x0C  
        C4DT, 8064  // Payload, offset 0x10, 1008 bytes  
      }  
  
      Method (ERST, 0, Serialized)  
      {  
        Store (One, GRST)  
      }  
    }  
  }  
}

**Namespace notes:** **PCC3** references PCCT subspace ID 0 and **PCC4** references subspace ID 1. Each OperationRegion remains exactly **0x400 bytes**. The first **16 bytes** follow the Extended PCC shared-memory header layout: a 32-bit **Signature** at offset 0x00, 32-bit **Flags** at offset 0x04, 32-bit **Length** at offset 0x08, and 32-bit **Command** at offset 0x0C. The remaining **1008 bytes**, starting at offset 0x10, are available for the eSPI packet payload.

### Sample PCCT Subspace Definitions

The PCCT contains an ordered array of PCC subspace structures, with the array index becoming the PCC subspace ID. The following pseudo-definition shows the fields that platform firmware would emit for the two channels; exact C structure names depend on the firmware ACPI table library.

PCCT.Header.Signature = 'PCCT'  
PCCT.Header.Revision = 2  
PCCT.Flags = 1  // Platform interrupt supported  
  
// Subspace ID 0: Extended PCC Master (Type 3)  
Type = 3  
Length = sizeof (TYPE3_SUBSPACE)  
PlatformInterrupt = \<GSI mapped from an eSPI Interrupt Event VWire\><br>
InterruptFlags = \<edge/level and polarity for that GSI\>  
BaseAddress = 0x00000000FEDC2000  
AddressLength = 0x0000000000000400  // 1024 bytes  
DoorbellRegister = GAS(SystemMemory, 32, 0, DWord, 0xFEDC1008)  
DoorbellPreserve = 0xFFFFFFFE  
DoorbellWrite = 0x00000001  
CommandCompleteCheck = GAS(SystemMemory, 32, 0, DWord, 0xFEDC1000)  
CommandCompleteMask = 0x00000001  
CommandCompleteValue = 0x00000001  
ErrorStatusRegister = GAS(SystemMemory, 32, 0, DWord, 0xFEDC1000)  
ErrorStatusMask = 0x00000002  
  
// Subspace ID 1: Extended PCC Slave (Type 4)  
Type = 4  
Length = sizeof (TYPE4_SUBSPACE)  
PlatformInterrupt = \<GSI mapped from an eSPI Interrupt Event VWire\><br>
InterruptFlags = \<edge/level and polarity for that GSI\>  
BaseAddress = 0x00000000FEDC2400  
AddressLength = 0x0000000000000400  // 1024 bytes  
DoorbellRegister = GAS(SystemMemory, 32, 0, DWord, 0xFEDC1008)  
DoorbellPreserve = 0xFFFFFFFE  
DoorbellWrite = 0x00000001  
CommandCompleteCheck = GAS(SystemMemory, 32, 0, DWord, 0xFEDC1000)  
CommandCompleteMask = 0x00000004  
CommandCompleteValue = 0x00000004  
PlatformAckRegister = GAS(SystemMemory, 32, 0, DWord, 0xFEDC100C)  
PlatformAckPreserve = 0xFFFFFFFE  
PlatformAckWrite = 0x00000001

## Proposed Recommendation

### Synchronous Type 3 Requests and Responses

The recommended baseline uses Type 3 as a synchronous, bidirectional request/response subspace. Bidirectional means that the request and response use the same region during different ownership phases; it does not permit the host and platform to write the region concurrently.

Type 4 is independent and is reserved for unsolicited platform notifications. A normal solicited response is returned through the Type 3 region that carried its request.

The recommended eSPI implementation delivers each PCC Platform Interrupt with a target-to-controller eSPI Interrupt Event VWire. The host eSPI controller maps the configured VWire IRQ line to the GSI advertised in the PCCT entry. The PCCT interrupt flags must match the mapped interrupt's trigger mode and polarity.

For a level-triggered interrupt, the PCCT Platform Interrupt Ack register must clear the PCC interrupt source and cause the EC to deassert the VWire. Because eSPI Peripheral and VWire traffic use independent channels, the implementation must ensure that the PCC message and Command Complete state are visible before asserting the VWire.

```mermaid
flowchart LR
    subgraph Host["Host"]
        CLIENT["Function Driver"]
        T3["Type 3 Shared Region"]
        T4["Type 4 Notification Region"]
    end

    subgraph EC["Embedded Controller"]
        HANDLER["Type 3 Command Handler"]
        FUNCTION["EC Function Implementation"]
        EVENTQ["Private Notification Queue"]
        PUBLISHER["Type 4 Publisher"]
    end

    CLIENT -->|"1. Write request<br/>2. Clear complete<br/>3. Ring doorbell"| T3
    T3 -->|"Ownership transfers to EC"| HANDLER
    HANDLER --> FUNCTION
    FUNCTION --> HANDLER
    HANDLER -->|"4. Write response<br/>5. Set complete"| T3
    T3 -->|"6. Interrupt or polling<br/>7. Read response"| CLIENT

    FUNCTION --> EVENTQ
    EVENTQ --> PUBLISHER
    PUBLISHER -->|"Write notification<br/>Raise Platform Interrupt"| T4
    T4 -->|"Read and acknowledge"| CLIENT
```

The function driver submits one command and receives one result. Ownership transfer, memory ordering, interrupt delivery, and notification flow control are handled by the PCC transport implementation. Support for messages larger than one eSPI payload remains an [open question](#pcc-region-size-versus-espi-maximum-payload-size).

### Synchronous Type 3 Transaction

```mermaid
sequenceDiagram
    participant Host as Host Driver
    participant T3 as Type 3 Shared Region
    participant EC as EC Command Handler
    participant Function as EC Function

    Host->>T3: Check Command Complete = 1
    Note over Host,T3: Host owns the available region
    Host->>T3: Write command and request payload
    Host->>T3: Clear Command Complete
    Host->>Host: Ensure request writes are visible
    Note over Host,T3: Ownership transfers to EC
    Host->>EC: Ring Type 3 doorbell
    EC->>T3: Read request
    EC->>Function: Execute operation
    Function-->>EC: Return status and response data
    EC->>T3: Write response into the same region
    EC->>EC: Ensure response writes are visible
    EC->>T3: Set Command Complete
    Note over Host,T3: Ownership transfers back to host
    EC-->>Host: Optional Type 3 Platform Interrupt
    Host->>T3: Read response
```

For this profile, Type 3 Command Complete means that the requested operation has completed and the response is available. Only one Type 3 transaction is outstanding on a subspace, so a transaction identifier is not required.

### Independent Type 4 Notification

Type 4 carries platform-initiated traffic such as HID Input reports, GPIO events, sensor notifications, or function-specific state changes. It may operate while a Type 3 command is in progress.

```mermaid
sequenceDiagram
    participant Host as Host Driver
    participant T3 as Type 3
    participant EC as EC
    participant T4 as Type 4

    Host->>T3: Submit request
    T3->>EC: Doorbell

    par Process synchronous command
        EC->>EC: Execute operation
        EC->>T3: Write response and set complete
        T3-->>Host: Optional completion interrupt
        Host->>T3: Read response
    and Publish unsolicited notification
        EC->>T4: Wait for Type 4 availability
        EC->>T4: Write notification and clear complete
        T4-->>Host: Platform Interrupt
        Host->>T4: Read notification
        Host->>T4: Set complete and acknowledge interrupt
    end
```

The platform must queue notifications internally while Type 4 is owned by OSPM. It must not overwrite an unacknowledged notification.

### Logical Sub-channel Allocation

The recommended design does not require one pair for every EC subsystem. It recommends dedicated pairs for functions whose latency, isolation, security, or independent reset requirements justify them. For example, a latency-sensitive HID function can use a dedicated pair while battery, thermal, and fan functions share a generic EC pair or continue to use the legacy interface.

```mermaid
flowchart TB
    subgraph Host["Host Drivers"]
        GH["Generic EC Driver"]
        HH["HID Driver"]
    end

    subgraph PCC["Logical PCC Sub-channels over one eSPI Peripheral Channel"]
        G3["Generic EC Type 3"]
        G4["Generic EC Type 4"]
        H3["Dedicated HID Type 3"]
        H4["Dedicated HID Type 4"]
    end

    subgraph EC["EC Functions"]
        GE["Battery / Thermal / Fan"]
        HE["HID Function"]
    end

    GH --> G3 --> GE
    GE --> G4 --> GH

    HH --> H3 --> HE
    HE --> H4 --> HH
```

Each pair consumes two PCCT subspaces, two shared-memory windows, Command Complete and doorbell state, and Type 4 interrupt-acknowledgment state. It does not consume another physical eSPI channel. Level-triggered Platform Interrupts may be shared when each subspace has a unique status and acknowledgment mask. The cost and availability of these resources on existing hardware remain an [open question](#pcc-resource-cost-and-in-market-device-capabilities).

The sample 1024-byte Type 3 and Type 4 regions require 2048 bytes of addressable storage for one pair. A 512-byte-per-region profile may be used where memory is constrained. The selected region size and maximum message size must be advertised to software and must follow the resolution of [PCC Region Size versus eSPI Maximum Payload Size](#pcc-region-size-versus-espi-maximum-payload-size).

### Shared-Pair Specialization

A resource-constrained platform may specialize the recommended design by multiplexing multiple EC functions through one Type 3/Type 4 pair. The transport semantics remain unchanged:

- Type 3 contains a synchronous request and its solicited response.
- Type 3 Command Complete means that the operation and response are complete.
- Type 4 contains unsolicited platform notifications.

Only the logical allocation changes. The shared payload format must identify the target function and command namespace, and the host requires an EC PCC bus driver that dispatches requests and notifications to function-specific clients.

```mermaid
flowchart LR
    subgraph Host["Host"]
        BUS["EC PCC Bus Driver"]
        BAT["Battery Client"]
        HID["HID Client"]
        THM["Thermal Client"]
        BAT --> BUS
        HID --> BUS
        THM --> BUS
    end

    subgraph PCC["Recommended Synchronous Transport<br/>Shared-Pair Specialization"]
        T3["Shared Type 3<br/>Request and Response"]
        T4["Shared Type 4<br/>Unsolicited Notifications"]
    end

    subgraph EC["Embedded Controller"]
        DISPATCH["Function Dispatcher"]
        SCHED["Priority and Fairness Policy"]
        FUNCTIONS["Battery / HID / Thermal Functions"]
        DISPATCH --> SCHED --> FUNCTIONS
    end

    BUS --> T3 --> DISPATCH
    FUNCTIONS --> T4 --> BUS
```

This specialization reduces PCCT entries, shared-memory windows, and interrupt state, but all participating functions share the same Type 3 ownership token and Type 4 notification slot. The implementation must define scheduling priorities, quotas, and starvation prevention. A slow operation blocks other Type 3 users until its synchronous response is complete, so latency-sensitive functions such as HID may still require a dedicated pair.

### Recommended Completion Semantics

| Operation | Recommended behavior |
|---|---|
| Short read or query | Synchronous Type 3 request and response |
| Short write with status | Synchronous Type 3 request and response |
| Fire-and-forget write | Type 3 completes after the platform has safely consumed the data |
| Unsolicited platform event | Type 4 notification |
| Long-running operation, such as firmware-update erase, program, or verification | Prefer a separate asynchronous command defined by the function protocol; see alternatives below |

## Other Design Alternatives

### Decoupled Type 3 Requests and Type 4 Responses

In the decoupled model, Type 3 Command Complete means only that the platform validated and copied the request. The final solicited response is later published through Type 4. This permits the Type 3 region to be released before a long-running operation, such as firmware-update erase, program, or verification, finishes, but it changes the transport interface from synchronous completion to asynchronous acceptance and completion.

Because solicited responses and unsolicited notifications share Type 4 in this model, every Type 4 message requires an explicit message class and transaction identifier:

| Type 4 message | Message class | Transaction identifier |
|---|---|---|
| Solicited response | Response | Copied from the corresponding Type 3 request |
| Unsolicited notification | Notification | Reserved notification value; no Type 3 request is referenced |

The message class distinguishes the two flows without relying on whether a transaction identifier happens to match an outstanding request. A transaction identifier must not be reused while its request is pending.

```mermaid
flowchart LR
    subgraph Host["Host"]
        CLIENT["Function Driver"]
        REQ["Type 3 Request Region"]
        MSG["Type 4 Response / Notification Region"]
        PENDING["Outstanding Transaction Table"]

        CLIENT --> REQ
        CLIENT --> PENDING
        MSG --> CLIENT
    end

    subgraph EC["Embedded Controller"]
        INTAKE["Type 3 Doorbell Handler"]
        REQQ["Private Request Queue"]
        WORKERS["Function Workers"]
        MSGQ["Response / Notification Queue"]
        PUB["Type 4 Publisher"]

        INTAKE -->|"Copy request"| REQQ
        REQQ --> WORKERS
        WORKERS --> MSGQ
        MSGQ --> PUB
    end

    REQ -->|"Doorbell"| INTAKE
    INTAKE -->|"Set Type 3 complete<br/>after copying"| REQ
    PUB -->|"Write response or event"| MSG
    PUB -->|"Platform Interrupt"| CLIENT
    CLIENT -->|"Set Type 4 complete"| PUB
```

```mermaid
sequenceDiagram
    participant Host as Host Driver
    participant T3 as Type 3 Region
    participant Intake as EC Doorbell Handler
    participant Worker as EC Function Worker
    participant T4 as Type 4 Region

    Host->>T3: Write request A, transaction 41
    Host->>T3: Clear Command Complete
    Host->>Intake: Ring Type 3 doorbell
    Intake->>T3: Read and copy request A
    Intake->>Intake: Enqueue request A
    Intake->>T3: Set Command Complete

    Note over Host,T3: Type 3 is available before request A finishes

    Host->>T3: Write request B, transaction 42
    Host->>Intake: Ring Type 3 doorbell
    Intake->>T3: Copy request B and set complete

    Worker->>Worker: Process B quickly
    Worker->>T4: Publish response B, transaction 42
    T4-->>Host: Platform Interrupt
    Host->>T4: Read and correlate response 42
    Host->>T4: Set Command Complete

    Worker->>Worker: Finish request A
    Worker->>T4: Publish response A, transaction 41
    T4-->>Host: Platform Interrupt
    Host->>T4: Read and correlate response 41
    Host->>T4: Set Command Complete
```

This alternative requires message classes, transaction identifiers, request and response queues, queue-full behavior, acceptance and final-result timeouts, cancellation, reset generation handling, and response/notification scheduling. It is appropriate when operations are long-running and concurrent acceptance provides measurable benefit. It should not be the default completion model for short commands.

### Type 3-Only Interrupt-and-Pull

Resource-constrained systems may omit Type 4. The platform signals that an event is pending, and OSPM issues a Type 3 request to retrieve it.

```mermaid
sequenceDiagram
    participant Function as EC Function
    participant EC as EC Transport
    participant Host as Host Driver
    participant T3 as Type 3 Region

    Function->>EC: Queue unsolicited event
    EC-->>Host: Raise event interrupt or status indication
    Host->>T3: Submit GET_PENDING_NOTIFICATION
    T3->>EC: Doorbell
    EC->>T3: Write next event and set complete
    Host->>T3: Read event
```

This profile saves one shared-memory region and the Type 4 acknowledgment state, but adds an interrupt-to-request-to-response round trip for every event. It is suitable for low-rate events, not high-rate input or sensor data.

### Design Comparison

| Property | Legacy ACPI EC | Recommended synchronous PCC | Decoupled PCC |
|---|---|---|---|
| Request path | Ports `0x62`/`0x66` | Type 3 | Type 3 |
| Solicited response path | Legacy data register | Same Type 3 region | Type 4 |
| Unsolicited event path | SCI plus `QR_EC` | Type 4 | Type 4 |
| Completion meaning | Byte/transaction consumed | Operation and response complete | Request copied/accepted |
| Transaction ID required | No | No for one outstanding request | Yes |
| Blocking scope | All EC users | One logical sub-channel or all functions in the shared-pair specialization | Request intake only; Type 4 may still block |
| EC private queues | Event queue | Notification queue | Request and response queues |
| Host complexity | Legacy EC state machine | Synchronous command plus notification handler; shared specialization adds dispatch and fairness | Correlation and pending-request state |

## Open Questions

### PCC Region Size versus eSPI Maximum Payload Size

Can the hardware expose a PCC region larger than the negotiated 64-, 128-, or 256-byte eSPI Maximum Payload Size and transfer the complete message through ordered transactions before signaling the doorbell, Command Complete, or Platform Interrupt?

The profile must select one of these approaches:

- Limit the PCC message and region size to the negotiated eSPI Maximum Payload Size.
- Define which protocol layer fragments and reassembles larger messages, which message classes may use it, and its ordering and error rules.
- Add a controller-managed staging or DMA mechanism with an explicit whole-message completion indication.

### PCC Resource Cost and In-Market Device Capabilities

Can in-market ECs and eSPI controllers expose a Type 3/Type 4 pair, including two shared-memory windows, signaling state, and a Type 4 interrupt acknowledgment path, without hardware changes?

The specification must determine the typical SRAM and controller-resource cost, identify capabilities that software can discover, and define a compatible profile for devices that support only one region or the legacy interface.

## References

- [ACPI 6.6, Chapter 14: Platform Communications Channel](https://uefi.org/specs/ACPI/6.6/14_Platform_Communications_Channel.html)
- [ACPI 6.5, Chapter 12: Embedded Controller Interface](https://uefi.org/specs/ACPI/6.5/12_Embedded_Controller_Interface_Specification.html)
- [eSPI Base Specification](https://www.intel.com/content/www/us/en/content-details/841852/enhanced-serial-peripheral-interface-espi-interface-base-specification-for-client-and-server-platforms.html)