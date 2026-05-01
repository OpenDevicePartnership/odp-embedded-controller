pub(crate) mod pts;

use crate::fan;
use defmt::info;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_imxrt::i2c;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::once_lock::OnceLock;
use pts::Pts;
use static_cell::StaticCell;
use thermal_service as ts;
use tmp108::Tmp108;

const SAMPLE_BUF_LEN: usize = 16;
type Tmp108Type = Tmp108<I2cDevice<'static, ThreadModeRawMutex, i2c::master::I2cMaster<'static, i2c::Async>>>;
type Tmp108SensorType = ts::sensor::Sensor<Pts<Tmp108Type>, SAMPLE_BUF_LEN>;

pub(crate) const TMP108_ID: ts::sensor::DeviceId = ts::sensor::DeviceId(0);

pub async fn init(
    spawner: embassy_executor::Spawner,
    sensor_i2c: &'static Mutex<ThreadModeRawMutex, i2c::master::I2cMaster<'static, i2c::Async>>,
    fan_config: fan::FanConfig,
) -> &'static ts::Service<'static> {
    info!("Initializing thermal service...");

    let bus = I2cDevice::new(sensor_i2c);
    let driver = Tmp108::new_with_a0_gnd(bus);

    let pts = Pts::new(driver);
    static SENSOR: StaticCell<Tmp108SensorType> = StaticCell::new();
    let sensor = SENSOR.init(ts::sensor::Sensor::new(TMP108_ID, pts, ts::sensor::Profile::default()));
    let fan_handle = fan::init(spawner, fan_config).await.expect("Failed to initialize fan");

    static SENSORS: StaticCell<[&'static ts::sensor::Device; 1]> = StaticCell::new();
    let sensors = SENSORS.init([sensor.device()]);

    static FANS: StaticCell<[&'static ts::fan::Device; 1]> = StaticCell::new();
    let fans = FANS.init([fan_handle.fan.device()]);

    static STORAGE: OnceLock<ts::Service<'static>> = OnceLock::new();
    let service = ts::Service::init(&STORAGE, sensors, fans).await;

    type Tmp108Service = ts::sensor::Service<'static, Pts<Tmp108Type>, SAMPLE_BUF_LEN>;
    odp_service_common::spawn_service!(
        spawner,
        Tmp108Service,
        ts::sensor::InitParams {
            sensor,
            thermal_service: service,
        }
    )
    .expect("Failed to spawn TMP108 sensor service");

    fan::spawn_service(spawner, &fan_handle, service).await;

    fan::tachometer::fan_tach().enable();
    service
        .execute_fan_request(fan::FAN_ID, ts::fan::Request::EnableAutoControl)
        .await
        .expect("Failed to enable fan auto control");

    info!("Thermal service initialized");
    service
}

#[embassy_executor::task]
pub async fn monitor(thermal_service: &'static ts::Service<'static>) -> ! {
    use embassy_time::Timer;
    use embedded_services::error;

    loop {
        match thermal_service
            .execute_sensor_request(TMP108_ID, ts::sensor::Request::GetTemp)
            .await
        {
            Ok(ts::sensor::ResponseData::Temp(temp)) => info!("TMP108 temp: {} C", temp),
            _ => error!("Failed to read TMP108"),
        }
        match thermal_service
            .execute_fan_request(fan::FAN_ID, ts::fan::Request::GetRpm)
            .await
        {
            Ok(ts::fan::ResponseData::Rpm(rpm)) => info!("Fan RPM: {}", rpm),
            _ => error!("Failed to read fan RPM"),
        }
        Timer::after_secs(5).await;
    }
}
