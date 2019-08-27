//! This crate provides abstraction layer for idf framework access.
//! Currently, to obtain access to the specific hardware, client code
//! should own all peripherals with `peripherals::Peripherals::take()`
//! and than move required peripheral parts (e.g. wifi) to the destination
//! (e.g. function, structure, thread).
//! After peripheral was owned, it can be passed to the corresponding
//! wrapper class (e.g. wifi to `wifi::WifiHardware`

#![no_std]

pub mod wifi;
pub mod peripherals;
