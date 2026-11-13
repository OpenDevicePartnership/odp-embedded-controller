//! Mock power policy with one PSU: a DC jack that a mock wall adapter can be plugged into.
//!
//! The policy's notifications go to the battery service, so its `_PSR` follows the adapter.

use embassy_sync::channel::{Channel, DynamicReceiver, DynamicSender};
use embassy_sync::mutex::Mutex;
use embedded_services::event::MapSender;
use embedded_services::named::Named;
use embedded_services::{info, GlobalRawMutex};
use power_policy_interface::capability::{
    ConsumerFlags, ConsumerPowerCapability, PowerCapability, ProviderPowerCapability, PsuType,
};
use power_policy_interface::charger::mock::ChargerType;
use power_policy_interface::psu::{self, event::EventData as PsuEventData, PsuState};
use power_policy_interface::service::event::{Event, EventData as PowerPolicyEventData, SenderNotifier};
use power_policy_service as pps;
use pps::psu::PsuEventReceivers;
use pps::service::registration::ArrayRegistration;
use static_cell::StaticCell;

/// The mock wall adapter: 20 V at 3 A from an outlet.
const ADAPTER: ConsumerPowerCapability = ConsumerPowerCapability {
    capability: PowerCapability {
        voltage_mv: 20_000,
        current_ma: 3_000,
    },
    flags: ConsumerFlags {
        unconstrained_power: true,
        psu_type: Some(PsuType::DcJack),
    },
};

/// The mock DC jack, registered with the power policy as its only PSU.
pub struct MockPsu {
    state: psu::State,
}

impl psu::Psu for MockPsu {
    async fn disconnect(&mut self) -> Result<(), psu::Error> {
        self.state.disconnect(false)
    }

    async fn connect_provider(&mut self, capability: ProviderPowerCapability) -> Result<(), psu::Error> {
        self.state.connect_provider(capability)
    }

    async fn connect_consumer(&mut self, capability: ConsumerPowerCapability) -> Result<(), psu::Error> {
        self.state.connect_consumer(capability)
    }

    fn state(&self) -> &psu::State {
        &self.state
    }

    fn state_mut(&mut self) -> &mut psu::State {
        &mut self.state
    }
}

impl Named for MockPsu {
    fn name(&self) -> &'static str {
        "Mock PSU"
    }
}

pub type Psu = Mutex<GlobalRawMutex, MockPsu>;
const PSU_COUNT: usize = 1;

// Service events reference their PSU, but the battery service's default routing takes only the payload.
type ServiceEvent = Event<'static, Psu>;
type BatteryEventSender = MapSender<
    ServiceEvent,
    PowerPolicyEventData,
    DynamicSender<'static, PowerPolicyEventData>,
    fn(ServiceEvent) -> PowerPolicyEventData,
>;
type ServiceNotifier = SenderNotifier<'static, Psu, BatteryEventSender>;
const NOTIFIER_COUNT: usize = 1;
const CHARGER_COUNT: usize = 0;

type Reg = ArrayRegistration<'static, Psu, PSU_COUNT, ServiceNotifier, NOTIFIER_COUNT, ChargerType, CHARGER_COUNT>;
pub type PowerPolicyService = Mutex<GlobalRawMutex, pps::service::Service<'static, Reg>>;

/// Plugs the mock adapter into the mock PSU, or unplugs it.
pub struct MockPsuControl {
    psu: &'static Psu,
    events: DynamicSender<'static, PsuEventData>,
}

impl MockPsuControl {
    /// Plug the adapter in (`true`) or unplug it (`false`), notifying the power policy of any change.
    pub async fn set_attached(&mut self, attached: bool) -> Result<(), psu::Error> {
        {
            let mut psu = self.psu.lock().await;
            let state = &mut psu.state;
            if attached == (state.psu_state != PsuState::Detached) {
                return Ok(());
            }
            if attached {
                state.attach()?;
                state.update_consumer_power_capability(Some(ADAPTER))?;
            } else {
                state.detach();
            }
        }

        // Notify after releasing the PSU: the policy task locks it while handling events, so waiting
        // for room in the event channel while holding it could deadlock.
        if attached {
            info!("Mock PSU: adapter plugged in");
            self.events.send(PsuEventData::Attached).await;
            self.events
                .send(PsuEventData::UpdatedConsumerCapability(Some(ADAPTER)))
                .await;
        } else {
            info!("Mock PSU: adapter unplugged");
            self.events.send(PsuEventData::Detached).await;
        }
        Ok(())
    }
}

/// Start the power policy with the mock PSU, sending its notifications to `battery_events`.
///
/// The adapter starts unplugged.
pub fn init(
    spawner: embassy_executor::Spawner,
    battery_events: DynamicSender<'static, PowerPolicyEventData>,
) -> MockPsuControl {
    info!("Initializing power policy service...");

    static PSU_EVENTS: StaticCell<Channel<GlobalRawMutex, PsuEventData, 4>> = StaticCell::new();
    let psu_events = PSU_EVENTS.init(Channel::new());

    static PSU: StaticCell<Psu> = StaticCell::new();
    let psu: &'static Psu = PSU.init(Mutex::new(MockPsu {
        state: psu::State::default(),
    }));

    let to_payload: fn(ServiceEvent) -> PowerPolicyEventData = PowerPolicyEventData::from;
    static SERVICE: StaticCell<PowerPolicyService> = StaticCell::new();
    let service = SERVICE.init(Mutex::new(pps::service::Service::new(
        ArrayRegistration {
            psus: [psu],
            service_notifiers: [SenderNotifier::new(MapSender::new(battery_events, to_payload))],
            chargers: [],
        },
        pps::service::config::Config::default(),
    )));

    spawner.spawn(
        power_policy_task(PsuEventReceivers::new([psu], [psu_events.dyn_receiver()]), service)
            .expect("Failed to spawn power policy task"),
    );

    MockPsuControl {
        psu,
        events: psu_events.dyn_sender(),
    }
}

#[embassy_executor::task]
async fn power_policy_task(
    psu_events: PsuEventReceivers<'static, PSU_COUNT, Psu, DynamicReceiver<'static, PsuEventData>>,
    service: &'static PowerPolicyService,
) -> ! {
    pps::service::task::psu_task(psu_events, service).await
}
