//! Clock configuration for the MCXA dev board.

use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::clocks::config::{
    ClocksConfig, CoreSleep, Div8, FircConfig, FircFreqSel, FlashSleep, MainClockConfig, MainClockSource,
    VddDriveStrength, VddLevel,
};

/// Build the desired clock tree configuration for the board.
pub fn config() -> ClocksConfig {
    let mut cfg = ClocksConfig::default();

    // Enable 180MHz clock source
    let mut fcfg = FircConfig::default();
    fcfg.frequency = FircFreqSel::Mhz180;
    fcfg.power = PoweredClock::NormalEnabledDeepSleepDisabled;
    fcfg.fro_hf_enabled = true;
    fcfg.clk_hf_fundamental_enabled = false;
    fcfg.fro_hf_div = Some(const { Div8::from_divisor(4).unwrap() });
    cfg.firc = Some(fcfg);

    // Enable 12M osc
    cfg.sirc.fro_12m_enabled = true;
    cfg.sirc.fro_lf_div = Some(Div8::no_div());
    cfg.sirc.power = PoweredClock::AlwaysEnabled;

    // Disable 16K osc
    cfg.fro16k = None;

    // Disable external osc
    cfg.sosc = None;

    // Disable PLL
    cfg.spll = None;

    // Feed core from 180M osc
    cfg.main_clock = MainClockConfig {
        source: MainClockSource::FircHfRoot,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        ahb_clk_div: Div8::no_div(),
    };

    // We don't sleep, set relatively high power
    cfg.vdd_power.active_mode.level = VddLevel::OverDriveMode;
    cfg.vdd_power.low_power_mode.level = VddLevel::MidDriveMode;
    cfg.vdd_power.active_mode.drive = VddDriveStrength::Normal;
    cfg.vdd_power.low_power_mode.drive = VddDriveStrength::Low { enable_bandgap: false };

    // Set "never sleep" mode
    cfg.vdd_power.core_sleep = CoreSleep::WfeUngated;

    // Set flash doze, allowing internal flash clocks to be gated on sleep
    cfg.vdd_power.flash_sleep = FlashSleep::FlashDoze;

    cfg
}
