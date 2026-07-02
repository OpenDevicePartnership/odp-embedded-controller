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

    let relay = platform_common::mock::init(spawner).await;
    spawner.spawn(uart_service(board.uart, relay).expect("Failed to spawn UART service task"));

    // Bring up a minimal HID-over-I2C device so a host (e.g. Windows) can
    // complete its initial HID handshake against the EC
    spawner.spawn(hid::host_task(board.i2c).expect("Failed to spawn HID host task"));
    spawner.spawn(hid::device_task(board.gpio).expect("Failed to spawn HID device task"));
}
