use idf_sys::watchdog::*;

pub fn reset_watchdog() {
    unsafe { esp_task_wdt_reset() };
}