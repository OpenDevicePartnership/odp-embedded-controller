#![no_std]
#![no_main]

mod board;
mod clocks;

use board::Board;
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa::lpuart;
use panic_probe as _;
use platform_common::board::BoardIo;
use platform_common::mock::MockOdpRelayHandler;
use static_cell::StaticCell;

#[embassy_executor::task]
async fn uart_service(uart: lpuart::LpuartBbq, relay: MockOdpRelayHandler) {
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
    let mut cfg = embassy_mcxa::config::Config::default();
    cfg.clock_cfg = clocks::config();
    let p = embassy_mcxa::init(cfg);
    let board = Board::init(p);

    info!("Hello world from MCXA!");

    let relay = platform_common::mock::init(spawner).await;
    spawner.spawn(uart_service(board.uart, relay).expect("Failed to spawn UART service task"));
}
