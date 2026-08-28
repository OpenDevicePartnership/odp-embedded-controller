Request #:        HUTRR TBD
Title:            Control Method Battery
Spec Release:     HID Usage Tables 1.7
Received:         <Filled by HID Chair>
Requester:        Phil Weber
Company:          Microsoft
------------------
Pages Affected:   Battery System Page (0x85)
Values Checked:   0x0100-0x0186
------------------
Current Status:   Draft
Priority:         Normal
Submitted:        <TBD>
------------------
Required Voter:   <TBD>
Required Voter:   <TBD>
Required Voter:   <TBD>
------------------
Voting Begins:    <Filled by HID Chair>
Voting Ends:      <Filled by HID Chair>
Voting Result:    <Filled by HID Chair>


Summary
=======

Add a Control Method Battery application usage to the Battery System Usage Page
(0x85). The new top-level collection (TLC) exposes one independently managed
battery using a transport-neutral HID protocol modeled on the ACPI 6.5 Control
Method Battery interface.

The TLC reuses existing Power Page (0x84) and Battery System Page (0x85) usages
where their semantics match. New usages are introduced for ACPI fields and
operations that have no equivalent HID usage, including extended static
information, portable UTF-8 identity strings, measurement configuration,
maintenance controls, charge/runtime estimates, peak-power capabilities,
thresholds, thermal throttling, capability negotiation, notifications, and a
stable battery tag.

This draft intentionally defines the complete ACPI 6.5 battery surface as part
of the HIDBattery contract rather than defining only the subset currently
consumed by one operating system.


Background
==========

ACPI Control Method Batteries provide operating systems with a common model for
static battery information, dynamic status, capacity-change notification, and
optional battery management functions. That model is independent of the
physical fuel-gauge or charger implementation, but ACPI normally exposes it
through AML control methods.

HID already defines Power (0x84) and Battery System (0x85) pages. Those pages
contain useful electrical and Smart Battery usages, but they do not define an
application collection whose report contract is equivalent to an ACPI Control
Method Battery. Implementations consequently need vendor-defined collections
and host-specific knowledge even when the underlying information is standard.

Windows battery miniclass drivers are expected to provide battery identity,
static information, current status, status-change notifications, and selected
controls to the battery class driver. A HID miniclass can implement those
callbacks directly from this TLC without requiring vendor-specific report
formats.


Goals
=====

1. Define one standard battery TLC that is independent of USB, HID-over-I2C,
   HID-over-SPI, and other HID transports.
2. Preserve the meaning, units, special values, and event behavior of the ACPI
   6.5 Control Method Battery interface.
3. Reuse existing HID usages when their meaning is equivalent.
4. Support a persistent controller with a removable or replaceable battery
   pack.
5. Support multiple batteries by exposing one TLC for each independently
   managed battery.
6. Carry battery identity strings in reports rather than relying on
   transport-specific string descriptor indexes.
7. Provide the information and controls needed by the Windows battery
   miniclass interface without making Windows structures the wire format.


Non-goals
=========

1. This proposal does not define charger, AC adapter, UPS, or system aggregate
   battery TLCs.
2. This proposal does not require report IDs to have fixed numeric values.
   Hosts discover reports and fields by usages.
3. This proposal does not expose AML, ACPI namespace paths, General Purpose
   Events, or System Control Interrupts on the wire.
4. This proposal does not replace the existing Smart Battery collections.


Proposal
========

Add the Control Method Battery application usage and supporting usages to the
Battery System Page (0x85). Change the final reserved range from 0xF4-0xFFFF to:

    0x00F4-0x00FF  Reserved
    0x0100-0x0186  Defined by this proposal
    0x0187-0xFFFF  Reserved

For this TLC only, add Dynamic Flag (DF) as an allowed alternate usage type for
the existing Charging (0x44) and Discharging (0x45) usages. Their existing
Selector (Sel) type and existing behavior are unchanged.


Control Method Battery application usage
----------------------------------------

Usage ID   Usage Name               Type
--------   -----------------------  ----
0x0100     Control Method Battery   CA

Control Method Battery (0x0100)

Defines an Application Collection for one independently managed battery. A
device containing multiple independently managed batteries shall expose one
Control Method Battery TLC per battery. A persistent controller may keep the
TLC enumerated while Battery Present is zero. Battery Tag shall change when the
pack represented by the TLC is removed, replaced, or materially changes
identity.

All reports defined by this proposal are required. A field may contain an
unknown value only when this proposal or the referenced ACPI definition
explicitly defines an unknown value.


Collection usages
-----------------

Usage ID   Usage Name                              Type
--------   --------------------------------------  ----
0x0101     Control Method Battery Information      CL
0x0102     Control Method Battery Identity         CL
0x0103     Control Method Battery Status           CL
0x0104     Control Method Battery Notification     CL
0x0105     Battery Measurement Configuration       CL
0x0106     Battery Maintenance                     CL
0x0107     Battery Charge Time                     CL
0x0108     Battery Runtime                         CL
0x0109     Battery Power Characteristics           CL
0x010A     Battery Power State                     CL
0x010B     Battery Power Threshold                 CL
0x010C     Battery Capacity Trip Point              CL
0x010D     Battery Thermal Control                 CL
0x010E     Battery Capability Negotiation          CL
0x010F     Power Consumer List                     CL

Each collection usage groups the fields for the correspondingly named report.
The grouping does not change the independent meaning of any data usage.


Protocol and static-information usages
--------------------------------------

Usage ID   Usage Name                         Type   Description
--------   ---------------------------------  -----  -----------
0x0120     Protocol Revision                  SV     Revision of this HIDBattery report contract. This draft defines revision 1.
0x0121     Battery Tag                        DV     Nonzero identity value for the currently installed pack. It changes after removal, replacement, or identity change. Zero means that no battery is present.
0x0122     Battery Information Revision       SV     Revision of the static information layout. ACPI _BIX revision 1 is encoded as 1.
0x0123     Battery Power Unit                 CL     Selects the unit for all capacity, rate, and peak-power fields in this TLC.
0x0124     Capacity in Milliwatt Hours        Sel    Capacity is in mWh and rate/power is in mW.
0x0125     Capacity in Milliampere Hours      Sel    Capacity is in mAh and rate/current is in mA.
0x0126     Battery Technology                 CL     Selects primary or secondary battery technology.
0x0127     Primary Battery Technology         Sel    Non-rechargeable battery.
0x0128     Secondary Battery Technology       Sel    Rechargeable battery.
0x0129     Design Capacity Low                SV     OEM low-capacity threshold in the selected capacity unit.
0x012A     Measurement Accuracy               SV     Measurement accuracy in thousandths of one percent; 80000 represents 80 percent.
0x012B     Maximum Sampling Time              SV     Maximum supported sampling interval in milliseconds. All ones means unavailable.
0x012C     Minimum Sampling Time              SV     Minimum supported sampling interval in milliseconds. All ones means unavailable.
0x012D     Maximum Averaging Interval         SV     Maximum supported averaging interval in milliseconds.
0x012E     Minimum Averaging Interval         SV     Minimum supported averaging interval in milliseconds.
0x012F     Battery Swapping Capability        CL     Selects the replacement behavior of the battery.
0x0130     Battery Not Swappable              Sel    Battery is not user replaceable.
0x0131     Battery Cold Swappable             Sel    System must be shut down before replacement while on battery power.
0x0132     Battery Hot Swappable              Sel    Battery may be replaced without shutting down the system.

The following existing usages are reused in Control Method Battery Information:

Page   ID     Usage Name                  Meaning in this TLC
-----  -----  --------------------------  --------------------------------------
0x85   0x83   Design Capacity             ACPI _BIX Design Capacity.
0x85   0x67   Full Charge Capacity        ACPI _BIX Last Full Charge Capacity.
0x84   0x40   Config Voltage              ACPI _BIX Design Voltage, in mV.
0x85   0x8C   Warning Capacity Limit      ACPI _BIX Design Capacity of Warning.
0x85   0x6B   Cycle Count                 ACPI _BIX Cycle Count.
0x85   0x8D   Capacity Granularity 1      ACPI _BIX Battery Capacity Granularity 1.
0x85   0x8E   Capacity Granularity 2      ACPI _BIX Battery Capacity Granularity 2.
0x85   0x85   Manufacture Date            Packed Smart Battery manufacture date.

Design Capacity, Full Charge Capacity, and Config Voltage use all ones to mean
unknown. Cycle Count uses all ones to mean unknown. Capacity values use the
selected Battery Power Unit. Voltage is in millivolts.


Identity usages and string encoding
-----------------------------------

Usage ID   Usage Name                      Type
--------   ------------------------------  -------------
0x0133     Battery Model Number            BufferedBytes
0x0134     Battery Serial Number String    BufferedBytes
0x0135     Battery Type String             BufferedBytes
0x0136     Battery OEM Information         BufferedBytes
0x0137     Battery Manufacturer Name       BufferedBytes
0x0138     Battery Unique Identifier       BufferedBytes
0x0139     Power Consumer Identifiers      BufferedBytes

Each identity field is a fixed-size byte buffer declared by the report
descriptor. It contains a UTF-8 string terminated by 0x00. The first 0x00 and
all remaining bytes shall be zero. A value that does not fit shall be truncated
at a UTF-8 code-point boundary and terminated. An unsupported or empty value is
encoded as an all-zero buffer.

Battery Model Number, Battery Serial Number String, Battery Type String, and
Battery OEM Information correspond to the ACPI _BIX strings. Battery
Manufacturer Name and Battery Unique Identifier provide the additional identity
needed by battery class APIs. Battery Unique Identifier shall be stable for the
life of a physical pack and unique among packs that can be attached to the
system.

Power Consumer Identifiers is a sequence of NUL-terminated UTF-8 stable device
identifiers followed by an additional NUL. It represents the relationships
reported by ACPI _PCL. An empty list contains two zero bytes. Each identifier
shall be either an ACPI namespace path beginning with "acpi:" or an RFC 8141
UUID URN beginning with "urn:uuid:". A platform that uses UUID URNs shall expose
the same UUID as a device-enumeration property on the referenced consumer so
the host can resolve the relationship.

The complete encoded list, including the final extra NUL, shall fit in the
declared report field. It shall not be silently truncated. A conforming device
therefore selects a field size large enough for its complete consumer list.


Dynamic-status and event usages
-------------------------------

Usage ID   Usage Name                         Type   Description
--------   ---------------------------------  -----  -----------
0x0140     Battery Critical                   DF     Battery is in the critical energy state.
0x0141     Battery Charge Limiting Active     DF     Battery charge limiting is active.
0x0142     Battery Present Rate               DV     Power accepted or supplied at the battery terminals, in mW or mA according to Battery Power Unit. Direction is given by Charging and Discharging. All ones means unknown.
0x0143     Battery Status Sequence            DV     Monotonically increasing status generation. Wrap from 0xFFFFFFFF to zero is permitted.
0x0144     Battery Information Changed        DF     Static information or identity changed and shall be reread.
0x0145     Battery Maintenance Changed        DF     Maintenance status changed and shall be reread.
0x0146     Battery Power State Changed        DF     Peak-power capability changed and shall be reread.
0x0147     Battery Power Threshold Crossed    DF     A configured peak-power threshold was crossed.
0x0186     System On Line                     DF     The system is drawing operating power from an external source.

The following existing usages are reused in Control Method Battery Status:

Page   ID     Usage Name          Meaning in this TLC
-----  -----  ------------------  --------------------------------------------
0x85   0x44   Charging            ACPI _BST charging state; used as a DF.
0x85   0x45   Discharging         ACPI _BST discharging state; used as a DF.
0x85   0xD0   AC Present          External power is physically connected.
0x85   0xD1   Battery Present     A battery pack is installed in this TLC.
0x85   0x66   Remaining Capacity  ACPI _BST Battery Remaining Capacity.
0x84   0x30   Voltage             ACPI _BST Battery Present Voltage, in mV.
0x84   0x36   Temperature         Battery temperature, in 0.1 kelvin.

Charging and Discharging are mutually exclusive. A rechargeable battery shall
report a valid Present Rate, Remaining Capacity, and Voltage. A primary battery
may report all ones where ACPI permits an unknown value.

AC Present reports physical source presence. System On Line reports the
effective source supplying the system. System On Line shall be zero while the
battery is intentionally discharging with AC present, including Discharge on AC
Enabled operation. Hosts shall use System On Line, not AC Present, for
operating-system "on line power" state.

The device shall send the status Input report when any status value changes,
when Battery Tag changes, or when one of the event flags is asserted. Event
flags describe the reason for the current report and are cleared after the
report is successfully delivered. Battery Status Sequence increments before
each changed report is made available. The host shall tolerate lost or
coalesced Input reports and may read all Feature reports at any time.

Battery Information Changed corresponds to ACPI Notify 0x81. Battery
Maintenance Changed corresponds to Notify 0x82. Battery Power State Changed and
Battery Power Threshold Crossed correspond to Notify 0x83. A remaining-capacity
trip-point crossing causes a status report and corresponds to Notify 0x80.


Windows notification support usages
-----------------------------------

Usage ID   Usage Name                       Type   Description
--------   -------------------------------  -----  -----------
0x0182     Notification Power State Mask    DV     Power-state changes that trigger a status Input report.
0x0183     Notification Low Capacity        DV     Inclusive lower capacity bound in the selected capacity unit.
0x0184     Notification High Capacity       DV     Inclusive upper capacity bound in the selected capacity unit.
0x0185     Critical Bias                    DV     Capacity reserved for the critical transition, in the selected capacity unit.

The host writes the Control Method Battery Notification Feature report to arm
notifications. The device shall generate a status Input report when the power
state differs from the requested mask or Remaining Capacity leaves the
inclusive low/high band. Writing a power-state mask of zero and both capacity
bounds as zero disables this notification condition. ACPI Battery Capacity Trip
Point remains independently active.


Measurement configuration usages
--------------------------------

Usage ID   Usage Name          Type   Description
--------   ------------------  -----  -----------
0x0148     Sampling Time       DV     Time in milliseconds between measurements used for status values.
0x0149     Averaging Interval  DV     Time in milliseconds over which status measurements are averaged.
0x014A     Operation Status    DV     Result of the most recent operation in the containing report.

Sampling Time maps to ACPI _BMS and shall be within Minimum Sampling Time and
Maximum Sampling Time. Averaging Interval maps to ACPI _BMA and shall be within
Minimum Averaging Interval and Maximum Averaging Interval. Values are 1 through
0xFFFFFFFF milliseconds.

The host sets both values in one Set Feature operation. The device applies the
values atomically. Operation Status is read-only:

    0  Success
    1  One or both values are outside the supported range
    2-0xFFFFFFFF  Reserved

The host may Get Feature after Set Feature to verify the applied values and
Operation Status.


Maintenance usages
------------------

Usage ID   Usage Name                                             Type
--------   -----------------------------------------------------  ----
0x0150     Calibration Active                                     DF
0x0151     Charging Disabled                                      DF
0x0152     Discharge on AC Enabled                                DF
0x0153     Recalibration Requested                                DF
0x0154     Standby Recommended for Calibration                    DF
0x0155     Charge Limit Suspension Blocked by Thermal Condition   DF
0x0156     Charge Limit Suspension Blocked by Battery Protection  DF
0x0157     Calibration Supported                                  SF
0x0158     Charging Control Supported                             SF
0x0159     Discharge on AC Supported                              SF
0x015A     Individual Battery Control Supported                   SF
0x015B     Charge Before Calibration Required                     SF
0x015C     Charge Limit Suspension Supported                      SF
0x015D     Recalibrate Count                                      SV
0x015E     Quick Recalibrate Time                                 DV
0x015F     Slow Recalibrate Time                                  DV
0x0160     Start Calibration                                      OOC
0x0161     Disable Charging                                       OOC
0x0162     Enable Discharge on AC                                 OOC
0x0163     Suspend Charge Limiting                                OOC

The state, capability, count, and time usages map in order to ACPI _BMD.
Start Calibration, Disable Charging, Enable Discharge on AC, and Suspend Charge
Limiting map to ACPI _BMC argument bits 0 through 3.

Start Calibration is mutually exclusive with the other three controls and takes
precedence if set. Suspend Charge Limiting shall be set only when Charge Limit
Suspension Supported is one. A device that cannot apply a control shall leave
the corresponding state unchanged and assert Battery Maintenance Changed.

If Individual Battery Control Supported is zero, writing any maintenance
control in one TLC applies to every affected battery in the system. The device
shall update the maintenance state and send Battery Maintenance Changed status
reports for every affected TLC. Hosts shall not assume that an operation is
local to the TLC that received the write.

Recalibrate Count is zero when recalibration is requested only by
Recalibration Requested. Quick and Slow Recalibrate Time are in seconds:

    0           Meaning defined by the corresponding ACPI _BMD field
    1-0xFFFFFFFE Estimated or remaining time
    0xFFFFFFFF  Unknown


Charge-time and runtime usages
------------------------------

Usage ID   Usage Name               Type   Description
--------   -----------------------  -----  -----------
0x0168     Target Charge Level      DV     Requested percentage of Full Charge Capacity, from 1 through 100.
0x0169     Estimated Charge Time    DV     ACPI _BCT result in seconds.
0x016A     Requested Discharge Rate DV     Expected discharge rate in mW or mA; zero selects the current average rate.
0x016B     Estimated Runtime        DV     ACPI _BTM result in seconds.

For Battery Charge Time, the host writes Target Charge Level and then reads the
same Feature report. Estimated Charge Time is read-only:

    0             Target is below current capacity or above 100 percent
    1-0xFFFFFFFE  Estimated charging time in seconds
    0xFFFFFFFF    Unknown

For Battery Runtime, the host writes Requested Discharge Rate and then reads the
same Feature report. Estimated Runtime is read-only:

    0             Requested rate is unsustainable, or battery is critical when
                  Requested Discharge Rate is zero
    1-0xFFFFFFFE  Estimated runtime in seconds
    0xFFFFFFFF    Unknown

Set Feature completion means that the result is available to Get Feature. A
device shall not expose partially updated request/result pairs.


Peak-power usages
-----------------

Usage ID   Usage Name                                      Type
--------   ----------------------------------------------  ----
0x0170     Power Characteristics Revision                  SV
0x0171     Instantaneous Peak Power Threshold Supported    SF
0x0172     Sustainable Peak Power Threshold Supported      SF
0x0173     Maximum Instantaneous Peak Power Threshold      SV
0x0174     Maximum Sustainable Peak Power Threshold        SV
0x0175     Power State Revision                            DV
0x0176     Instantaneous Peak Power Level                  DV
0x0177     Instantaneous Peak Power Period                 DV
0x0178     Sustainable Peak Power Level                    DV
0x0179     Sustainable Peak Power Period                   DV
0x017A     Power Threshold Revision                        DV
0x017B     Power Threshold Identifier                      DV
0x017C     Power Threshold Value                           DV

Power Characteristics usages map to ACPI _BPC revision 1. Maximum threshold
values use mW or mA according to Battery Power Unit.

Power State usages map to ACPI _BPS revision 1. Peak Power Levels use mW or mA.
Peak Power Periods use milliseconds. A level and period of zero means that the
corresponding capability is unsupported. The device sends the Battery Power
State Input report when either capability changes by the configured relative
threshold.

Power Threshold maps to ACPI _BPT revision 1:

    Power Threshold Identifier 0  Clear all thresholds
    Power Threshold Identifier 1  Instantaneous peak power
    Power Threshold Identifier 2  Sustainable peak power

Power Threshold Value is the relative change threshold in mW or mA. Zero
disables the selected threshold. Operation Status in this report is:

    0  Success
    1  Invalid threshold value
    2  Hardware timeout
    3  Unknown hardware error
    4  Unsupported threshold type
    5  Unsupported revision
    6-0xFFFFFFFF  Reserved


Capacity-trip, thermal, and capability usages
---------------------------------------------

Usage ID   Usage Name                  Type   Description
--------   --------------------------  -----  -----------
0x017D     Capacity Trip Point         DV     Remaining-capacity crossing point in the selected capacity unit. Zero clears it.
0x017E     Thermal Throttle Limit      DV     Maximum permitted charging level in percent, from 0 through 100.
0x0180     Host Capability Flags       DV     Host capabilities corresponding to ACPI battery _OSC revision 1 DWORD 2.
0x0181     Device Capability Flags     DV     Device acknowledgement and supported capability bits.

Capacity Trip Point maps to ACPI _BTP. The device sends a status Input report
when Remaining Capacity crosses the programmed value in either direction.

Thermal Throttle Limit maps to ACPI _BTH. At 100 percent the battery may charge
at maximum current. The device is responsible for applying the limit.

Capability negotiation uses the ACPI Control Method Battery UUID
F18FC78B-0F15-4978-B793-53F833A1D35B and revision 1 semantics. Defined flags are:

    Bit 0  Revised battery granularity definition
    Bit 1  Wake on low-battery user preference
    Bit 2  Battery power-delivery threshold notifications
    Bits 3-31  Reserved

The host writes Host Capability Flags. The device returns supported and
accepted bits in Device Capability Flags. Reserved bits shall be zero.


Report contract
===============

The Control Method Battery TLC contains the following reports. Report IDs are
assigned by the descriptor and are not standardized.

Report                                  Kind      Access
--------------------------------------  --------  ------------------------------
Control Method Battery Information      Feature   Device to host
Control Method Battery Identity         Feature   Device to host
Control Method Battery Status           Input     Device to host, on change
Control Method Battery Notification     Feature   Host configuration/readback
Battery Measurement Configuration       Feature   Host configuration/readback
Battery Maintenance                     Feature   Bidirectional control/status
Battery Charge Time                     Feature   Host request/device result
Battery Runtime                         Feature   Host request/device result
Battery Power Characteristics           Feature   Device to host
Battery Power State                     Input     Device to host, on change
Battery Power Threshold                 Feature   Host configuration/readback
Battery Capacity Trip Point             Feature   Host configuration/readback
Battery Thermal Control                 Feature   Host configuration/readback
Battery Capability Negotiation          Feature   Host configuration/readback
Power Consumer List                     Feature   Device to host

Feature reports that contain read-only fields shall ignore host-written values
for those fields. Host-writable fields are applied atomically at successful Set
Feature completion. Get Feature returns the last accepted configuration and the
result derived from that configuration.


Common representation rules
---------------------------

1. Fields are packed as declared by the HID report descriptor. Multi-byte
   fields use HID little-endian bit ordering.
2. ACPI Integer (DWORD) fields use 32 report bits unless a smaller range is
   explicitly defined by this proposal.
3. ACPI DWORD values are interpreted as unsigned. The all-ones bit pattern is
   0xFFFFFFFF even if a descriptor tool displays it as signed -1.
4. Reserved bits are zero when sent and ignored when received.
5. Capacity, rate, and peak-power values use Battery Power Unit. Voltage uses
   millivolts. Time fields state milliseconds or seconds in their definitions.
6. Charging and Discharging shall not both be one.
7. Static information shall describe the battery identified by Battery Tag.
   The host shall discard cached information when Battery Tag changes.
8. If Battery Present is zero, Battery Tag is zero and all other dynamic fields
   are ignored.
9. While Battery Tag is zero, every identity buffer is all zero. Static fields
   are zero or use their defined unknown sentinel. A host shall not cache
   identity or static information read while Battery Tag is zero.


Host operation sequences
------------------------

Enumeration

1. Parse each Control Method Battery TLC.
2. Read Control Method Battery Information and Identity.
3. Read configuration/status Feature reports.
4. Arm Control Method Battery Notification.
5. Consume status and power-state Input reports.

Battery replacement

1. Device sends a status Input report with Battery Present zero and Battery Tag
   zero.
2. When a pack is inserted, the device assigns a new nonzero Battery Tag.
3. Device sends a status Input report with Battery Present one, the new tag, and
   Battery Information Changed one.
4. Host invalidates all cached data and rereads static, identity, maintenance,
   and power-characteristic reports.

Measurement configuration

1. Host reads the supported ranges from Control Method Battery Information.
2. Host sets Sampling Time and Averaging Interval.
3. Host gets Battery Measurement Configuration and verifies Operation Status.

Charge-time or runtime query

1. Host sets the request field in the corresponding Feature report.
2. Device computes and stores the result before completing Set Feature.
3. Host gets the same Feature report and reads the result.


ACPI 6.5 mapping
================

ACPI object  HIDBattery representation
-----------  ---------------------------------------------------------------
_STA         Battery Present; HID transport presence remains independent.
_BIX         Control Method Battery Information and Identity.
_BIF         Represented by the _BIX superset; no separate legacy report.
_BST         Control Method Battery Status Input report.
_BTP         Capacity Trip Point and status Input report on crossing.
_BMS         Sampling Time and Operation Status.
_BMA         Averaging Interval and Operation Status.
_BMD         Battery Maintenance status/capability/count/time fields.
_BMC         Battery Maintenance control fields.
_BCT         Battery Charge Time request/result Feature report.
_BTM         Battery Runtime request/result Feature report.
_BPC         Battery Power Characteristics Feature report.
_BPS         Battery Power State Input report.
_BPT         Battery Power Threshold Feature report and threshold event.
_BTH         Thermal Throttle Limit Feature report.
_OSC         Battery Capability Negotiation Feature report.
_PCL         Power Consumer List Feature report.


Windows battery miniclass mapping
=================================

Battery miniclass operation         HIDBattery source
----------------------------------  ------------------------------------------
BatteryMiniQueryTag                 Battery Present and Battery Tag.
BatteryMiniQueryStatus              Status flags, Remaining Capacity, Voltage, and Present Rate.
BatteryMiniQueryInformation         Static information, Identity, Temperature, Manufacture Date, Cycle Count, and Estimated Runtime.
BatteryMiniSetInformation           Critical Bias and applicable Battery Maintenance controls.
BatteryMiniSetStatusNotify          Notification Power State Mask and low/high capacity bounds.
BatteryMiniDisableStatusNotify      Clear the Control Method Battery Notification report.

BATTERY_INFORMATION mapping:

Windows field          HIDBattery source
---------------------  -------------------------------------------------------
Capabilities           Battery Present, Secondary Battery Technology, and driver policy.
Technology             Primary or Secondary Battery Technology.
Chemistry              First four ASCII bytes of Battery Type String, padded with spaces.
DesignedCapacity       Design Capacity.
FullChargedCapacity    Full Charge Capacity.
DefaultAlert1          Design Capacity Low.
DefaultAlert2          Warning Capacity Limit.
CriticalBias           Critical Bias.
CycleCount             Cycle Count.

BATTERY_STATUS mapping:

Windows field   HIDBattery source
--------------  -------------------------------------------------------------
PowerState      System On Line, Charging, Discharging, and Battery Critical.
Capacity        Remaining Capacity.
Voltage         Voltage.
Rate            Present Rate, positive while charging and negative while discharging.

When Battery Power Unit selects milliampere-hours/milliamperes, a Windows
miniclass converts every Windows-facing capacity or rate quantity using Voltage
when the Windows API requires power units. This includes DesignedCapacity,
FullChargedCapacity, DefaultAlert1, DefaultAlert2, CriticalBias, status Capacity
and Rate, and notification low/high bounds. If a reliable conversion is not
possible, the driver uses the Windows relative-capacity capability consistently
for capacities, alert thresholds, critical bias, and notification bounds while
preserving their ordering and threshold semantics.


Sample descriptor
=================

The accompanying HIDBattery.wara file is a non-normative Waratah 1.9 source
descriptor for this proposal. It:

* extends the built-in Battery System page with the proposed usages;
* creates one Control Method Battery TLC;
* reuses the existing Power and Battery System usages listed above;
* contains the complete report surface described by this proposal; and
* compiles with WaratahCmd 1.9 against HID Usage Tables 1.7.

Waratah 1.9 cannot declare the complete unsigned 32-bit logical range required
by ACPI while also generating the all-ones sentinel. The sample therefore uses
a signed logical range for raw ACPI DWORD fields, and some non-sentinel fields
are optimized to fewer than 32 bits. The sample validates report organization
and usage compatibility, but it is not a normative firmware descriptor for
values from 0x80000000 through 0xFFFFFFFE. A production descriptor shall follow
the Common representation rules above and encode the complete 32-bit field.


References
==========

1. USB-IF, HID Usage Tables 1.7, Power Page (0x84) and Battery System Page
   (0x85): https://usb.org/sites/default/files/hut1_7.pdf
2. UEFI Forum, ACPI Specification 6.5, section 10.2.2, Battery Control Methods:
   https://uefi.org/specs/ACPI/6.5/10_Power_Source_and_Power_Meter_Devices.html#battery-control-methods
3. Microsoft, Writing Battery Miniclass Drivers:
   https://learn.microsoft.com/windows-hardware/drivers/battery/writing-battery-miniclass-drivers
4. Microsoft, hidtools / Waratah:
   https://github.com/microsoft/hidtools
5. USB-IF, HUTRR submission template and process:
   https://www.usb.org/sites/default/files/hutrr_form.txt


Response
========

<Filled during HID Working Group review.>
