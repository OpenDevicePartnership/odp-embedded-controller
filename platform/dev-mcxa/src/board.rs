use embassy_mcxa::lpuart;
use platform_common::board::BoardIo;

/// Board IO for the dev-mcxa platform.
///
/// This minimal development board provides a UART interface
/// for ODP service communication.
pub struct Board {
    /// UART for ODP service communication.
    pub uart: lpuart::Lpuart<'static, lpuart::Dma<'static>>,
}

impl BoardIo for Board {
    type Peripherals = embassy_mcxa::Peripherals;

    fn init(p: Self::Peripherals) -> Self {
        let config = lpuart::Config::default();

        let uart = lpuart::Lpuart::new_async_with_dma(
            p.LPUART3, p.P4_5, // TX pin
            p.P4_2, // RX pin
            p.DMA0_CH0, p.DMA0_CH1, config,
        )
        .expect("failed to initialize async LPUART");

        Board { uart }
    }
}
