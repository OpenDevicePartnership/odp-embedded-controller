use core::sync::atomic;
use embassy_imxrt::pwm::{CentiPercent, Pwm};
use embassy_imxrt::timer::{CTimerMatchOutput, CTimerPwm, CTimerPwmPeriodChannel};
use embassy_imxrt::Peri;
use embedded_fans_async::{Error, ErrorKind, ErrorType, Fan, RpmSense};
use thermal_service as ts;
use ts::fan;

pub(crate) struct PhysicalFan {
    pwm: CTimerPwm<'static>,
    rpm: &'static atomic::AtomicU16,
}

impl PhysicalFan {
    pub(crate) fn new(
        pwm_pin: Peri<'static, impl CTimerMatchOutput>,
        pwm_timer: &'static CTimerPwmPeriodChannel<'static>,
        pwm_match_channel: Peri<'static, impl embassy_imxrt::timer::Instance>,
        rpm: &'static atomic::AtomicU16,
    ) -> Result<Self, super::FanError> {
        let mut pwm: CTimerPwm<'static> =
            CTimerPwm::new(pwm_match_channel, pwm_timer, pwm_pin).map_err(|_| super::FanError)?;
        pwm.enable(());
        pwm.set_duty((), CentiPercent(0, 0));

        Ok(Self { pwm, rpm })
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PhysicalFanError;
impl Error for PhysicalFanError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

impl ErrorType for PhysicalFan {
    type Error = PhysicalFanError;
}

impl Fan for PhysicalFan {
    fn min_rpm(&self) -> u16 {
        0
    }

    fn max_rpm(&self) -> u16 {
        6000
    }

    fn min_start_rpm(&self) -> u16 {
        3000
    }

    async fn set_speed_rpm(&mut self, rpm: u16) -> Result<u16, Self::Error> {
        let duty = ((rpm - self.min_rpm()) as f32 / (self.max_rpm() - self.min_rpm()) as f32) * 100.0;
        let duty = duty.clamp(0.0, 100.0) as u8;
        self.pwm.set_duty((), CentiPercent(duty, 0));
        Ok(rpm)
    }
}

impl RpmSense for PhysicalFan {
    async fn rpm(&mut self) -> Result<u16, Self::Error> {
        Ok(self.rpm.load(atomic::Ordering::Acquire))
    }
}

impl fan::CustomRequestHandler for PhysicalFan {}

impl fan::RampResponseHandler for PhysicalFan {}

impl fan::Controller for PhysicalFan {}
