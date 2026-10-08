use defmt::info;
use embassy_qemu_riscv::uart::{buffered, Async};
use platform_common::mock::MockOdpRelayHandler;
use static_cell::StaticCell;

#[embassy_executor::task]
pub async fn uart_service(uart: buffered::Uart<'static, Async>, relay: MockOdpRelayHandler) {
    info!("Starting uart service");
    static UART_SERVICE: StaticCell<uart_service::MctpSerialService<MockOdpRelayHandler>> = StaticCell::new();
    let uart_service =
        uart_service::MctpSerialService::default_mctp_serial(relay).expect("failed to init MctpSerial uart-service");
    let uart_service = UART_SERVICE.init(uart_service);
    let Err(e) = uart_service::task::uart_service(uart_service, uart).await;
    panic!("uart-service error: {:?}", e);
}
