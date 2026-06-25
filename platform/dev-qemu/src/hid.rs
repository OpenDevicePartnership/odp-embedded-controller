//! Minimal HID-over-I2C device.
//!
//! Just enough to satisfy a host (e.g. Windows) performing the initial
//! HID-over-I2C handshake: serve the HID descriptor and report descriptor,
//! acknowledge `RESET` (asserting the interrupt so the host reads the all-zero
//! reset sentinel), and ack the remaining setup commands.
//!
//! This also demonstrates the basic setup of a HID-over-I2C device on the QEMU EC platform
//! for further development/testing.

use core::borrow::BorrowMut;
use defmt::error;
use embassy_qemu_riscv::gpio::Output;
use embassy_qemu_riscv::i2c::target::{Async, I2c};
use embedded_mcu_hal::i2c::target::Request as TargetRequest;
use embedded_services::hid::{self, Descriptor, DeviceId, RegisterFile, Request, Response};
use embedded_services::{comms, define_static_buffer};
use hid_service::i2c::{Command, Host, HostConfig, I2cSlaveAsync};
use static_cell::StaticCell;

// HID device ID used to route requests between the I2C host bridge and the device.
const HID_DEVICE_ID: DeviceId = DeviceId(0);

// Vendor / product / version IDs reported in the HID descriptor.
const HID_VID: u16 = 0x045E; // MSFT
const HID_PID: u16 = 0x0002;
const HID_VERSION: u16 = 0x0100;

// Minimal vendor-defined report descriptor: a single 1-byte input report.
//
// It carries no useful data; it only needs to be a well-formed HID report
// descriptor so the host can parse it and finish enumerating the device.
#[rustfmt::skip]
const REPORT_DESCRIPTOR: &[u8] = &[
    0x06, 0x00, 0xFF, // Usage Page (Vendor Defined 0xFF00)
    0x09, 0x01,       // Usage (0x01)
    0xA1, 0x01,       // Collection (Application)
    0x09, 0x01,       //   Usage (0x01)
    0x15, 0x00,       //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x75, 0x08,       //   Report Size (8)
    0x95, 0x01,       //   Report Count (1)
    0x81, 0x02,       //   Input (Data, Variable, Absolute)
    0xC0,             // End Collection
];

// Max input report length advertised in the descriptor: 2-byte length prefix + 1 data byte.
const INPUT_MAX_LEN: usize = 3;

// Build the HID descriptor from the device register layout.
fn descriptor(regs: &RegisterFile) -> Descriptor {
    Descriptor {
        w_hid_desc_length: hid::DESCRIPTOR_LEN as u16,
        bcd_version: HID_VERSION,
        w_report_desc_length: REPORT_DESCRIPTOR.len() as u16,
        w_report_desc_register: regs.report_desc_reg,
        w_input_register: regs.input_reg,
        w_max_input_length: INPUT_MAX_LEN as u16,
        w_output_register: regs.output_reg,
        w_max_output_length: 0,
        w_command_register: regs.command_reg,
        w_data_register: regs.data_reg,
        w_vendor_id: HID_VID,
        w_product_id: HID_PID,
        w_version_id: HID_VERSION,
    }
}

struct HidI2cSlave(I2c<'static, Async>);
impl I2cSlaveAsync for HidI2cSlave {
    type Error = core::convert::Infallible;

    async fn listen(&mut self) -> Result<Command, Self::Error> {
        loop {
            match self.0.listen().await {
                TargetRequest::Read(_) => return Ok(Command::Read),
                TargetRequest::Write(_) => return Ok(Command::Write),
                _ => continue,
            }
        }
    }

    async fn respond_to_write(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let _ = self.0.respond_to_write(buf).await;
        Ok(())
    }

    async fn respond_to_read(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        let _ = self.0.respond_to_read(buf).await;
        Ok(())
    }
}

#[embassy_executor::task]
pub async fn host_task(i2c: I2c<'static, Async>) {
    define_static_buffer!(host_buf, u8, [0u8; 256]);
    let buf = host_buf::get_mut().expect("HID host buffer must not already be borrowed");

    static HOST: StaticCell<Host<HidI2cSlave>> = StaticCell::new();
    let host = HOST.init(Host::new(HID_DEVICE_ID, HidI2cSlave(i2c), buf, HostConfig::default()));

    comms::register_endpoint(host, &host.tp)
        .await
        .expect("HID host endpoint already registered");

    loop {
        match host.process().await {
            Ok(()) => {}
            Err(hid_service::Error::Bus(_)) => error!("HID host I2C bus error"),
            Err(hid_service::Error::Hid(e)) => error!("HID host error: {:?}", e),
            Err(hid_service::Error::Buffer(e)) => error!("HID host buffer error: {:?}", e),
        }
    }
}

#[embassy_executor::task]
pub async fn device_task(mut hid_int: Output<'static>) {
    let regs = RegisterFile::default();

    static DEVICE: StaticCell<hid::Device> = StaticCell::new();
    let device = DEVICE.init(hid::Device::new(HID_DEVICE_ID, regs));
    hid::register_device(device)
        .await
        .expect("HID device already registered");

    // Static response buffers: the encoded HID descriptor, the report descriptor,
    // and an always-zero input report (doubles as the reset sentinel).
    define_static_buffer!(hid_desc_buf, u8, [0u8; hid::DESCRIPTOR_LEN]);
    define_static_buffer!(report_desc_buf, u8, [0u8; REPORT_DESCRIPTOR.len()]);
    define_static_buffer!(input_buf, u8, [0u8; INPUT_MAX_LEN]);

    {
        let mut owned = hid_desc_buf::get_mut()
            .expect("HID descriptor buffer must not already be borrowed")
            .borrow_mut()
            .expect("HID descriptor buffer borrow");
        let buf: &mut [u8] = owned.borrow_mut();
        descriptor(&regs)
            .encode_into_slice(buf)
            .expect("HID descriptor fits its buffer");
    }
    {
        let mut owned = report_desc_buf::get_mut()
            .expect("report descriptor buffer must not already be borrowed")
            .borrow_mut()
            .expect("report descriptor buffer borrow");
        let buf: &mut [u8] = owned.borrow_mut();
        buf.copy_from_slice(REPORT_DESCRIPTOR);
    }

    loop {
        match device.wait_request().await {
            Request::Descriptor => {
                let _ = device
                    .send_response(Some(Response::Descriptor(hid_desc_buf::get())))
                    .await;
            }
            Request::ReportDescriptor => {
                let _ = device
                    .send_response(Some(Response::ReportDescriptor(report_desc_buf::get())))
                    .await;
            }
            Request::InputReport => {
                // Hand back the empty/reset sentinel and deassert the interrupt
                let _ = device
                    .send_response(Some(Response::InputReport(input_buf::get())))
                    .await;
                hid_int.set_high();
            }
            Request::Command(cmd) => match cmd {
                hid::Command::Reset => {
                    // Ack the reset, then assert the interrupt so the host reads the reset sentinel
                    let _ = device.send_response(None).await;
                    hid_int.set_low();
                }
                hid::Command::GetReport { .. } => {
                    let _ = device
                        .send_response(Some(Response::InputReport(input_buf::get())))
                        .await;
                }
                // Other commands: nothing to do, just ack
                _ => {
                    let _ = device.send_response(None).await;
                }
            },
            Request::OutputReport(..) => {
                let _ = device.send_response(None).await;
            }
        }
    }
}
