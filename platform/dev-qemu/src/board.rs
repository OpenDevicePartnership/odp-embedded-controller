use embassy_qemu_riscv::espi::{self, Espi};
use embassy_qemu_riscv::gpio::{Level, Output};
use embassy_qemu_riscv::i2c::target::{self, Async as I2cAsync, I2c};
use embassy_qemu_riscv::uart::{buffered, Async};
use embassy_qemu_riscv::{bind_interrupts, peripherals, uart};
use platform_common::board::BoardIo;
use static_cell::StaticCell;

// 7-bit I2C address the EC will respond to
const I2C_ADDR: u8 = 0x2C;

bind_interrupts!(struct Irqs {
    UART0 => uart::buffered::InterruptHandler<peripherals::UART0>;
    I2C_TARGET => target::InterruptHandler<peripherals::I2C_TARGET>;
    ESPI_TARGET => espi::InterruptHandler<peripherals::ESPI_TARGET>;
});

/// Board IO for the dev-qemu platform.
///
/// This minimal development board provides a UART interface for ODP service
/// communication, an eSPI target for PCC, and I2C/GPIO for HIDI2C.
pub struct Board {
    /// UART for ODP service communication.
    pub uart: buffered::Uart<'static, Async>,
    /// I2C target acting as the HID-over-I2C device endpoint.
    pub i2c: I2c<'static, I2cAsync>,
    /// Interrupt line the HID device drives to signal the host (active low).
    pub gpio: Output<'static>,
    /// eSPI target for the PCC mailbox.
    pub espi: Espi<'static, espi::Async>,
}

impl BoardIo for Board {
    type Peripherals = embassy_qemu_riscv::Peripherals;

    fn init(p: Self::Peripherals) -> Self {
        static RX_BUF: StaticCell<[u8; 256]> = StaticCell::new();
        let rx_buf = RX_BUF.init([0u8; 256]);

        let uart =
            buffered::Uart::new_async(p.UART0, Irqs, rx_buf, Default::default()).expect("Failed to initialize UART");

        let i2c = I2c::new_async(p.I2C_TARGET, Irqs, I2C_ADDR);

        // Start high (since this is an active-low signal)
        let gpio = Output::new(p.GPIO0, Level::High);

        let espi = Espi::new_async(p.ESPI_TARGET, Irqs, espi::Config::default()).expect("Failed to initialize eSPI");

        Board { uart, i2c, gpio, espi }
    }
}
