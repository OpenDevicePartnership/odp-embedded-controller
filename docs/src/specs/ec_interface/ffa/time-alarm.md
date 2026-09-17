# EC Time Alarm Service

The following sections define the operation and definition of the
optional control method-based Time and Alarm device, which provides a
hardware independent abstraction and a more robust alternative to the
Real Time Clock (RTC)

ACPI specification details are in version 6.5 Chapter 9.

[9. ACPI-Defined Devices and Device-Specific Objects — ACPI
Specification 6.5 documentation
(uefi.org)](https://uefi.org/specs/ACPI/6.5/09_ACPI_Defined_Devices_and_Device_Specific_Objects.html#time-and-alarm-device)

| **Command**             | **Description**                                   |
| ----------------------- | ------------------------------------------------- |
| EC_TAS_GET_GCP = 0x1 | Get the capabilities of the time and alarm device |
| EC_TAS_GET_GRT = 0x2 | Get the Real Time                                 |
| EC_TAS_SET_SRT = 0x3 | Set the Real Time                                 |
| EC_TAS_GET_GWS = 0x4 | Get Wake Status                                   |
| EC_TAS_SET_CWS = 0x5 | Clear Wake Status                                 |
| EC_TAS_SET_STV = 0x6 | Set Timer value for given timer                   |
| EC_TAS_GET_TIV = 0x7 | Get Timer value remaining for given timer         |
| EC_TAS_SET_STP = 0x8 | Set expired timer policy for given timer          |
| EC_TAS_GET_TIP = 0x9 | Get expired timer policy for given timer          |

## Relay-backed FF-A command layout

The service UUID is `23ea63ed-b593-46ea-b027-8924df88e92f`. In an FF-A
Direct Request v2 payload, byte 0 is the command and arguments start at byte 1.
Response data starts at payload byte 0. These correspond to ACPI `BUFF`
offsets 32, 33, and 32 respectively; they are not an EC peripheral-memory map.
All multibyte argument and response fields are little-endian.

The secure-world handler forwards commands through the existing ODP relay
(service ID `0x0B`, MCTP message type `0x7D`). The ODP header is big-endian
and carries the command separately from the body. The implemented command
and response IDs are defined in
[`time-alarm-service-relay`](https://github.com/OpenDevicePartnership/embedded-services/blob/main/time-alarm-service-relay/src/serialization.rs).

| Command | Argument bytes | EC success response ID / body |
| --- | --- | --- |
| 1 / _GCP | None | 1 / u32 capabilities |
| 2 / _GRT | None | 2 / 16-byte ACPI timestamp |
| 3 / _SRT | 16-byte ACPI timestamp | 6 / empty |
| 4 / _GWS | u32 timer ID | 3 / u32 wake status |
| 5 / _CWS | u32 timer ID | 6 / empty |
| 6 / _STV | u32 timer ID, u32 seconds | 6 / empty |
| 7 / _TIV | u32 timer ID | 5 / u32 seconds |
| 8 / _STP | u32 timer ID, u32 policy seconds | 6 / empty |
| 9 / _TIP | u32 timer ID | 4 / u32 policy seconds |

Timer ID 0 selects AC; 1 selects DC. The expired-timer policy controls the
delay after returning to the correct power source when a timer expired on
the wrong source: 0 means immediately, and `0xFFFFFFFF` means never.
Other u32 values specify the delay in seconds. `_CWS` clears only the selected
timer's status, not its policy or the other timer's status.

For setters (3, 5, 6, 8), the SP converts an empty EC success response into
a u32 status at FF-A response offset 0: 0 for success, the EC error
discriminant for a remote failure, or `0xFFFFFFFF` for a local, transport,
or protocol failure. The current EC service error is 1 (unspecified failure).
FF-A framework status (`STAT` in the examples) is separate: successful
FF-A delivery alone does not establish command success.
The ACPI methods must translate any nonzero SP status or FF-A failure to
their specified failure value: `0xFFFFFFFF` for `_SRT`, and 1 for `_CWS`,
`_STV`, and `_STP`. These method results are not raw transport error codes.
Scalar getter failures return `0xFFFFFFFF`; `_GRT` failures return an
all-zero invalid timestamp. A policy value of `0xFFFFFFFF` is also valid,
so an isolated `_TIP` read cannot distinguish that policy from a failure.

The timestamp body contains year (u16, offset 0), month/day/hour/minute/second
(bytes 2..6), padding/valid (byte 7), milliseconds (u16, offset 8), timezone
(i16, offset 10), daylight (byte 12), and three reserved zero bytes (13..15).
For `_SRT`, byte 7 is padding (conventionally 0); for successful `_GRT`, it is
the valid byte, 1. The current shared relay serializer also emits 1 for
`_SRT`; the EC decoder accepts either. The SP forwards the input bytes
unchanged rather than normalizing that compatibility difference.

The shared decoder accepts milliseconds 0..999 and daylight values 0, 1,
and 3. ACPI 6.6 still lists milliseconds 1..1000 and does not explicitly
reserve daylight value 2. These decoder restrictions are implementation
compatibility choices, not corrections to the published ACPI specification.

These synchronous commands do not establish physical wake or asynchronous
notification delivery. Power-source/wake integration and notification routing
remain separate work; an EC triggered-wake status bit is bookkeeping, not
proof that the host resumed.

## EC_TAS_GET_GCP

This object is required and provides the OSPM with a bit mask of the
device capabilities.

[9. ACPI-Defined Devices and Device-Specific Objects — ACPI
Specification 6.5
documentation](https://uefi.org/specs/ACPI/6.5/09_ACPI_Defined_Devices_and_Device_Specific_Objects.html#gcp-get-capability)

### Input Parameters

Input parameters as described in ACPI specification.

### Output Parameters

Should return structure as defined by ACPI specification

### FFA ACPI 
```
Method (_GCP) {
  // Check to make sure FFA is available and not unloaded
  If(LEqual(\\_SB.FFA0.AVAL,One)) {
    CreateQwordField(BUFF,0,STAT) // Out – Status for req/rsp
    CreateField(BUFF,128,128,UUID) // UUID of service
    CreateByteField(BUFF,32, CMDD) // In – First byte of command
    CreateDwordField(BUFF,32,GCPD) // Out – 32-bit integer described above
  
    Store(0x1, CMDD) // EC_TAS_GET_GCP
    Store(ToUUID("23ea63ed-b593-46ea-b027-8924df88e92f"), UUID) // RTC
    Store(Store(BUFF, \_SB_.FFA0.FFAC), BUFF)

    If(LEqual(STAT,0x0) ) // Check FF-A successful?
    {
      Return (GCPD)
    }
  }
  Return(Zero)
}
```

## EC_TAS_GET_GRT

This object is required if the capabilities bit 2 is set to 1. The OSPM
can use this object to get time. The return value is a buffer containing
the time information as described below.

[9. ACPI-Defined Devices and Device-Specific Objects — ACPI
Specification 6.5
documentation](https://uefi.org/specs/ACPI/6.5/09_ACPI_Defined_Devices_and_Device_Specific_Objects.html#grt-get-real-time)

### Input Parameters

Input parameters as described in ACPI specification.

### Output Parameters

Should return structure as defined by ACPI specification

### FFA ACPI Example
```
Method (_GRT) {
  Name(RBUF, Buffer(16){})
  // Check to make sure FFA is available and not unloaded
  If(LEqual(\\_SB.FFA0.AVAL,One)) {
    CreateQwordField(BUFF,0,STAT) // Out – Status for req/rsp
    CreateField(BUFF,128,128,UUID) // UUID of service
    CreateByteField(BUFF,32, CMDD) // In – First byte of command
    CreateField(BUFF,256,128,GRTD) // Out – 16-byte timestamp
    Store(0x2, CMDD) // EC_TAS_GET_GRT
    Store(ToUUID("23ea63ed-b593-46ea-b027-8924df88e92f"), UUID) // RTC
    Store(Store(BUFF, \_SB_.FFA0.FFAC), BUFF)

    If(LEqual(STAT,0x0) ) // Check FF-A successful?
    {
      Store(GRTD, RBUF)
    }
  }
  Return(RBUF)
}
```

## EC_TAS_SET_SRT

This object is required if the capabilities bit 2 is set to 1. The OSPM
can use this object to set the time.

[9. ACPI-Defined Devices and Device-Specific Objects — ACPI
Specification 6.5
documentation](https://uefi.org/specs/ACPI/6.5/09_ACPI_Defined_Devices_and_Device_Specific_Objects.html#srt-set-real-time)

### Input Parameters

Input parameters as described in ACPI specification.

### Output Parameters

Should return structure as defined by ACPI specification

### FFA ACPI Example
```
Method (_SRT, 1) {
  // Check to make sure FFA is available and not unloaded
  If(LEqual(\\_SB.FFA0.AVAL,One)) {
    CreateQwordField(BUFF,0,STAT) // Out – Status for req/rsp
    CreateField(BUFF,128,128,UUID) // UUID of service
    CreateByteField(BUFF,32, CMDD) // In – First byte of command
    CreateField(BUFF,264,128,SRTD)  // 16 bytes of data
    CreateDwordField(BUFF,32,SRTS) // Out – Command status

    Store(0x3, CMDD) // EC_TAS_SET_SRT
    Store(ToUUID("23ea63ed-b593-46ea-b027-8924df88e92f"), UUID) // RTC
    Store(Arg0, SRTD) // Copy over the RTC data
    Store(Store(BUFF, \_SB_.FFA0.FFAC), BUFF)

    If(LAnd(LEqual(STAT,0), LEqual(SRTS,0)))
    {
      Return (Zero)
    }
  }
  Return(0xFFFFFFFF)
}
```

## EC_TAS_GET_GWS

This object is required if the capabilities bit 0 is set to 1. It
enables the OSPM to read the status of wake alarms

[9. ACPI-Defined Devices and Device-Specific Objects — ACPI
Specification 6.5
documentation](https://uefi.org/specs/ACPI/6.5/09_ACPI_Defined_Devices_and_Device_Specific_Objects.html#gws-get-wake-alarm-status)

### Input Parameters

Input parameters as described in ACPI specification.

### Output Parameters

Should return structure as defined by ACPI specification

### FFA ACPI Example
```
Method (_GWS, 1) {
  // Check to make sure FFA is available and not unloaded
  If(LEqual(\\_SB.FFA0.AVAL,One)) {
    CreateQwordField(BUFF,0,STAT) // Out – Status for req/rsp
    CreateField(BUFF,128,128,UUID) // UUID of service
    CreateByteField(BUFF,32, CMDD) // In – First byte of command
    CreateDwordField(BUFF,33,GWS1) // In – Dword for timer type AC/DC
    CreateDwordField(BUFF,32,GWSD) // Out – Dword timer state

    Store(20, LENG)
    Store(0x4, CMDD) // EC_TAS_GET_GWS
    Store(Arg0, GWS1)
    Store(ToUUID("23ea63ed-b593-46ea-b027-8924df88e92f"), UUID) // RTC
    Store(Store(BUFF, \_SB_.FFA0.FFAC), BUFF)

    If(LEqual(STAT,0x0) ) // Check FF-A successful?
    {
      Return (GWSD)
    } 
  } 
  Return(Zero)
}
```
##  EC_TAS_SET_CWS

This object is required if the capabilities bit 0 is set to 1. It
enables the OSPM to clear the status of wake alarms

[9. ACPI-Defined Devices and Device-Specific Objects — ACPI
Specification 6.5
documentation](https://uefi.org/specs/ACPI/6.5/09_ACPI_Defined_Devices_and_Device_Specific_Objects.html#cws-clear-wake-alarm-status)

### Input Parameters

Input parameters as described in ACPI specification.

### Output Parameters

Should return structure as defined by ACPI specification

### FFA ACPI Example

```
Method (_CWS, 1) {
  // Check to make sure FFA is available and not unloaded
  If(LEqual(\\_SB.FFA0.AVAL,One)) {
    CreateQwordField(BUFF,0,STAT) // Out – Status for req/rsp
    CreateField(BUFF,128,128,UUID) // UUID of service
    CreateByteField(BUFF,32, CMDD) // In – First byte of command
    CreateDwordField(BUFF,33, CWS1) // In – Dword for timer type AC/DC
    CreateDwordField(BUFF,32,CWSD) // Out – Command status
 
    Store(20, LENG)
    Store(0x5, CMDD) // EC_TAS_SET_CWS
    Store(Arg0,CWS1)
    Store(ToUUID("23ea63ed-b593-46ea-b027-8924df88e92f"), UUID) // RTC
    Store(Store(BUFF, \_SB_.FFA0.FFAC), BUFF)

    If(LAnd(LEqual(STAT,0), LEqual(CWSD,0)))
    {
      Return (Zero)
    }
  } 
  Return(One)
}
```

## EC_TAS_SET_STV

This object is required if the capabilities bit 0 is set to 1. It sets
the timer to the specified value. 

[9. ACPI-Defined Devices and Device-Specific Objects — ACPI
Specification 6.5
documentation](https://uefi.org/specs/ACPI/6.5/09_ACPI_Defined_Devices_and_Device_Specific_Objects.html#stv-set-timer-value)

### Input Parameters

Input parameters as described in ACPI specification.

### Output Parameters

Should return structure as defined by ACPI specification

### FFA ACPI Example
```
Method (_STV, 2) {
  // Check to make sure FFA is available and not unloaded
  If(LEqual(\\_SB.FFA0.AVAL,One)) {
    CreateQwordField(BUFF,0,STAT) // Out – Status for req/rsp
    CreateField(BUFF,128,128,UUID) // UUID of service
    CreateByteField(BUFF,32, CMDD) // In – First byte of command
    CreateDwordField(BUFF,33, STV1) // In – Dword for timer type AC/DC
    CreateDwordField(BUFF,37, STV2) // In – Dword Timer Value
    CreateDwordField(BUFF,32,STVD) // Out – Command status

    Store(0x6, CMDD) // EC_TAS_SET_STV
    Store(Arg0,STV1)
    Store(Arg1,STV2)
    Store(ToUUID("23ea63ed-b593-46ea-b027-8924df88e92f"), UUID) // RTC
    Store(Store(BUFF, \_SB_.FFA0.FFAC), BUFF)
  
    If(LAnd(LEqual(STAT,0), LEqual(STVD,0)))
    {
      Return (Zero)
    }
  }
  Return(One)
}
```

## EC_TAS_GET_TIV

This object is required if the capabilities bit 0 is set to 1. It
returns the remaining time of the specified timer before that expires.

[9. ACPI-Defined Devices and Device-Specific Objects — ACPI
Specification 6.5
documentation](https://uefi.org/specs/ACPI/6.5/09_ACPI_Defined_Devices_and_Device_Specific_Objects.html#tiv-timer-values)

### Input Parameters

Input parameters as described in ACPI specification.

### Output Parameters

Should return structure as defined by ACPI specification

### FFA ACPI Example

```
Method (_TIV, 1) {
  // Check to make sure FFA is available and not unloaded
  If(LEqual(\\_SB.FFA0.AVAL,One)) {
    CreateQwordField(BUFF,0,STAT) // Out – Status for req/rsp
    CreateField(BUFF,128,128,UUID) // UUID of service
    CreateByteField(BUFF,32, CMDD) // In – First byte of command
    CreateDwordField(BUFF,33, TIV1) // In – Dword for timer type AC/DC
    CreateDwordField(BUFF,32,TIVD) // Out – Dword timer state

    Store(0x7, CMDD) // EC_TAS_GET_TIV
    Store(Arg0,TIV1)
    Store(ToUUID("23ea63ed-b593-46ea-b027-8924df88e92f"), UUID) // RTC
    Store(Store(BUFF, \_SB_.FFA0.FFAC), BUFF)

    If(LEqual(STAT,0x0) ) // Check FF-A successful?
    {
      Return (TIVD)
    }
  }
  Return(Zero)
}
```
