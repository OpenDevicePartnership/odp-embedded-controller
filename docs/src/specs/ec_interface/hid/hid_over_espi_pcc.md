# HID over eSPI PCC

## Scope

This document defines a possible HID protocol overlay on the function-neutral [eSPI PCC transport](../espi/espi_pcc_draft.md). It maps HID operations onto the transport profiles defined by that specification without changing their ownership, signaling, or completion semantics.

The eSPI PCC specification remains authoritative for:

- Type 3 and Type 4 ownership and completion.
- Shared-memory and PCCT layout.
- VWire-to-GSI interrupt mapping.
- Logical sub-channel allocation.
- Payload-size, ordering, and hardware-resource requirements.

This overlay defines only HID-specific message mapping and binding requirements. Generic PCC transport questions are tracked in the base specification.

## HID Payload

The PCC payload requires a HID header that identifies the HID message or command type, Report Type, Report ID, and payload length. Responses may also carry HID operation status distinct from transport-level PCC errors. The exact HID field layout and command values remain to be defined.

Descriptor and large Feature Report transfer depends on the resolution of [PCC Region Size versus eSPI Maximum Payload Size](../espi/espi_pcc_draft.md#pcc-region-size-versus-espi-maximum-payload-size).

## Recommended Synchronous Profile

The recommended mapping uses synchronous Type 3 for HID requests and responses and independent Type 4 for unsolicited Input and Wake notifications.

| HID operation | Type 3 request and response | Type 4 notification |
|---|---|---|
| Device and Report Descriptor discovery | Host requests a descriptor; EC returns it in the same Type 3 region | None |
| Get Input Report or Get Feature Report | Host supplies Report Type and Report ID; EC returns the current report | None |
| Set Output Report or Set Feature Report | Host supplies Report Type, Report ID, and report data; EC returns status | None |
| Input Report | None | EC publishes Report ID and report data |
| Reset | Host requests a logical HID reset and waits for completion | Optional device-initiated reset indication |
| Set Power ON, SLEEP, or OFF | Host sends the requested state; Command Complete provides transport completion | None |
| Wake | None | EC publishes a Wake notification while the device is in SLEEP |
| Get/Set Idle or Get/Set Protocol | Optional Type 3 commands for USB boot-device compatibility | None |

### HID Binding Requirements

The base transport specification tracks the generic [notification queue](../espi/espi_pcc_draft.md#notification-queue-semantics), [reset](../espi/espi_pcc_draft.md#reset-and-stale-message-handling), [power and wake](../espi/espi_pcc_draft.md#power-state-and-wake-behavior), and [timeout and error](../espi/espi_pcc_draft.md#timeout-and-error-semantics) questions. After those contracts are defined, the HID binding must:

- Classify which HID reports require lossless delivery and which may be coalesced.
- Define HID descriptor, report, and protocol state after a transport-generation change.
- Map HID ON, SLEEP, OFF, and Wake behavior onto the transport's ACPI power-state and wake rules.
- Map HID operation failures and any operation-specific timeout requirements onto the transport status and recovery model.

### Shared-Pair Specialization

The Shared-Pair Specialization uses the same HID mapping but requires a Function ID in every request and notification. HID shares the Type 3 ownership token and Type 4 notification slot with other EC functions, creating head-of-line blocking and notification-scheduling risks. A latency-sensitive HID function may therefore require a dedicated pair.

## Component Firmware Update

HID firmware update should use the Microsoft Component Firmware Update (CFU) protocol rather than define a PCC-specific firmware-update command. The overlay transports CFU reports without redefining firmware-version discovery, offer and acceptance handling, component and segment identifiers, host tokens, content sequence numbers, per-block status, image validation results, or busy/readiness behavior.

In the recommended synchronous profile, CFU Get Feature, Output Report, and associated response exchanges use Type 3. CFU content blocks are individually acknowledged; a firmware image is not represented as one long-running PCC transaction.

The CFU `OFFER_NOTIFY_ON_READY` response may be delayed, but remains a solicited response. The synchronous profile therefore keeps Type 3 occupied until that response arrives. If delayed readiness must release Type 3, the decoupled profile carries it as a Type 4 Response with a transport transaction ID, not as an unsolicited Notification.

CFU does not resolve the PCC transport questions. The binding must still define how each CFU HID report is encoded in the PCC HID payload. Whether a CFU report plus PCC framing fits in one transfer depends on [PCC Region Size versus eSPI Maximum Payload Size](../espi/espi_pcc_draft.md#pcc-region-size-versus-espi-maximum-payload-size).

## Decoupled Type 3 and Type 4 Profile

In the [decoupled profile](../espi/espi_pcc_draft.md#decoupled-type-3-requests-and-type-4-responses), Type 3 reports request acceptance while solicited responses and unsolicited notifications share Type 4.

| HID operation | Type 3 path | Type 4 path |
|---|---|---|
| Descriptor discovery and Get Report | Request is accepted and assigned a transaction ID | Response carries the matching transaction ID and returned data |
| Set Report, Output Report, Reset, or Set Power | Request is accepted and assigned a transaction ID | Response carries final status when the operation completes |
| Input Report or Wake | None | Notification uses the Notification message class and reserved transaction-ID value |
| CFU HID report | Request acceptance releases Type 3 before processing finishes | Response uses the original transport transaction ID; CFU tokens and sequence numbers remain part of the CFU payload |

### HID Binding Requirements

The generic decoupled request lifecycle, timeout, error, reset, and Type 4 scheduling rules belong to the [base transport specification](../espi/espi_pcc_draft.md#decoupled-request-lifecycle). The HID binding must define only how HID semantics use those rules:

- SLEEP and OFF operations need an explicit terminal-acceptance rule if no later HID Response is expected.
- CFU tokens and content sequence numbers must not replace the decoupled transport transaction ID.
- The binding must identify which HID operations may use decoupled completion instead of the recommended synchronous profile.

## Type 3-Only Interrupt-and-Pull Profile

The [Type 3-only profile](../espi/espi_pcc_draft.md#type-3-only-interrupt-and-pull) consists of a Type 3 subspace plus a dedicated notification interrupt. The notification interrupt is separate from Type 3 command completion and can be implemented with a dedicated eSPI Interrupt Event VWire mapped by the host controller to a GSI. The EC asserts it when a HID event is pending, and the host retrieves the event with a Type 3 request.

| HID operation | Type 3 path | Event indication |
|---|---|---|
| Descriptor discovery, Get/Set Report, Reset, or Set Power | Synchronous request and response, as in the recommended profile | None |
| Input Report | Host sends `GET_PENDING_NOTIFICATION`; EC returns the next Input Report | EC asserts the dedicated notification VWire/GSI |
| Wake | Host sends `GET_PENDING_NOTIFICATION`; EC returns a Wake event | EC asserts the dedicated notification VWire/GSI |
| CFU HID report | Each CFU report uses a synchronous Type 3 request and response | Pending HID input cannot be retrieved while a CFU report occupies Type 3 |

This profile resembles HID over I2C's interrupt followed by an Input Register read.

### HID Binding Requirements

The dedicated interrupt, ACPI advertisement, queue, event framing, and Type 3 arbitration rules belong to the [base transport specification](../espi/espi_pcc_draft.md#type-3-only-notification-retrieval). The HID binding must:

- Assign HID event-type values for Input Report, Wake, and device-reset notifications.
- Define the HID payload returned by `GET_PENDING_NOTIFICATION`, including its event type, Report ID, and report data.

The additional request/response round trip and single Type 3 ownership token make this profile unsuitable for high-rate touch, pen, or precision-touchpad traffic.

## References

- [eSPI PCC Specification](../espi/espi_pcc_draft.md)
- [USB Device Class Definition for Human Interface Devices, Version 1.11](https://www.usb.org/sites/default/files/hid1_11.pdf)
- [Microsoft HID over I2C Protocol Specification, Version 1.0](https://download.microsoft.com/download/7/D/D/7DD44BB7-2A7A-4505-AC1C-7227D3D96D5B/hid-over-i2c-protocol-spec-v1-0.docx)
- [Microsoft Component Firmware Update Protocol Specification](https://learn.microsoft.com/en-us/windows-hardware/drivers/cfu/cfu-specification)
