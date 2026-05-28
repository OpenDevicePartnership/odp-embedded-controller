#![no_std]
#![no_main]

mod board;

use board::Board;
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_microchip::uart;
use panic_probe as _;
use platform_common::board::BoardIo;
use platform_common::mock::MockOdpRelayHandler;
use static_cell::StaticCell;

#[embassy_executor::task]
async fn uart_service(uart: uart::Uart<'static, uart::Async>, relay: MockOdpRelayHandler) {
    info!("Starting uart service");
    static UART_SERVICE: StaticCell<uart_service::DefaultService<MockOdpRelayHandler>> = StaticCell::new();
    let uart_service = uart_service::DefaultService::default_smbusespi(relay).unwrap();
    let uart_service = UART_SERVICE.init(uart_service);

    let Err(e) = uart_service::task::uart_service(uart_service, uart).await;
    panic!("uart-service error: {:?}", e);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Booting...");
    let p = embassy_microchip::init(embassy_microchip::config::Config::default());
    let board = Board::init(p);

    info!("Hello world from MEC1723!");

    let relay = platform_common::mock::init(spawner).await;
    spawner.spawn(uart_service(board.uart, relay).expect("Failed to spawn UART service task"));
}