use platform_common::board::BoardIo;
use embassy_microchip::{uart, bind_interrupts, peripherals, Peripherals};
use static_cell::ConstStaticCell;

bind_interrupts!(struct Irqs {
    UART1 => uart::InterruptHandler::<peripherals::UART1>;
});

static UART_BUFFER: ConstStaticCell<[u8; 1024]> = ConstStaticCell::new([0u8; 1024]);

/// Board IO for the dev-mec platform.
pub struct Board {
    pub uart: uart::Uart<'static, uart::Async>,
}

impl BoardIo for Board {
    type Peripherals = Peripherals;

    fn init(p: Self::Peripherals) -> Self {
        Board {
            /* Set up async UART */
            uart: uart::Uart::new_async(
                p.UART1,
                p.GPIO171,
                p.GPIO170,
                Irqs,
                UART_BUFFER.take(),
                uart::Config::default()
            ).expect("Failed to create 'uart' in 'Board'.")
         }
    }
}
