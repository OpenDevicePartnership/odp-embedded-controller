//! Minimal HID-over-I2C startup fixture, with no unsolicited input reports.
//! The target service owns descriptor framing, reset acknowledgment, and GPIO0.

use embassy_executor::Spawner;
use embassy_qemu_riscv::gpio::Output;
use embassy_qemu_riscv::i2c::target::{Async, I2c};
use embedded_services::relay::hid::{
    GetHidReport, GetHidReportType, HidDevice, HidDevicePowerState, HidError, HidReport, HidReportDescriptor, ReportId,
    SetHidReport,
};
use hidi2c_target_service::{HardwareVersionInfo, ProductId, TimeoutSettings, VendorId, VersionId};

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

struct MockHidDevice {
    descriptor: HidReportDescriptor<'static>,
}

impl HidDevice for MockHidDevice {
    type InputReportMaxSize = typenum::U1;
    type OutputReportMaxSize = typenum::U0;
    type FeatureReportMaxSize = typenum::U0;

    const MAX_REPORT_COUNT: u8 = 1;
    const MAX_DESCRIPTOR_LEN: usize = REPORT_DESCRIPTOR.len();

    fn report_descriptor(&self) -> &HidReportDescriptor<'_> {
        &self.descriptor
    }

    async fn process_get_report<R>(
        &mut self,
        report_type: GetHidReportType,
        report_id: ReportId,
        process_report: impl AsyncFnOnce(GetHidReport<'_>) -> R,
    ) -> Result<R, HidError> {
        if !matches!(report_type, GetHidReportType::Input) || report_id != ReportId(0) {
            defmt::warn!("Unsupported mock HID GET_REPORT");
            return Err(HidError::TriggerReset);
        }
        Ok(process_report(GetHidReport::Input(HidReport::new(report_id, &[0]))).await)
    }

    async fn set_report(&mut self, _report: &SetHidReport<'_>) -> Result<(), HidError> {
        defmt::trace!("Ignoring mock HID SET_REPORT");
        Ok(())
    }

    async fn wait_for_input_report(&mut self) {
        core::future::pending().await
    }

    fn has_pending_input_report(&mut self) -> bool {
        false
    }

    async fn process_next_input_report<R>(
        &mut self,
        _process_report: impl AsyncFnOnce(HidReport<'_>) -> R,
    ) -> Result<R, HidError> {
        core::future::pending().await
    }

    async fn set_power_state(&mut self, _state: HidDevicePowerState) -> Result<(), HidError> {
        Ok(())
    }

    async fn reset(&mut self) {}
}

pub async fn init(spawner: Spawner, i2c: I2c<'static, Async>, hid_int: Output<'static>) {
    odp_service_common::spawn_service!(
        spawner,
        hidi2c_target_service::Service<'static, I2c<'static, Async>, Output<'static>, MockHidDevice>,
        |resources| hidi2c_target_service::Service::new(
            resources,
            i2c,
            hid_int,
            MockHidDevice {
                descriptor: HidReportDescriptor::new(REPORT_DESCRIPTOR).expect("Invalid mock HID report descriptor"),
            },
            HardwareVersionInfo {
                vendor_id: VendorId::new(HID_VID).expect("HID vendor ID must be nonzero"),
                product_id: ProductId(HID_PID),
                version_id: VersionId(HID_VERSION),
            },
            TimeoutSettings::default(),
        )
    )
    .expect("Failed to initialize HID target service");
}
