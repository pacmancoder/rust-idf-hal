//! This crate provides abstraction layer for idf framework access.
//! To obtain access to the specific hardware, client code
//! should own all peripherals with `peripherals::Peripherals::take()`
//! and then move required peripheral parts (e.g. wifi) to the destination
//! (e.g. function, structure, thread). These peripheral parts are serving as virtual "handles",
//! and to make actual work with them, they should be passed to wrapper classses such as
//! [WifiHardware](wifi/struct.WiFiHardware.html)
//!
//! See example of peripheral initialization in the [wifi](wifi/index.html) crate

#![no_std]

pub mod wifi;
pub mod peripherals;
