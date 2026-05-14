use embassy_mcxa::{bind_interrupts, clocks::periph_helpers::LpuartClockSel, lpuart};
use platform_common::board::BoardIo;
use static_cell::ConstStaticCell;

bind_interrupts!(struct Irqs {
    LPUART2 => lpuart::BbqInterruptHandler::<embassy_mcxa::peripherals::LPUART2>;
});

const SIZE: usize = 4096;
static RX_BUF: ConstStaticCell<[u8; SIZE]> = ConstStaticCell::new([0u8; SIZE]);
static TX_BUF: ConstStaticCell<[u8; SIZE]> = ConstStaticCell::new([0u8; SIZE]);

/// Board IO for the dev-mcxa platform.
///
/// This minimal development board provides a UART interface
/// for ODP service communication.
pub struct Board {
    /// UART for ODP service communication.
    pub uart: lpuart::LpuartBbq,
}

impl BoardIo for Board {
    type Peripherals = embassy_mcxa::Peripherals;

    fn init(p: Self::Peripherals) -> Self {
        let mut config = lpuart::BbqConfig::default();
        config.power = embassy_mcxa::clocks::PoweredClock::NormalEnabledDeepSleepDisabled;
        config.source = LpuartClockSel::FroHfDiv;

        let tx_buf = TX_BUF.take();
        let rx_buf = RX_BUF.take();

        // Create UART instance with DMA channels
        let tx_dma = embassy_mcxa::dma::DmaChannel::new(p.DMA0_CH0);
        let rx_dma = embassy_mcxa::dma::DmaChannel::new(p.DMA0_CH1);

        let parts = lpuart::BbqParts::new(p.LPUART2, Irqs, p.P2_2, tx_buf, tx_dma, p.P2_3, rx_buf, rx_dma)
            .expect("failed to create BbqParts");

        let lpuart = lpuart::LpuartBbq::new(parts, config, lpuart::BbqRxMode::Efficiency)
            .expect("failed to initialize async LPUART");

        Board { uart: lpuart }
    }
}
