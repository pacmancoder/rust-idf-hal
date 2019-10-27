use idf_sys::freertos::*;

pub fn delay_ms(ms: usize) {
    unsafe {
        vTaskDelay(ms / (1000 / xPortGetTickRateHz()))
    };
}

pub fn task_yield() {
    unsafe { PendSV(1) };
}