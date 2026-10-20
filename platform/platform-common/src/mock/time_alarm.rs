use embedded_services::info;
use static_cell::StaticCell;
use time_alarm_service as tas;

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
