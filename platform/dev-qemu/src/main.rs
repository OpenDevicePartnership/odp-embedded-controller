#![no_main]
#![no_std]

mod board;
mod hid;
mod mctp;

use board::Board;
use defmt_semihosting as _;
use embassy_executor::Spawner;
#[cfg(feature = "time-alarm-wake")]
use embedded_services::info;
use platform_common::board::BoardIo;
use semihosting as _; // Panic handler

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_qemu_riscv::init();
    let board = Board::init(p);
    #[cfg(not(feature = "time-alarm-wake"))]
    let (relay, time_alarm) = platform_common::mock::init_with_time_alarm(spawner).await;
    #[cfg(feature = "time-alarm-wake")]
    let (relay, time_alarm) = {
        use time_alarm_service_interface::{AcpiTimerId, AlarmTimerSeconds, TimeAlarmService};
        let source = match option_env!("ODP_WAKE_SOURCE") {
            Some("ac") => AcpiTimerId::AcPower,
            Some("dc") => AcpiTimerId::DcPower,
            _ => panic!("Build the wake fixture with ODP_WAKE_SOURCE=ac or dc"),
        };
        info!("TimeAlarm wake source: {:?}", source);
        let (relay, service) = platform_common::mock::init_with_time_alarm_configure(spawner, |service| {
            for timer in [AcpiTimerId::AcPower, AcpiTimerId::DcPower] {
                service
                    .set_timer_value(timer, AlarmTimerSeconds::DISABLED)
                    .expect("Failed to disable initial fixture alarm");
            }
            service.set_power_source(source);
        })
        .await;
        spawner.spawn(time_alarm_wake(service, board.wake_gpio).expect("Failed to spawn TimeAlarm wake task"));
        (relay, service)
    };

    // This allows thermal and battery to continue receiving requests over uart/mctp, but also allows
    // time-alarm to simultaneously receive requests over uart/mctp or hidi2c
    spawner.spawn(mctp::uart_service(board.uart, relay).expect("Failed to spawn UART service task"));
    hid::init(spawner, board.i2c, board.gpio, time_alarm).await;
}

#[cfg(feature = "time-alarm-wake")]
#[embassy_executor::task]
async fn time_alarm_wake(
    service: platform_common::mock::time_alarm::TimeAlarmService,
    mut gpio: embassy_qemu_riscv::gpio::Output<'static>,
) {
    loop {
        let requested = service.wait_for_wake_signal().await;
        if requested {
            gpio.set_high();
        } else {
            gpio.set_low();
        }
        info!("TimeAlarm wake requested: {}", requested);
    }
}
