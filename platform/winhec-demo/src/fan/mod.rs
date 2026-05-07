pub(crate) mod physical_fan;
pub(crate) mod tachometer;

use embassy_executor::Spawner;
use embassy_imxrt::gpio;
use embassy_imxrt::peripherals::{CTIMER4_COUNT_CHANNEL0, CTIMER4_COUNT_CHANNEL2, PIO0_28, PIO0_30};
use embassy_imxrt::pwm::MicroSeconds;
use embassy_imxrt::timer::CTimerPwmPeriodChannel;
use embassy_imxrt::Peri;
use physical_fan::PhysicalFan;
use static_cell::StaticCell;
use thermal_service as ts;

use crate::thermal::TMP108_ID;

const SAMPLE_BUF_LEN: usize = 16;

pub(crate) const FAN_ID: ts::fan::DeviceId = ts::fan::DeviceId(0);

#[derive(Copy, Clone, Debug)]
pub(crate) struct FanError;

pub(crate) struct FanConfig {
    pub(crate) pwm_pin: Peri<'static, PIO0_30>,
    pub(crate) pwm_match_channel: Peri<'static, CTIMER4_COUNT_CHANNEL2>,
    pub(crate) pwm_period_channel: Peri<'static, CTIMER4_COUNT_CHANNEL0>,
    pub(crate) tach_pin: Peri<'static, PIO0_28>,
}

pub(crate) struct FanHandle {
    pub fan: &'static ts::fan::Fan<PhysicalFan, SAMPLE_BUF_LEN>,
}

pub(crate) async fn init(spawner: Spawner, config: FanConfig) -> Result<FanHandle, FanError> {
    tachometer::init();

    static FAN_PWM_TIMER: StaticCell<CTimerPwmPeriodChannel<'static>> = StaticCell::new();
    let pwm_timer = FAN_PWM_TIMER
        .init(CTimerPwmPeriodChannel::new(config.pwm_period_channel, MicroSeconds(40)).map_err(|_| FanError)?);

    let driver = PhysicalFan::new(
        config.pwm_pin,
        pwm_timer,
        config.pwm_match_channel,
        tachometer::fan_tach().rpm(),
    )?;

    let profile = ts::fan::Profile {
        sensor_id: TMP108_ID,
        auto_control: true,
        on_temp: 25.0,
        ramp_temp: 30.0,
        max_temp: 35.0,
        hysteresis: 0.0,
        ..Default::default()
    };

    static FAN: StaticCell<ts::fan::Fan<PhysicalFan, SAMPLE_BUF_LEN>> = StaticCell::new();
    let fan = FAN.init(ts::fan::Fan::new(FAN_ID, driver, profile));

    let tach_pin = gpio::Input::new(config.tach_pin, gpio::Pull::None, gpio::Inverter::Disabled);
    spawner.must_spawn(tachometer::tachometer_task(tach_pin, tachometer::fan_tach()));

    Ok(FanHandle { fan })
}

pub(crate) async fn spawn_service(
    spawner: Spawner,
    handle: &FanHandle,
    thermal_service: &'static ts::Service<'static>,
) {
    type FanService = ts::fan::Service<'static, PhysicalFan, SAMPLE_BUF_LEN>;
    odp_service_common::spawn_service!(
        spawner,
        FanService,
        ts::fan::InitParams {
            fan: handle.fan,
            thermal_service,
        }
    )
    .expect("Failed to spawn fan service");
}
