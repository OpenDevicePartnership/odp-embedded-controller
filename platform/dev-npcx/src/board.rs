use embassy_npcx::{bind_interrupts, peripherals, uart};
use platform_common::board::BoardIo;

bind_interrupts!(pub struct Irqs {
    CR_UART1_MDMA1 => uart::InterruptHandler<peripherals::CR_UART1>;
});

/// Board IO for the dev-npcx platform.
///
/// This minimal development board provides a UART interface
/// for ODP service communication.
pub struct Board {
    /// UART for ODP service communication.
    pub uart: uart::Uart<'static, peripherals::CR_UART1>,
}

impl BoardIo for Board {
    type Peripherals = embassy_npcx::Peripherals;

    fn init(p: Self::Peripherals) -> Self {
        let mut config = uart::Config::default();
        config.baudrate = 115200;

        let uart = uart::Uart::new(p.CR_UART1, p.PG04, p.PH04, Irqs, config);
        Board { uart }
    }
}
