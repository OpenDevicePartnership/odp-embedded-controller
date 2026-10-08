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
    init_with_time_alarm(spawner).await.0
}

/// Initialize mock services and expose the shared time-alarm handle.
///
/// This is primarily used right now so the time-alarm handle can also be used by the hidi2c
/// service on `dev-qemu`.
pub async fn init_with_time_alarm(
    spawner: embassy_executor::Spawner,
) -> (MockOdpRelayHandler, time_alarm_service::Service<'static>) {
    embedded_services::info!("Initializing mock services...");
    embedded_services::init().await;

    let thermal = thermal::init(spawner).await;
    let battery = battery::init(spawner).await;
    let tas = time_alarm::init(spawner).await;

    let relay = MockOdpRelayHandler::new(
        battery_service_relay::BatteryServiceRelayHandler::new(battery),
        thermal_service_relay::ThermalServiceRelayHandler::new(thermal),
        time_alarm_service_relay::mctp::TimeAlarmServiceRelayHandler::new(tas),
    );

    (relay, tas)
}
