use battery_service as bs;
use bs::mock::{MockBattery, MockBatteryDriver};
use embassy_time::Timer;
use embedded_services::{error, info};
use static_cell::StaticCell;

type BatteryService = bs::Service<'static, 1>;

// The mock battery requires device ID of 0
const BAT_ID: bs::device::DeviceId = bs::device::DeviceId(0);

pub async fn init(spawner: embassy_executor::Spawner) -> BatteryService {
    info!("Initializing battery service...");

    static BATTERY_DEVICE: StaticCell<bs::device::Device> = StaticCell::new();
    let device = BATTERY_DEVICE.init(bs::device::Device::new(BAT_ID));
    let driver = MockBatteryDriver::new();
    let battery = MockBattery::new(device, driver);

    let service = odp_service_common::spawn_service!(
        spawner,
        BatteryService,
        bs::InitParams {
            devices: [device],
            config: bs::context::Config::default(),
        }
    )
    .expect("Failed to initialize battery service");

    spawner.spawn(battery_device_controller_task(battery).expect("Failed to spawn battery device controller task"));

    bs::mock::init_state_machine(&service)
        .await
        .expect("Failed to initialize battery state machine");
    spawner.spawn(update_data_task(service.clone()).expect("Failed to spawn battery update data task"));

    service
}

#[embassy_executor::task]
async fn battery_device_controller_task(battery: MockBattery<'static>) {
    battery.process().await;
}

#[embassy_executor::task]
pub async fn update_data_task(service: BatteryService) -> ! {
    let mut failures: u32 = 0;
    let mut count: usize = 0;
    loop {
        Timer::after_secs(1).await;
        if count.is_multiple_of(const { 60 * 60 * 60 }) {
            if let Err(e) = service
                .execute_event(bs::context::BatteryEvent {
                    event: bs::context::BatteryEventInner::PollStaticData,
                    device_id: BAT_ID,
                })
                .await
            {
                failures += 1;
                error!("FG: Static data error: {:#?}", e);
            }
        }
        if let Err(e) = service
            .execute_event(bs::context::BatteryEvent {
                event: bs::context::BatteryEventInner::PollDynamicData,
                device_id: BAT_ID,
            })
            .await
        {
            failures += 1;
            error!("FG: Dynamic data error: {:#?}", e);
        }

        if failures > 10 {
            failures = 0;
            count = 0;
            error!("FG: Too many errors, timing out and starting recovery...");
            if bs::mock::recover_state_machine(&service).await.is_err() {
                error!("FG: Failed to recover state machine!");
            }
        }

        count = count.wrapping_add(1);
    }
}
