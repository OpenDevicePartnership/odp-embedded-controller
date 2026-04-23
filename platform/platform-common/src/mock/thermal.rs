use embassy_sync::channel::Sender as ChannelSender;
use embedded_services::info;
use embedded_services::GlobalRawMutex;
use static_cell::StaticCell;
use thermal_service as ts;
use thermal_service_interface::{fan, sensor};
use ts::mock::{fan::MockFan, sensor::MockSensor};

const SENSOR_EVENT_CHANNEL_SIZE: usize = 8;
const FAN_EVENT_CHANNEL_SIZE: usize = 8;

type SensorEventSender = ChannelSender<'static, GlobalRawMutex, sensor::Event, SENSOR_EVENT_CHANNEL_SIZE>;
type FanEventSender = ChannelSender<'static, GlobalRawMutex, fan::Event, FAN_EVENT_CHANNEL_SIZE>;
type SensorService = ts::sensor::Service<'static, MockSensor, SensorEventSender, 16>;
type FanService = ts::fan::Service<'static, MockFan, SensorService, FanEventSender, 16>;
pub type ThermalService = ts::Service<'static, SensorService, FanService>;

pub async fn init(spawner: embassy_executor::Spawner) -> ThermalService {
    info!("Initializing thermal service...");

    // Create and spawn mock sensor service
    let sensor_service = odp_service_common::spawn_service!(
        spawner,
        SensorService,
        ts::sensor::InitParams {
            driver: MockSensor::new(),
            config: MockSensor::config(),
            event_senders: &mut [],
        }
    )
    .expect("Failed to spawn mock sensor service");

    // Create and spawn mock fan service
    let fan_service = odp_service_common::spawn_service!(
        spawner,
        FanService,
        ts::fan::InitParams {
            driver: MockFan::new(),
            config: MockFan::config(),
            sensor_service,
            event_senders: &mut [],
        }
    )
    .expect("Failed to spawn mock fan service");

    // Create the thermal service
    static SENSORS: StaticCell<[SensorService; 1]> = StaticCell::new();
    let sensors = SENSORS.init([sensor_service]);

    static FANS: StaticCell<[FanService; 1]> = StaticCell::new();
    let fans = FANS.init([fan_service]);

    static RESOURCES: StaticCell<ts::Resources<SensorService, FanService>> = StaticCell::new();
    let resources = RESOURCES.init(ts::Resources::default());
    let service = ts::Service::init(resources, ts::InitParams { sensors, fans });

    service
}
