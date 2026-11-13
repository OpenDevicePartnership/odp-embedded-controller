//! Provides mock hardware for development platforms lacking hardware.
//! Additionally, provides common setup and initialization if the platform doesn't need anything special.
//!
//! This allows for easy testing of host to EC comms.
pub mod battery;
pub mod power_policy;
pub mod thermal;
pub mod time_alarm;

crate::impl_relay_handler!(
    MockOdpRelayHandler,
    &'static crate::mock::battery::BatteryService,
    crate::mock::thermal::ThermalService
);

/// Initialize mock embedded services.
pub async fn init(spawner: embassy_executor::Spawner) -> MockOdpRelayHandler {
    init_with_time_alarm(spawner).await.0
}

/// Initialize mock services and expose the shared time-alarm handle.
///
/// This is primarily used right now so the time-alarm handle can also be used by the hidi2c
/// service on `dev-qemu`.
pub async fn init_with_time_alarm(
    spawner: embassy_executor::Spawner,
) -> (MockOdpRelayHandler, time_alarm_service::Service<'static>) {
    let (relay, tas, _psu) = init_with_time_alarm_configure(spawner, |_| {}).await;
    (relay, tas)
}

/// Configure TimeAlarm before its runner starts and retain its board control handle.
///
/// Also returns the control for the mock PSU, whose adapter starts unplugged.
pub async fn init_with_time_alarm_configure(
    spawner: embassy_executor::Spawner,
    configure: impl FnOnce(time_alarm::TimeAlarmService),
) -> (
    MockOdpRelayHandler,
    time_alarm::TimeAlarmService,
    power_policy::MockPsuControl,
) {
    embedded_services::info!("Initializing mock services...");
    embedded_services::init().await;

    let thermal = thermal::init(spawner).await;
    let (battery, battery_power_events) = battery::init(spawner).await;
    let psu = power_policy::init(spawner, battery_power_events);
    let tas = time_alarm::init_with(spawner, configure).await;

    let relay = MockOdpRelayHandler::new(
        battery_service_relay::BatteryServiceRelayHandler::new(battery),
        thermal_service_relay::ThermalServiceRelayHandler::new(thermal),
        time_alarm_service_relay::mctp::TimeAlarmServiceRelayHandler::new(tas),
    );

    (relay, tas, psu)
}
