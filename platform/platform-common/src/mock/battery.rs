use battery_service as bs;
use bs::mock::MockFuelGauge;
use bs::FuelGauge as _;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use embedded_services::{error, info, GlobalRawMutex};
use static_cell::StaticCell;

/// The fuel gauge, behind a mutex so the service and the driving task can share it.
type FuelGauge = Mutex<GlobalRawMutex, MockFuelGauge>;
/// Two registered fuel gauges: battery `0` is a 3S pack, battery `1` a 2S pack.
type Reg = bs::ArrayRegistration<'static, FuelGauge, 2>;
pub type BatteryService = bs::Service<'static, Reg>;

pub async fn init(spawner: embassy_executor::Spawner) -> BatteryService {
    info!("Initializing battery service...");

    static FUEL_GAUGE_0: StaticCell<FuelGauge> = StaticCell::new();
    let fuel_gauge_0: &'static FuelGauge = FUEL_GAUGE_0.init(Mutex::new(MockFuelGauge::new()));

    static FUEL_GAUGE_1: StaticCell<FuelGauge> = StaticCell::new();
    let fuel_gauge_1: &'static FuelGauge = FUEL_GAUGE_1.init(Mutex::new(MockFuelGauge::new_2s()));

    let service = bs::Service::new(bs::ArrayRegistration {
        fuel_gauges: [fuel_gauge_0, fuel_gauge_1],
    });

    for (battery_id, fuel_gauge) in [fuel_gauge_0, fuel_gauge_1].into_iter().enumerate() {
        bs::mock::init_state_machine(fuel_gauge)
            .await
            .expect("Failed to initialize battery state machine");
        spawner
            .spawn(update_data_task(battery_id as u8, fuel_gauge).expect("Failed to spawn battery update data task"));
    }

    service
}

#[embassy_executor::task(pool_size = 2)]
pub async fn update_data_task(battery_id: u8, fuel_gauge: &'static FuelGauge) -> ! {
    let mut failures: u32 = 0;
    let mut count: usize = 0;
    loop {
        Timer::after_secs(1).await;
        if count.is_multiple_of(const { 60 * 60 * 60 }) {
            if let Err(e) = fuel_gauge.lock().await.update_static_data().await {
                failures += 1;
                error!("FG {}: Static data error: {:?}", battery_id, defmt::Debug2Format(&e));
            }
        }
        if let Err(e) = fuel_gauge.lock().await.update_dynamic_data().await {
            failures += 1;
            error!("FG {}: Dynamic data error: {:?}", battery_id, defmt::Debug2Format(&e));
        }

        if failures > 10 {
            failures = 0;
            count = 0;
            error!(
                "FG {}: Too many errors, timing out and starting recovery...",
                battery_id
            );
            if bs::mock::recover_state_machine(fuel_gauge).await.is_err() {
                error!("FG {}: Failed to recover state machine!", battery_id);
            }
        }

        count = count.wrapping_add(1);
    }
}
