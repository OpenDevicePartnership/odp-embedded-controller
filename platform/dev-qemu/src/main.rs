#![no_main]
#![no_std]

mod board;
mod hid;
mod mctp;

use board::Board;
use defmt_semihosting as _;
use embassy_executor::Spawner;
use platform_common::{board::BoardIo, mock::time_alarm as mock_time_alarm};
use semihosting as _; // Panic handler

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_qemu_riscv::init();
    let board = Board::init(p);
    let (relay, time_alarm) = init_time_alarm(
        spawner,
        #[cfg(feature = "time-alarm-wake")]
        board.wake_gpio,
        #[cfg(feature = "time-alarm-power-input")]
        board.power_input,
    )
    .await;

    spawner.spawn(mctp::uart_service(board.uart, relay).expect("Failed to spawn UART service task"));
    hid::init(spawner, board.i2c, board.gpio, time_alarm).await;
}

async fn init_time_alarm(
    spawner: Spawner,
    #[cfg(feature = "time-alarm-wake")] wake_gpio: embassy_qemu_riscv::gpio::Output<'static>,
    #[cfg(feature = "time-alarm-power-input")] mut power_input: embassy_qemu_riscv::gpio::Input<
        'static,
        embassy_qemu_riscv::gpio::Async,
    >,
) -> (
    platform_common::mock::MockOdpRelayHandler,
    mock_time_alarm::TimeAlarmService,
) {
    #[cfg(not(feature = "time-alarm-wake"))]
    let (relay, time_alarm) = platform_common::mock::init_with_time_alarm(spawner).await;
    #[cfg(feature = "time-alarm-wake")]
    let (relay, time_alarm) = {
        #[cfg(not(feature = "time-alarm-power-input"))]
        use time_alarm_service_interface::AcpiTimerId;
        #[cfg(not(feature = "time-alarm-power-input"))]
        let source = match option_env!("ODP_WAKE_SOURCE") {
            Some("ac") => AcpiTimerId::AcPower,
            Some("dc") => AcpiTimerId::DcPower,
            _ => panic!("Build the wake fixture with ODP_WAKE_SOURCE=ac or dc"),
        };
        let (relay, service) = platform_common::mock::init_with_time_alarm_configure(spawner, |service| {
            #[cfg(feature = "time-alarm-power-input")]
            let source = {
                assert!(
                    power_input.is_initialized(),
                    "TimeAlarm GPIO2 power input must be explicitly initialized"
                );
                mock_time_alarm::power_source(&mut power_input).expect("Failed to read initial TimeAlarm power input")
            };
            mock_time_alarm::configure_wake(service, source);
        })
        .await;
        spawner.spawn(time_alarm_wake(service, wake_gpio).expect("Failed to spawn TimeAlarm wake task"));
        #[cfg(feature = "time-alarm-power-input")]
        spawner
            .spawn(time_alarm_power_input(service, power_input).expect("Failed to spawn TimeAlarm power input task"));
        (relay, service)
    };

    (relay, time_alarm)
}

#[cfg(feature = "time-alarm-wake")]
#[embassy_executor::task]
async fn time_alarm_wake(service: mock_time_alarm::TimeAlarmService, gpio: embassy_qemu_riscv::gpio::Output<'static>) {
    mock_time_alarm::drive_wake_output(service, gpio)
        .await
        .expect("TimeAlarm wake output failed");
}

#[cfg(feature = "time-alarm-power-input")]
#[embassy_executor::task]
async fn time_alarm_power_input(
    service: mock_time_alarm::TimeAlarmService,
    input: embassy_qemu_riscv::gpio::Input<'static, embassy_qemu_riscv::gpio::Async>,
) {
    mock_time_alarm::monitor_power_input(service, input)
        .await
        .expect("TimeAlarm power input failed");
}
