#![no_main]
#![no_std]

mod board;
mod hid;

use board::Board;
use defmt::info;
use defmt_semihosting as _;
use embassy_executor::Spawner;
use embassy_qemu_riscv::uart::{buffered, Async};
use platform_common::board::BoardIo;
use platform_common::mock::MockOdpRelayHandler;
use semihosting as _; // Panic handler
use static_cell::StaticCell;

#[embassy_executor::task]
async fn uart_service(uart: buffered::Uart<'static, Async>, relay: MockOdpRelayHandler) {
    info!("Starting uart service");
    static UART_SERVICE: StaticCell<uart_service::MctpSerialService<MockOdpRelayHandler>> = StaticCell::new();
    let uart_service =
        uart_service::MctpSerialService::default_mctp_serial(relay).expect("failed to init MctpSerial uart-service");
    let uart_service = UART_SERVICE.init(uart_service);
    let Err(e) = uart_service::task::uart_service(uart_service, uart).await;
    panic!("uart-service error: {:?}", e);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_qemu_riscv::init();
    let board = Board::init(p);

    #[cfg(not(feature = "time-alarm-wake"))]
    let relay = platform_common::mock::init(spawner).await;
    #[cfg(feature = "time-alarm-wake")]
    let relay = {
        use time_alarm_service_interface::{AcpiTimerId, AlarmTimerSeconds, TimeAlarmService};
        #[cfg(not(feature = "time-alarm-power-input"))]
        let source = match option_env!("ODP_WAKE_SOURCE") {
            Some("ac") => AcpiTimerId::AcPower,
            Some("dc") => AcpiTimerId::DcPower,
            _ => panic!("Build the wake fixture with ODP_WAKE_SOURCE=ac or dc"),
        };
        let (relay, service) = platform_common::mock::init_with_time_alarm(spawner, |service| {
            #[cfg(feature = "time-alarm-power-input")]
            let source = {
                assert!(
                    board.power_input.is_initialized(),
                    "TimeAlarm GPIO2 power input must be explicitly initialized"
                );
                power_source(&board.power_input)
            };
            info!("TimeAlarm wake source: {:?}", source);
            for timer in [AcpiTimerId::AcPower, AcpiTimerId::DcPower] {
                service
                    .set_timer_value(timer, AlarmTimerSeconds::DISABLED)
                    .expect("Failed to disable initial fixture alarm");
            }
            service.set_power_source(source);
        })
        .await;
        spawner.spawn(time_alarm_wake(service, board.wake_gpio).expect("Failed to spawn TimeAlarm wake task"));
        #[cfg(feature = "time-alarm-power-input")]
        spawner.spawn(
            time_alarm_power_input(service, board.power_input).expect("Failed to spawn TimeAlarm power input task"),
        );
        relay
    };
    spawner.spawn(uart_service(board.uart, relay).expect("Failed to spawn UART service task"));

    // Bring up a minimal HID-over-I2C device so a host (e.g. Windows) can
    // complete its initial HID handshake against the EC
    hid::init(spawner, board.i2c, board.gpio).await;
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

#[cfg(feature = "time-alarm-power-input")]
fn power_source(
    input: &embassy_qemu_riscv::gpio::Input<'_, embassy_qemu_riscv::gpio::Async>,
) -> time_alarm_service_interface::AcpiTimerId {
    use time_alarm_service_interface::AcpiTimerId;
    if input.is_high() {
        AcpiTimerId::AcPower
    } else {
        AcpiTimerId::DcPower
    }
}

#[cfg(feature = "time-alarm-power-input")]
#[embassy_executor::task]
async fn time_alarm_power_input(
    service: platform_common::mock::time_alarm::TimeAlarmService,
    mut input: embassy_qemu_riscv::gpio::Input<'static, embassy_qemu_riscv::gpio::Async>,
) {
    use time_alarm_service_interface::AcpiTimerId;
    loop {
        let source = power_source(&input);
        service.set_power_source(source);
        info!("TimeAlarm power input: {:?}", source);
        match source {
            AcpiTimerId::AcPower => input.wait_for_low().await,
            AcpiTimerId::DcPower => input.wait_for_high().await,
        }
    }
}
