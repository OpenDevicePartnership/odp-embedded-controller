use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::digital::Wait;
use embedded_services::info;
use static_cell::StaticCell;
use time_alarm_service as tas;
use time_alarm_service_interface::{AcpiTimerId, AlarmTimerSeconds, TimeAlarmService as _};

pub type TimeAlarmService = tas::Service<'static>;

pub async fn init(spawner: embassy_executor::Spawner) -> TimeAlarmService {
    init_with(spawner, |_| {}).await
}

pub async fn init_with(
    spawner: embassy_executor::Spawner,
    configure: impl FnOnce(TimeAlarmService),
) -> TimeAlarmService {
    info!("Initializing time and alarm service...");

    static TZ_STORAGE: StaticCell<tas::mock::MockNvramStorage<'static>> = StaticCell::new();
    let tz_storage = TZ_STORAGE.init(tas::mock::MockNvramStorage::new(0));

    static AC_EXP_STORAGE: StaticCell<tas::mock::MockNvramStorage<'static>> = StaticCell::new();
    let ac_exp_storage = AC_EXP_STORAGE.init(tas::mock::MockNvramStorage::new(0));

    static AC_POL_STORAGE: StaticCell<tas::mock::MockNvramStorage<'static>> = StaticCell::new();
    let ac_pol_storage = AC_POL_STORAGE.init(tas::mock::MockNvramStorage::new(0));

    static DC_EXP_STORAGE: StaticCell<tas::mock::MockNvramStorage<'static>> = StaticCell::new();
    let dc_exp_storage = DC_EXP_STORAGE.init(tas::mock::MockNvramStorage::new(0));

    static DC_POL_STORAGE: StaticCell<tas::mock::MockNvramStorage<'static>> = StaticCell::new();
    let dc_pol_storage = DC_POL_STORAGE.init(tas::mock::MockNvramStorage::new(0));

    static CLOCK: StaticCell<tas::mock::MockDatetimeClock> = StaticCell::new();
    let clock = CLOCK.init(tas::mock::MockDatetimeClock::new_running());

    let service = odp_service_common::spawn_service!(spawner, TimeAlarmService, |resources| async {
        tas::Service::new(
            resources,
            clock,
            tz_storage,
            ac_exp_storage,
            ac_pol_storage,
            dc_exp_storage,
            dc_pol_storage,
        )
        .await
        .map(|(service, runner)| {
            configure(service);
            (service, runner)
        })
    })
    .expect("Failed to initialize time-alarm service");

    service
}

/// Disable both alarms and select a source before starting a wake fixture's runner.
pub fn configure_wake(service: TimeAlarmService, source: AcpiTimerId) {
    info!("TimeAlarm wake source: {:?}", source);
    for timer in [AcpiTimerId::AcPower, AcpiTimerId::DcPower] {
        service
            .set_timer_value(timer, AlarmTimerSeconds::DISABLED)
            .expect("Failed to disable initial fixture alarm");
    }
    service.set_power_source(source);
}

/// Drive the current wake level and subsequent changes, returning any GPIO error.
pub async fn drive_wake_output<P: OutputPin>(service: TimeAlarmService, mut gpio: P) -> Result<(), P::Error> {
    let mut wake = service
        .wake_requested_receiver()
        .expect("Failed to allocate TimeAlarm wake receiver");
    let mut requested = wake.get().await;
    loop {
        if requested {
            gpio.set_high()?;
        } else {
            gpio.set_low()?;
        }
        info!("TimeAlarm wake requested: {}", requested);
        requested = wake.changed().await;
    }
}

/// Read an active-high AC power input (low indicates DC).
pub fn power_source<P: InputPin>(input: &mut P) -> Result<AcpiTimerId, P::Error> {
    Ok(if input.is_high()? {
        AcpiTimerId::AcPower
    } else {
        AcpiTimerId::DcPower
    })
}

/// Monitor AC/DC changes by resampling after opposite-level waits, returning any GPIO error.
pub async fn monitor_power_input<P: InputPin + Wait>(service: TimeAlarmService, mut input: P) -> Result<(), P::Error> {
    loop {
        let source = power_source(&mut input)?;
        service.set_power_source(source);
        info!("TimeAlarm power input: {:?}", source);
        match source {
            AcpiTimerId::AcPower => input.wait_for_low().await?,
            AcpiTimerId::DcPower => input.wait_for_high().await?,
        }
    }
}
