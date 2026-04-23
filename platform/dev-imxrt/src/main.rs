#![no_std]
#![no_main]

mod board;
mod imxrt;

use board::Board;
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_imxrt::uart;
use panic_probe as _;
use platform_common::board::BoardIo;
use platform_common::mock::MockOdpRelayHandler;
use static_cell::StaticCell;

#[embassy_executor::task]
async fn uart_service(uart: uart::Uart<'static, uart::Async>, relay: MockOdpRelayHandler) {
    info!("Starting uart service");
    static UART_SERVICE: StaticCell<uart_service::Service<MockOdpRelayHandler>> = StaticCell::new();
    let uart_service = uart_service::Service::new(relay).unwrap();
    let uart_service = UART_SERVICE.init(uart_service);

    let Err(e) = uart_service::task::uart_service(uart_service, uart).await;
    panic!("uart-service error: {:?}", e);
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());
    let board = Board::init(p);

    let relay = platform_common::mock::init(spawner).await;
    spawner.spawn(uart_service(board.uart, relay).expect("Failed to spawn UART service task"));
}
