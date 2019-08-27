//! This module provides access to `Peripherals` struct, which can be
//! used to get access to owned peripherals instance.
//!
//! ### Code example:
//! ```rust
//! use idf_hal::peripherals::Peripherals;
//!
//! let peripherals = Peripherals.take().unwrap();
//! let owned_wifi = peripherals.wifi;
//! // Use wifi peripherals
//! ```

// TODO: Implement atomic singleton when atomics will be available in LLVM-rs

// use core::sync::atomic::{ AtomicBool, Ordering };

/// Represents owned wifi peripherals
pub struct WiFiPeripherals {}

/// Represents owned idf peripherals. Can be deconstructed on the parts
/// for more granular access
pub struct OwnedPeripherals {
    /// Wifi peripherials
    pub wifi: WiFiPeripherals,
}

/// Provides access to IDF peripherals
pub struct Peripherals {
    // owned: AtomicBool,
    data: Option<OwnedPeripherals>,
}

static mut PERIPHERALS_SINGLETON : Peripherals = Peripherals::new();

impl OwnedPeripherals {
    const fn new() -> OwnedPeripherals {
        OwnedPeripherals {
            wifi: WiFiPeripherals {},
        }
    }
}

impl Peripherals {
    const fn new() -> Peripherals {
        Peripherals {
            // owned: AtomicBool::new(false),
            data: Some(OwnedPeripherals::new()),
        }
    }

    /// Owns idf peripherals
    /// returns Some(OwnedPeripherals) on success or None if peripherals
    /// already were taken
    pub fn take() -> Option<OwnedPeripherals> {
        unsafe {
            PERIPHERALS_SINGLETON.data.take()
            /*
            if PERIPHERALS_SINGLETON.owned.compare_and_swap(false, true, Ordering::SeqCst) == false
            {
                PERIPHERALS_SINGLETON.data.take()
            } else {
                None
            }
            */
        }
    }
}
