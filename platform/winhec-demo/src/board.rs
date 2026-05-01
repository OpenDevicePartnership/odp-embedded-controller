use crate::fan;
use embassy_imxrt::{bind_interrupts, i2c, peripherals, uart};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use platform_common::board::BoardIo;
use static_cell::StaticCell;

bind_interrupts!(pub struct Irqs {
    FLEXCOMM0 => uart::InterruptHandler<peripherals::FLEXCOMM0>;
    FLEXCOMM2 => i2c::InterruptHandler<peripherals::FLEXCOMM2>;
});

static SENSOR_I2C_BUS: StaticCell<Mutex<ThreadModeRawMutex, i2c::master::I2cMaster<'static, i2c::Async>>> =
    StaticCell::new();

pub struct Board {
    pub uart: uart::Uart<'static, uart::Async>,
    pub sensor_i2c: &'static Mutex<ThreadModeRawMutex, i2c::master::I2cMaster<'static, i2c::Async>>,
    pub fan_config: fan::FanConfig,
}

impl BoardIo for Board {
    type Peripherals = embassy_imxrt::Peripherals;

    fn init(p: Self::Peripherals) -> Self {
        let uart = uart::Uart::new_async(
            p.FLEXCOMM0,
            p.PIO0_1,
            p.PIO0_2,
            Irqs,
            p.DMA0_CH1,
            p.DMA0_CH0,
            Default::default(),
        )
        .expect("Failed to initialize UART");

        let sensor_i2c = SENSOR_I2C_BUS.init(Mutex::new(
            i2c::master::I2cMaster::new_async(
                p.FLEXCOMM2,
                p.PIO0_18,
                p.PIO0_17,
                Irqs,
                i2c::master::Config::default(),
                p.DMA0_CH5,
            )
            .expect("Failed to initialize sensor I2C bus"),
        ));

        let fan_config = fan::FanConfig {
            pwm_pin: p.PIO0_30,
            pwm_match_channel: p.CTIMER4_COUNT_CHANNEL2,
            pwm_period_channel: p.CTIMER4_COUNT_CHANNEL0,
            tach_pin: p.PIO0_28,
        };

        Board {
            uart,
            sensor_i2c,
            fan_config,
        }
    }
}
