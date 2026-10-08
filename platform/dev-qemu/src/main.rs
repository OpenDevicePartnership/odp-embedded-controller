#![no_main]
#![no_std]

mod board;
mod hid;
mod mctp;

use board::Board;
use defmt_semihosting as _;
use embassy_executor::Spawner;
use platform_common::board::BoardIo;
use semihosting as _; // Panic handler

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_qemu_riscv::init();
    let board = Board::init(p);
    let (relay, time_alarm) = platform_common::mock::init_with_time_alarm(spawner).await;

    // This allows thermal and battery to continue receiving requests over uart/mctp, but also allows
    // time-alarm to simultaneously receive requests over uart/mctp or hidi2c
    spawner.spawn(mctp::uart_service(board.uart, relay).expect("Failed to spawn UART service task"));
    hid::init(spawner, board.i2c, board.gpio, time_alarm).await;
}
