// TODO: Implement atomic singleton when atomics will be available in LLVM-rs

// use core::sync::atomic::{ AtomicBool, Ordering };

pub struct WiFiPeripherals {}

pub struct OwnedPeripherals {
    pub wifi: WiFiPeripherals,
}

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