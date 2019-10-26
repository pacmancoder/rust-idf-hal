//! This module provides access to `Peripherals` struct, which can be
//! used to get access to owned peripherals instance.
//!
//! **NOTE:** In the current implementation
//! [Peripherals::take()](struct.Peripherals.html#method.take) is not hread-safe. Please avoid
//! calling this method after from multiple threads until issue will be fixed
//!
//! # Examples:
//! ```rust
//! # use idf_hal::peripherals::Peripherals;
//!
//! let peripherals = Peripherals::take().unwrap();
//! let owned_wifi = peripherals.wifi;
//! // Use wifi peripherals
//! ```
use core::marker::PhantomData;

// TODO: Implement atomic singleton when atomics will be available in LLVM-rs

/// Represents owned wifi peripherals
pub struct WiFiPeripherals {}

/// Represents owned gpio peripherals
pub struct GpioPeripherals {}

/// Represents owned idf peripherals. Can be deconstructed on the parts with the public fields
/// for more granular access
pub struct OwnedPeripherals {
    /// Owned WiFi peripherals
    pub wifi: WiFiPeripherals,
    pub gpio: GpioPeripherals,
    _data : PhantomData<()>,
}

/// Provides access to IDF peripherals
pub struct Peripherals {
    data: Option<OwnedPeripherals>,
}

static mut PERIPHERALS_SINGLETON : Peripherals = Peripherals::new();

impl OwnedPeripherals {
    const fn new() -> OwnedPeripherals {
        OwnedPeripherals {
            wifi: WiFiPeripherals {},
            gpio: GpioPeripherals {},
            _data: PhantomData,
        }
    }
}

impl Peripherals {
    const fn new() -> Peripherals {
        Peripherals {
            data: Some(OwnedPeripherals::new()),
        }
    }

    /// Owns idf peripherals
    /// returns [OwnedPeripherals](struct.OwnedPeripherals.html) on success or `None` if peripherals
    /// were already taken
    pub fn take() -> Option<OwnedPeripherals> {
        unsafe {
            PERIPHERALS_SINGLETON.data.take()
        }
    }
}
