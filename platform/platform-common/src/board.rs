//! Board IO abstraction for standardized hardware initialization.
//!
//! This module defines the [`BoardIo`] trait that each platform implements
//! to provide a standardized structure for hardware IO initialization.
//! By following this pattern, all platforms organize their GPIO pins,
//! I2C buses, and other peripherals in a consistent way.
//!
//! # Pattern
//!
//! Each platform creates a `board` module containing:
//!
//! 1. Subsystem-specific IO structs grouping related pins (e.g., power
//!    sequence enables, monitoring inputs, I2C buses)
//! 2. A top-level `Board` struct that aggregates all subsystem IO
//! 3. An implementation of [`BoardIo`] with the platform's HAL peripherals
//!    type
//!
//! # Example
//!
//! ```ignore
//! use platform_common::board::BoardIo;
//!
//! pub struct Board {
//!     pub power_button: gpio::Input<'static>,
//!     pub led: gpio::Output<'static>,
//! }
//!
//! impl BoardIo for Board {
//!     type Peripherals = hal::Peripherals;
//!
//!     fn init(p: Self::Peripherals) -> Self {
//!         Board {
//!             power_button: gpio::Input::new(p.PIN_X, Pull::None),
//!             led: gpio::Output::new(p.PIN_Y, Level::Low),
//!         }
//!     }
//! }
//!
//! // In main.rs:
//! let p = hal::init(Default::default());
//! let board = Board::init(p);
//! spawner.spawn(button_task(board.power_button).expect("Failed to spawn button task"));
//! ```

/// Trait for standardized board IO initialization.
///
/// Each platform implements this trait on a `Board` struct that groups
/// all hardware IO by subsystem. The `init` method configures all GPIO
/// pins, I2C buses, and other peripherals needed by the platform,
/// returning them organized in subsystem-specific groups.
///
/// The associated `Peripherals` type allows each platform to specify
/// what HAL-specific resources are needed for initialization.
///
/// # Panics
///
/// Implementations may panic if hardware initialization fails (e.g.,
/// I2C bus creation error). This follows the project convention of
/// using `.expect()` for initialization failures that should never
/// happen at runtime.
pub trait BoardIo {
    /// HAL-specific peripherals type consumed during initialization.
    ///
    /// This is typically the `Peripherals` struct from the platform's
    /// Embassy HAL crate (e.g., `embassy_imxrt::Peripherals`).
    type Peripherals;

    /// Initialize all board-level IO from raw HAL peripherals.
    ///
    /// This method should configure all GPIO pins, I2C buses, and other
    /// peripherals needed by the platform.
    fn init(peripherals: Self::Peripherals) -> Self;
}
