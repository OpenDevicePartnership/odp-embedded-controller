//! HID-over-I2C access to the shared time-alarm service.

use embassy_executor::Spawner;
use embassy_qemu_riscv::gpio::Output;
use embassy_qemu_riscv::i2c::target::{Async, I2c};
use embedded_services::info;
use hidi2c_target_service::{HardwareVersionInfo, ProductId, TimeoutSettings, VendorId, VersionId};
use time_alarm_service_relay::hid::TimeAlarmHidRelay;

const HID_VID: u16 = 0x045E; // MSFT
const HID_PID: u16 = 0x0002;
const HID_VERSION: u16 = 0x0100;

type HidI2cService = hidi2c_target_service::Service<
    'static,
    I2c<'static, Async>,
    Output<'static>,
    TimeAlarmHidRelay<'static, time_alarm_service::Service<'static>>,
>;

pub async fn init(
    spawner: Spawner,
    i2c: I2c<'static, Async>,
    hid_int: Output<'static>,
    time_alarm: time_alarm_service::Service<'static>,
) {
    info!("Starting HID-I2C time-alarm service");
    let device = TimeAlarmHidRelay::new(time_alarm);
    let hwinfo = HardwareVersionInfo {
        vendor_id: VendorId::new(HID_VID).expect("HID vendor ID must be nonzero"),
        product_id: ProductId(HID_PID),
        version_id: VersionId(HID_VERSION),
    };

    odp_service_common::spawn_service!(spawner, HidI2cService, |resources| hidi2c_target_service::Service::new(
        resources,
        i2c,
        hid_int,
        device,
        hwinfo,
        TimeoutSettings::default(),
    ))
    .expect("Failed to initialize HID-I2C time-alarm service");
}
