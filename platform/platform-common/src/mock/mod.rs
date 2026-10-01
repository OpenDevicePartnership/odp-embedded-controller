//! Provides mock hardware for development platforms lacking hardware.
//! Additionally, provides common setup and initialization if the platform doesn't need anything special.
//!
//! This allows for easy testing of host to EC comms.
pub mod battery;
pub mod thermal;
pub mod time_alarm;

crate::impl_relay_handler!(
    MockOdpRelayHandler,
    crate::mock::battery::BatteryService,
    crate::mock::thermal::ThermalService
);

/// Initialize mock embedded services.
pub async fn init(spawner: embassy_executor::Spawner) -> MockOdpRelayHandler {
    init_with_time_alarm(spawner, |_| {}).await.0
}

/// Configure TimeAlarm before its runner starts and retain its board control handle.
pub async fn init_with_time_alarm(
    spawner: embassy_executor::Spawner,
    configure: impl FnOnce(time_alarm::TimeAlarmService),
) -> (MockOdpRelayHandler, time_alarm::TimeAlarmService) {
    embedded_services::info!("Initializing mock services...");
    embedded_services::init().await;

    let thermal = thermal::init(spawner).await;
    let battery = battery::init(spawner).await;
    let tas = time_alarm::init_with(spawner, configure).await;

    let relay = MockOdpRelayHandler::new(
        battery_service_relay::BatteryServiceRelayHandler::new(battery),
        thermal_service_relay::ThermalServiceRelayHandler::new(thermal),
        time_alarm_service_relay::TimeAlarmServiceRelayHandler::new(tas),
    );
    (relay, tas)
}
