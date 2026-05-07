#![no_std]
#![no_main]

mod board;
mod fan;
mod imxrt;
mod thermal;

use board::Board;
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_imxrt::uart;
use panic_probe as _;
use platform_common::board::BoardIo;
use platform_common::OdpRelayHandler;
use static_cell::StaticCell;

#[embassy_executor::task]
async fn uart_service(uart: uart::Uart<'static, uart::Async>, relay: OdpRelayHandler<'static>) {
    info!("Starting uart service");
    static UART_SERVICE: StaticCell<uart_service::Service<OdpRelayHandler>> = StaticCell::new();
    let uart_service = uart_service::Service::new(relay).unwrap();
    let uart_service = UART_SERVICE.init(uart_service);

    let Err(e) = uart_service::task::uart_service(uart_service, uart).await;
    panic!("uart-service error: {:?}", e);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());
    let board = Board::init(p);

    embedded_services::init().await;
    info!("WinHEC Demo platform starting up!");

    let thermal = thermal::init(spawner, board.sensor_i2c, board.fan_config).await;
    let battery = platform_common::mock::battery::init(spawner).await;
    let tas = platform_common::mock::time_alarm::init(spawner).await;

    let relay = OdpRelayHandler::new(battery, thermal, tas);
    spawner.must_spawn(uart_service(board.uart, relay));

    spawner.must_spawn(thermal::monitor(thermal));
}
