use embedded_sensors_hal_async::sensor as sensor_traits;
use embedded_sensors_hal_async::temperature::{DegreesCelsius, TemperatureSensor, TemperatureThresholdSet};
use thermal_service::sensor;

#[derive(Copy, Clone, Debug)]
pub(crate) struct PtsError;
impl sensor_traits::Error for PtsError {
    fn kind(&self) -> sensor_traits::ErrorKind {
        sensor_traits::ErrorKind::Other
    }
}

pub(crate) struct Pts<T: TemperatureSensor + TemperatureThresholdSet> {
    driver: T,
}

impl<T: TemperatureSensor + TemperatureThresholdSet> Pts<T> {
    pub(crate) fn new(driver: T) -> Self {
        Self { driver }
    }
}

impl<T: TemperatureSensor + TemperatureThresholdSet> sensor_traits::ErrorType for Pts<T> {
    type Error = PtsError;
}

impl<T: TemperatureSensor + TemperatureThresholdSet> TemperatureSensor for Pts<T> {
    async fn temperature(&mut self) -> Result<DegreesCelsius, Self::Error> {
        self.driver.temperature().await.map_err(|_| PtsError)
    }
}

impl<T: TemperatureSensor + TemperatureThresholdSet> TemperatureThresholdSet for Pts<T> {
    async fn set_temperature_threshold_low(&mut self, threshold: DegreesCelsius) -> Result<(), Self::Error> {
        self.driver
            .set_temperature_threshold_low(threshold)
            .await
            .map_err(|_| PtsError)
    }

    async fn set_temperature_threshold_high(&mut self, threshold: DegreesCelsius) -> Result<(), Self::Error> {
        self.driver
            .set_temperature_threshold_high(threshold)
            .await
            .map_err(|_| PtsError)
    }
}

impl<T: TemperatureSensor + TemperatureThresholdSet> sensor::CustomRequestHandler for Pts<T> {}
impl<T: TemperatureSensor + TemperatureThresholdSet> sensor::Controller for Pts<T> {}
