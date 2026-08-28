Request \#:        HUTRR120
Title:            RTC & System Wake Timer
Spec Release:    1.7
Requester:        Phil Weber
Company:          Microsoft
------------------
Pages Affected:  Time and Date (0x13) (*New Page*)
Generic Desktop (0x01)
Generic Device Controls (0x06)
------------------
Current Status:   In Review
Priority:         Normal
------------------
Required Voter:   QC
Required Voter:   Intel
Required Voter:   Dell
------------------

Voting Begins:    \<Filled by HID-Chairman\>
Voting Ends:      \<Filled by HID-Chairman\>
Voting Result:    \<Filled by HID-Chairman\>

**Summary:**

Add new Usages to support configuring a system's onboard RTC (Real-Time-Clock) and System Wake Timer.

**Scenario:**

CPU timers tend to drift and lack time-keeping capability during sleep/shutdown. An **RTC** (Real-Time-Clock, distinct from the CPU-Clock) is a dedicated, battery-powered device (typically integrated into motherboards/chipsets) to maintain a ‘wall-clock’ time+date with high precision. Commonly used to provide current time+date to the user, data timestamping, task-scheduling and a basis for authentication.

The pairing with a battery/super-capacitor, ensures accurate time+date is kept while the CPU is not running. While this device is very-accurate and typically factory-initialized, it still requires regular updates from the system to account for drift (e.g. using NTP), timezone changes and addition of leap-seconds. The system typically retrieves the current time from the device on returning to its working-state.

The **system wake-timer** device provides a CPU-external mechanism to signal a ‘wake’. It is commonly implemented as an ‘always on’ low-power hardware block in the chipset/SoC or directly into the platform’s power-management/wake controller.

When configured, the device will wake the system at the specified future time after the timer expires. This enables the system to save power by sleeping during idle periods and then resume to perform scheduled tasks.

**Proposal:**

Add to Table 9.1 – Generic Device Controls Page

| **Usage Id** | **Usage Name**       | **Usage Types** |
|--------------|----------------------|-----------------|
| 0x46-4F      | *Reserved*           |                 |
| 0x50         | Vendor Current State | DV              |
| 0x51         | Current State        | NAry            |
| 0x52         | Cleared              | Sel             |
| 0x53         | Failed               | Sel             |
| 0x54         | Running              | Sel             |
| 0x55         | Expired              | Sel             |
| 0x56         | Signaled             | Sel             |

Add a new section to Generic Device Controls with the following details.

*Reviewer Note: These Usages are reminiscent of ‘23.3 Sensor State Event Usages’. New Usages have been declared since Sensor Events are expected to be used with Modifiers. Modifiers are not applicable to non-sensor scenarios, so new, dedicated Usages shall be created.*

9.5 Device State

Some devices can fail requested operations (though the Report is successfully sent/received) because of underlying hardware faults, loss of backing resources, invalid configuration, or other device-specific conditions that are not visible through transport-level success alone. These usages allow the system to determine whether a requested operation succeeded, failed, or requires recovery.

<table style="width:100%;">
<colgroup>
<col style="width: 27%" />
<col style="width: 16%" />
<col style="width: 56%" />
</colgroup>
<thead>
<tr>
<th><strong>Usage Name</strong></th>
<th><strong>Usage Types</strong></th>
<th><strong>Usage Description</strong></th>
</tr>
</thead>
<tbody>
<tr>
<td>Current State</td>
<td>NAry</td>
<td>The most-recent state</td>
</tr>
<tr>
<td>Cleared</td>
<td>Sel</td>
<td>State is uninitialized or recently reset/cleared.</td>
</tr>
<tr>
<td>Failed</td>
<td>Sel</td>
<td><p>Failed to perform the most recent operation. (Note: HID-transport functionality is still clearly working)</p>
<p>Common for when the device is actually backed by another device that can experience failures</p></td>
</tr>
<tr>
<td>Expired</td>
<td>Sel</td>
<td>Expired (e.g. timer has completed)</td>
</tr>
<tr>
<td>Signaled</td>
<td>Sel</td>
<td>Signaled (e.g. external interrupt raised)</td>
</tr>
<tr>
<td>Running</td>
<td>Sel</td>
<td>Running (e.g. everything behaving normally)</td>
</tr>
</tbody>
</table>

<table style="width:100%;">
<colgroup>
<col style="width: 27%" />
<col style="width: 16%" />
<col style="width: 56%" />
</colgroup>
<thead>
<tr>
<th><strong>Usage Name</strong></th>
<th><strong>Usage Types</strong></th>
<th><strong>Usage Description</strong></th>
</tr>
</thead>
<tbody>
<tr>
<td>Vendor Current State</td>
<td>DV</td>
<td><p>Opaque state-value meaningful only to the device vendor.</p>
<p>Typically used in conjunction with ‘Current State’</p></td>
</tr>
</tbody>
</table>

Create new page ‘Time and Date’ (0x13).

*Reviewer Note: These Usages are reminiscent of ‘23.15 Time Sensor Field Usages’. New Usages have been declared since Sensor Data Fields are expected to be used with Modifiers. Modifiers are not applicable to non-sensor scenarios, so new, dedicated Usages shall be created.*

22 - Time and Date (0x13)

Common Usages to describe times and dates.

| **Usage Id** | **Usage Name**                 | **Usage Types** |
|--------------|--------------------------------|-----------------|
| 0x00         | *Undefined*                    |                 |
| 0x01         | Year                           | DV              |
| 0x02         | Month                          | DV              |
| 0x03         | Day                            | DV              |
| 0x04         | Hour                           | DV              |
| 0x05         | Minute                         | DV              |
| 0x06         | Second                         | DV              |
| 0x07         | Millisecond                    | DV              |
| 0x08-F       | *Reserved*                     |                 |
| 0x10         | Time Zone Offset From UTC      | DV              |
| 0x11         | Daylight Savings Time Observed | DF              |
| 0x12         | Daylight Savings Time Active   | DF              |

22.1 Date

Conventional UTC (proleptic Gregorian) calendar divisions.

<table>
<colgroup>
<col style="width: 17%" />
<col style="width: 19%" />
<col style="width: 63%" />
</colgroup>
<thead>
<tr>
<th><strong>Usage Name</strong></th>
<th><strong>Usage Types</strong></th>
<th><strong>Usage Description</strong></th>
</tr>
</thead>
<tbody>
<tr>
<td>Year</td>
<td>DV</td>
<td><p>Standard UTC Gregorian calendar year.</p>
<p>Positive integers for AD and negative for BC.</p>
<p>0 is undefined in the Gregorian calendar.</p></td>
</tr>
<tr>
<td>Month</td>
<td>DV</td>
<td>Standard UTC Gregorian calendar month</td>
</tr>
<tr>
<td>Day</td>
<td>DV</td>
<td>Standard UTC Gregorian calendar day of the month</td>
</tr>
</tbody>
</table>

22.2 Time

22.2.1 Conventional UTC time (Coordinated Universal Time) divisions.

| **Usage Name** | **Usage Types** | **Usage Description**    |
|----------------|-----------------|--------------------------|
| Hour           | DV              | Standard UTC hour        |
| Minute         | DV              | Standard UTC minute      |
| Second         | DV              | Standard UTC second      |
| Millisecond    | DV              | Standard UTC millisecond |

22.2.2 Time zone and Daylight Savings Time (DST).

<table>
<colgroup>
<col style="width: 37%" />
<col style="width: 15%" />
<col style="width: 46%" />
</colgroup>
<thead>
<tr>
<th><strong>Usage Name</strong></th>
<th><strong>Usage Types</strong></th>
<th><strong>Usage Description</strong></th>
</tr>
</thead>
<tbody>
<tr>
<td>Time Zone Offset From UTC</td>
<td>DV</td>
<td><p>Minutes local time is behind (-) or ahead (+) of UTC+00:00</p>
<p>Offset = (localTime – UTC)<br />
e.g. AEST timezone of UTC+10:00 is ahead +600 minutes.</p>
<p>e.g. EST timezone of UTC-05:00 is behind -300 minutes.</p></td>
</tr>
<tr>
<td>Daylight Savings Time Observed</td>
<td>DF</td>
<td>Indicates if DST is observed in current time zone</td>
</tr>
<tr>
<td>Daylight Savings Time Active</td>
<td>DF</td>
<td><p>Indicates if time has been adjusted by system to apply DST.</p>
<p>Useful when the device is shared to coordinate applying/removing DST.</p></td>
</tr>
</tbody>
</table>

Add to Table 4.1 – Generic Desktop Page

| **Usage Id** | **Usage Name**                         | **Usage Types** |
|--------------|----------------------------------------|-----------------|
|              |                                        |                 |
| 0x14         | **System Wake Timer**                  | CA              |
| 0x15         | **System Real Time Clock**             | CA              |
|              |                                        |                 |
| 0xF0         | Timer Expiration: External Power       | DV              |
| 0xF1         | Timer Expiration: Internal Power       | DV              |
| 0xF2         | Power Source Change Minimum Expiration | DV              |
| 0xF3         | Lowest System Wakeable Power State     | NAry            |
| 0xF4         | S1                                     | Sel             |
| 0xF5         | S2                                     | Sel             |
| 0xF6         | S3                                     | Sel             |
| 0xF7         | S4                                     | Sel             |
| 0xF8         | S5                                     | Sel             |
| 0xF9-FFFF    | *Reserved*                             |                 |

Add 4.17 System Wake Timer

The **system wake-timer** device provides a CPU-external mechanism to signal a ‘wake’. It is commonly implemented as an ‘always on’ low-power hardware block in the chipset/SoC or directly into the platform’s power-management/wake controller.

When configured, the device will wake the system (via the above out-of-band means) at the specified future time after the timer expires. This enables the system to save power by sleeping during idle periods and then resume to perform scheduled tasks.

Separate wake configurations may be made for whether the system is running on External (i.e. AC/wall) power or Internal (i.e. DC/battery) power. It is up to the device to determine the power source and choose the right configuration, switching as necessary.

There is typically a single system wake timer available to the system.

<table>
<colgroup>
<col style="width: 34%" />
<col style="width: 17%" />
<col style="width: 48%" />
</colgroup>
<thead>
<tr>
<th><strong>Usage Name</strong></th>
<th><strong>Usage Types</strong></th>
<th><strong>Usage Description</strong></th>
</tr>
</thead>
<tbody>
<tr>
<td><strong>System Wake Timer</strong></td>
<td>CA</td>
<td>A configurable timer, capable of waking the system when in a low-power state.</td>
</tr>
<tr>
<td>Timer Expiration: External Power</td>
<td>DV</td>
<td>Timer expiration when device is connected to external (i.e. AC/wall) power.</td>
</tr>
<tr>
<td>Timer Expiration: Internal Power</td>
<td>DV</td>
<td>Timer expiration when device is connected to internal (i.e. DC/battery) power.</td>
</tr>
<tr>
<td>Power Source Change Minimum Expiration</td>
<td>DV</td>
<td><p>Additional time the wake-timer must wait for, when the power-source changes (e.g. from external to internal) while the system is in Sx and the timer has already expired.</p>
<p>(e.g. System enters Sx while on external-power with internal-power timer expired. While in Sx, power-source changes to internal, must wait for additional period before signaling wake.)</p></td>
</tr>
</tbody>
</table>

4.17.1 Lowest System Wakeable Power State

By ACPI convention, S0 is the working-state, and S1-5 (i.e. Sx) is a low-power state, (where the larger the digit, the lower the power).

<table>
<colgroup>
<col style="width: 45%" />
<col style="width: 17%" />
<col style="width: 37%" />
</colgroup>
<thead>
<tr>
<th><strong>Usage Name</strong></th>
<th><strong>Usage Types</strong></th>
<th><strong>Usage Description</strong></th>
</tr>
</thead>
<tbody>
<tr>
<td>Lowest System Wakeable Power State</td>
<td>NAry</td>
<td>Indicates the lowest power state the system can enter and still be signaled awake.</td>
</tr>
<tr>
<td>S1</td>
<td>Sel</td>
<td>Standard ACPI S1 sleep state</td>
</tr>
<tr>
<td>S2</td>
<td>Sel</td>
<td>Standard ACPI S2 sleep state</td>
</tr>
<tr>
<td>S3</td>
<td>Sel</td>
<td>Standard ACPI S3 sleep state</td>
</tr>
<tr>
<td>S4</td>
<td>Sel</td>
<td><p>Standard ACPI S4 (Hibernate) sleep state.</p>
<p>System context saved, hardware mostly off</p></td>
</tr>
<tr>
<td>S5</td>
<td>Sel</td>
<td><p>Standard ACPI S5 (soft off) sleep state.</p>
<p>System context lost, requires full reboot.</p></td>
</tr>
</tbody>
</table>

4.17.2 Timer Expiration Rules

1.  Upon configuration, the wake timer immediately begins counting-down (i.e. while in S0).

2.  A wake shall be signalled, ONLY when the system is in Sx (i.e. a non-working state). The lowest such state is defined by ‘Lowest System Wakeable Power State’

3.  If the system is in Sx, and the wake timer expires, the system is signalled to wake.

4.  If the system is in S0, and the wake timer expires, no wake is signalled.

5.  If the system enters Sx with an expired wake timer, the system is immediately signalled to wake.

6.  Once expired, the wake timer remains expired until it is cleared or re-configured. (i.e. entering S0 or signalling wake does NOT clear the timer)

7.  If the system changes power-source while in Sx, and the associated timer has already expired, ‘Power Source Change Minimum Expiration’ must additionally be waited before signalling to wake.

Add 4.18 System Real Time Clock (RTC)

CPU timers tend to drift and lack time-keeping capability during sleep/shutdown. An **RTC** (Real-Time-Clock, distinct from the CPU-Clock) is a dedicated, battery-powered device (typically integrated into motherboards/chipsets) to maintain a ‘wall-clock’ time+date with high precision. Commonly used to provide current time+date to the user, data timestamping, task-scheduling and a basis for authentication.

The pairing with a battery/super-capacitor, ensures accurate time+date is kept while the CPU is not running. While this device is typically factory-initialized, it requires regular system updates to account for drift (e.g. using NTP), timezone changes and addition of leap-seconds. The system typically retrieves the current time from the device on returning to its working-state.

There is typically a single RTC available to the system.

| **Usage Name**             | **Usage Types** | **Usage Description**      |
|----------------------------|-----------------|----------------------------|
| **System Real Time Clock** | CA              | The system accessible RTC. |

**Sample Descriptor .wara:**

The below sample describes a System Wake Timer and a System Real Time Clock.

System Wake Timer Reports:-

<table>
<colgroup>
<col style="width: 13%" />
<col style="width: 13%" />
<col style="width: 11%" />
<col style="width: 60%" />
</colgroup>
<thead>
<tr>
<th><strong>Report Type</strong></th>
<th></th>
<th><strong>Report Id</strong></th>
<th><strong>Description</strong></th>
</tr>
</thead>
<tbody>
<tr>
<td>Feature</td>
<td></td>
<td>1</td>
<td>Describes the lowest system power state this wake timer can wake the system.</td>
</tr>
<tr>
<td>Output</td>
<td></td>
<td>1</td>
<td><p>Wait timer configuration.</p>
<p>Setting a null-value clears the configuration, stopping any active countdown.</p>
<p>Must request corresponding Input report to validate configuration was applied successfully, as underlying hardware may fail</p></td>
</tr>
<tr>
<td>Input</td>
<td></td>
<td>1</td>
<td><p>Current wake-timer configuration.</p>
<p>Must be requested explicitly by system. Will NOT be sent as Interrupt.</p>
<p>Examine ‘Current State’ to determine if configuration was successfully applied.</p>
<ul>
<li><p>Cleared (wake-timer is NOT running)</p></li>
<li><p>Failed (could NOT apply requested configuration, timer is NOT running)</p></li>
<li><p>Running (successfully applied configuration, timer is counting-down)</p></li>
<li><p>Expired (wake-timer has expired but NOT signaled a wake)</p></li>
<li><p>Signaled (expired wake has been signaled)</p></li>
</ul></td>
</tr>
</tbody>
</table>

System Real Time Clock Reports:-

<table>
<colgroup>
<col style="width: 16%" />
<col style="width: 13%" />
<col style="width: 69%" />
</colgroup>
<thead>
<tr>
<th>Output</th>
<th>3</th>
<th><p>Sets the RTC.</p>
<p>This device supports a resolution of 1 millisecond.</p>
<p>Must request corresponding Input report to validate configuration was applied successfully, as underlying hardware may fail.</p></th>
</tr>
</thead>
<tbody>
<tr>
<td>Input</td>
<td>3</td>
<td><p>Retrieves the current RTC time.</p>
<p>Examine ‘Current State’ to determine if previously set time was successfully applied.</p>
<ul>
<li><p>Failed (could NOT apply requested time, current time is invalid)</p></li>
<li><p>Running (successfully applied time)</p></li>
</ul></td>
</tr>
</tbody>
</table>