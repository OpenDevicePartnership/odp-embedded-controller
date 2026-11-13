use battery_service as bs;
use bs::mock::MockFuelGauge;
use bs::{DefaultPowerPolicyRouting, FuelGauge as _};
use embassy_sync::channel::{Channel, DynamicReceiver, DynamicSender};
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use embedded_services::{error, info, GlobalRawMutex};
use static_cell::StaticCell;

/// The fuel gauge, behind a mutex so the service and the driving task can share it.
type FuelGauge = Mutex<GlobalRawMutex, MockFuelGauge>;
const FUEL_GAUGE_COUNT: usize = if cfg!(feature = "dual-fuel-gauge") { 2 } else { 1 };
type Reg = bs::ArrayRegistration<'static, FuelGauge, FUEL_GAUGE_COUNT, DefaultPowerPolicyRouting>;
pub type BatteryService = bs::Service<'static, Reg>;

/// Initialize the battery service, returning it with the sender for its power-policy notifications.
pub async fn init(
    spawner: embassy_executor::Spawner,
) -> (
    &'static BatteryService,
    DynamicSender<'static, bs::PowerPolicyEventData>,
) {
    info!("Initializing battery service...");

    static FUEL_GAUGE_0: StaticCell<FuelGauge> = StaticCell::new();
    let fuel_gauge_0: &'static FuelGauge = FUEL_GAUGE_0.init(Mutex::new(MockFuelGauge::new()));

    let fuel_gauges = [
        fuel_gauge_0,
        #[cfg(feature = "dual-fuel-gauge")]
        {
            static FUEL_GAUGE_1: StaticCell<FuelGauge> = StaticCell::new();
            let fuel_gauge_1: &'static FuelGauge = FUEL_GAUGE_1.init(Mutex::new(MockFuelGauge::new_2s()));
            fuel_gauge_1
        },
    ];

    static SERVICE: StaticCell<BatteryService> = StaticCell::new();
    let service: &'static BatteryService = SERVICE.init(bs::Service::new(bs::ArrayRegistration {
        fuel_gauges,
        routing: DefaultPowerPolicyRouting,
    }));

    for (battery_id, fuel_gauge) in fuel_gauges.into_iter().enumerate() {
        bs::mock::init_state_machine(fuel_gauge)
            .await
            .expect("Failed to initialize battery state machine");
        spawner
            .spawn(update_data_task(battery_id as u8, fuel_gauge).expect("Failed to spawn battery update data task"));
    }

    static POWER_POLICY_EVENTS: StaticCell<Channel<GlobalRawMutex, bs::PowerPolicyEventData, 4>> = StaticCell::new();
    let power_policy_events = POWER_POLICY_EVENTS.init(Channel::new());
    spawner.spawn(
        power_policy_event_task(service, power_policy_events.dyn_receiver())
            .expect("Failed to spawn battery power-policy event task"),
    );

    (service, power_policy_events.dyn_sender())
}

/// Apply power-policy notifications to the fuel gauges' cached external power state.
#[embassy_executor::task]
async fn power_policy_event_task(
    service: &'static BatteryService,
    events: DynamicReceiver<'static, bs::PowerPolicyEventData>,
) -> ! {
    loop {
        let event = events.receive().await;
        if let Err(e) = service.process_power_policy_event(&event).await {
            error!("Battery power-policy event error: {:?}", e);
        }
    }
}

#[embassy_executor::task(pool_size = FUEL_GAUGE_COUNT)]
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
