use core::sync::atomic;
use embassy_imxrt::gpio;
use embassy_sync::signal::Signal;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, once_lock::OnceLock};
use embassy_time::{with_timeout, Duration};

static FAN_TACH: OnceLock<Tachometer> = OnceLock::new();

pub(crate) struct Tachometer {
    rpm: atomic::AtomicU16,
    en_signal: Signal<ThreadModeRawMutex, ()>,
    enabled: atomic::AtomicBool,
}

impl Tachometer {
    pub(crate) fn new() -> Self {
        Self {
            rpm: atomic::AtomicU16::new(0),
            en_signal: Signal::new(),
            enabled: atomic::AtomicBool::new(false),
        }
    }

    pub(crate) fn rpm(&self) -> &atomic::AtomicU16 {
        &self.rpm
    }

    pub(crate) fn enable(&self) {
        self.enabled.store(true, atomic::Ordering::Release);
        self.en_signal.signal(());
    }

    #[allow(dead_code)]
    pub(crate) fn disable(&self) {
        self.enabled.store(false, atomic::Ordering::Release);
    }
}

pub(crate) fn init() {
    FAN_TACH.get_or_init(Tachometer::new);
}

pub(crate) fn fan_tach() -> &'static Tachometer {
    FAN_TACH
        .try_get()
        .expect("tachometer::init() must be called before this")
}

#[embassy_executor::task]
pub(crate) async fn tachometer_task(mut tach_pin: gpio::Input<'static>, tachometer: &'static Tachometer) {
    let window_ms = 1000;
    let scale = 60_000 / window_ms as u16;
    let pulses_per_rev = 2;

    loop {
        if tachometer.enabled.load(atomic::Ordering::Acquire) {
            let mut pulses = 0;
            let _ = with_timeout(Duration::from_millis(window_ms), async {
                loop {
                    tach_pin.wait_for_falling_edge().await.unwrap();
                    pulses += 1;
                }
            })
            .await;

            let measured_rpm = (pulses * scale) / pulses_per_rev;
            tachometer.rpm.store(measured_rpm, atomic::Ordering::Release);
        } else {
            tachometer.en_signal.wait().await;
        }
    }
}
