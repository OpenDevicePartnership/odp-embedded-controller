#![no_std]
pub mod board;
#[cfg(feature = "mock")]
pub mod mock;

/// Defines an ODP MCTP relay handler with the standard service IDs and relay types.
///
/// The handler name, battery service type, and thermal service type vary per
/// platform; the time-alarm service type is shared across all ODP platforms.
///
/// # Example
/// ```ignore
/// platform_common::impl_relay_handler!(
///     OdpRelayHandler,
///     battery_service::Service<'static, 1>,
///     crate::thermal::MyThermalService
/// );
/// ```
#[macro_export]
macro_rules! impl_relay_handler {
    ($handler_name:ident, $battery_service_ty:ty, $thermal_service_ty:ty) => {
        embedded_services::relay::mctp::impl_odp_mctp_relay_handler!(
            $handler_name;
            Battery, 0x08,
                battery_service_relay::BatteryServiceRelayHandler<$battery_service_ty>;
            Thermal, 0x09,
                thermal_service_relay::ThermalServiceRelayHandler<$thermal_service_ty>;
            TimeAlarm, 0x0B,
                time_alarm_service_relay::TimeAlarmServiceRelayHandler<time_alarm_service::Service<'static>>;
        );
    };
}
