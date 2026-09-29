use defmt::{info, warn};
use embassy_qemu_riscv::espi::{Async, Espi, Mailbox};

const MAILBOX: Mailbox = Mailbox::Mailbox0;
const LENGTH_OFFSET: usize = 8;
const HEADER_SIZE: usize = 16;
const PAYLOAD_OFFSET: usize = HEADER_SIZE;
const PING_COMMAND: u32 = 1;
const NOTIFY_ON_COMPLETION: u32 = 1;
const COMPLETE: u32 = 1;
const ERROR: u32 = 2;

#[repr(C)]
#[derive(defmt::Format)]
struct ExtendedPccHeader {
    signature: u32,
    flags: u32,
    length: u32,
    command: u32,
}

const _: () = assert!(core::mem::size_of::<ExtendedPccHeader>() == HEADER_SIZE);

impl ExtendedPccHeader {
    fn from_bytes(bytes: [u8; HEADER_SIZE]) -> Self {
        Self {
            signature: u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            flags: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            length: u32::from_le_bytes(bytes[8..12].try_into().unwrap()),
            command: u32::from_le_bytes(bytes[12..16].try_into().unwrap()),
        }
    }
}

// This task demonstrates the basic response flow of the EC on the type 3 subspace.
//
// See: https://uefi.org/specs/ACPI/6.5/14_Platform_Communications_Channel.html#doorbell-protocol
// for a diagram and explanation of the flow.
//
// Currently, type 4 subspace (async notifications) is not demonstrated here.
#[embassy_executor::task]
pub async fn task(mut espi: Espi<'static, Async>) {
    // Ensure initially the CC bit is set (the host reads this to know it can send a command)
    // Corresponds to step 1 in diagram
    espi.set_shared_status(MAILBOX, COMPLETE);
    info!("PCC ping responder ready");

    loop {
        // Wait for doorbell from host
        let events = espi.wait_for_events().await;
        if events.doorbells[0] {
            // Ensure the host has cleared the CC bit
            // Corresponds to optional step 5 in the diagram
            if espi.mailbox_status(MAILBOX).shared_status & COMPLETE != 0 {
                warn!("PCC doorbell ignored: command complete is still set");
                espi.acknowledge_doorbell(MAILBOX);
            } else {
                let mut header_bytes = [0u8; HEADER_SIZE];
                // Extract the header from the shared memory region which tells us what the command is
                // (as well as flags that tell us if the host wants to be interrupted)
                //
                // See: https://uefi.org/specs/ACPI/6.5/14_Platform_Communications_Channel.html#extended-pcc-subspace-shared-memory-region
                //
                // Note: Length will be reported as 0 because the Windows interface I go through
                // on the host side to talk to the PCC engine does not write length.
                let header = if espi.read_mailbox(MAILBOX, 0, &mut header_bytes).is_ok() {
                    let header = ExtendedPccHeader::from_bytes(header_bytes);
                    info!("PCC request header: {}", header);
                    Some(header)
                } else {
                    None
                };
                let notify_on_completion = header
                    .as_ref()
                    .is_some_and(|header| header.flags & NOTIFY_ON_COMPLETION != 0);

                // Now extract the payload for the command and process it
                // Corresponds to step 6 in the diagram
                let mut payload = [0u8; 8];
                let valid = header.is_some_and(|header| header.command == PING_COMMAND)
                    && espi.read_mailbox(MAILBOX, PAYLOAD_OFFSET, &mut payload).is_ok()
                    && &payload[..4] == b"PING";

                if valid {
                    payload[..4].copy_from_slice(b"PONG");
                    info!(
                        "PCC PING sequence={} PONG",
                        u32::from_le_bytes(payload[4..8].try_into().unwrap())
                    );
                } else {
                    payload = [0; 8];
                    warn!("Invalid PCC ping request");
                }

                // Here we write the response back into the same type 3 shared memory region
                // Also corresponds to step 6 in the diagram
                espi.write_mailbox(MAILBOX, PAYLOAD_OFFSET, &payload)
                    .expect("PCC response payload is in bounds");
                let length = if valid { 4 + payload.len() as u32 } else { 4 };
                espi.write_mailbox(MAILBOX, LENGTH_OFFSET, &length.to_le_bytes())
                    .expect("PCC response length is in bounds");
                espi.acknowledge_doorbell(MAILBOX);

                // If the command was valid, set the CC bit which the host will check
                // Corresponds to step 7 in the diagram
                espi.set_shared_status(MAILBOX, COMPLETE | if valid { 0 } else { ERROR });

                // Finally if the host requested to be interrupted, then notify the host (via vwire under the hood)
                // Corresponds to step 8
                if notify_on_completion {
                    espi.raise_host_irq(MAILBOX);
                }
            }
        }

        // As mentioned, we aren't really doing anything for mailbox1 (type 4 subspace) yet
        if events.doorbells[1] {
            espi.acknowledge_doorbell(Mailbox::Mailbox1);
            warn!("Unexpected PCC mailbox 1 doorbell");
        }
    }
}
