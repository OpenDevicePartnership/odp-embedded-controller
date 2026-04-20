#![no_std]
pub mod board;
#[cfg(feature = "mock")]
pub mod mock;

// Shared relay handler for ODP reference platforms
embedded_services::relay::mctp::impl_odp_mctp_relay_handler!(
    OdpRelayHandler;
    Battery, 0x08, battery_service::Service<'static, 1>;
    Thermal, 0x09, thermal_service::Service<'static>;
    TimeAlarm, 0x0B, time_alarm_service::Service<'static>;
);
