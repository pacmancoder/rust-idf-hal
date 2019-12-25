use idf_sys::{
    system_event::*,
    ffi::*,
    wifi::*,
};
use alloc::boxed::Box;
use idf_sys::error::{esp_err_t, esp_err_t_ESP_OK};

#[non_exhaustive]
pub struct StaConnectedEvent {}

#[non_exhaustive]
#[derive(Eq, PartialEq)]
pub enum StaDisconnectReason {
    /// Usually signals that wifi should be switched to bgn mode
    BasicRateIsNotSupported,
    Unknown,
}

#[non_exhaustive]
pub struct StaDisconnectedEvent {
    pub reason: StaDisconnectReason,
}

pub enum SystemEvent {
    StaStarted,
    StaConnected(StaConnectedEvent),
    StaDisconnected(StaDisconnectedEvent),
    Unknown,
}

mod sys_to_hal {
    use super::*;

    pub fn make_sta_disconnect_reason(reason: wifi_err_reason_t) -> StaDisconnectReason {
        match reason {
            wifi_err_reason_t_WIFI_REASON_BASIC_RATE_NOT_SUPPORT =>
                StaDisconnectReason::BasicRateIsNotSupported,
            _ => StaDisconnectReason::Unknown,
        }
    }
}

unsafe extern "C" fn event_loop_wrapper<F>(ctx: *mut xtensa_void, event: *mut system_event_t) -> esp_err_t
    where F: FnMut(SystemEvent) + Send + 'static
{
    let mut closure = Box::<F>::from_raw(ctx as *mut F);

    closure(match (*event).event_id {
        system_event_id_t_SYSTEM_EVENT_STA_START => SystemEvent::StaStarted,
        system_event_id_t_SYSTEM_EVENT_STA_CONNECTED => {
            SystemEvent::StaConnected( StaConnectedEvent{} )
        }
        system_event_id_t_SYSTEM_EVENT_STA_DISCONNECTED => {
            SystemEvent::StaDisconnected(
                StaDisconnectedEvent {
                    reason: sys_to_hal::make_sta_disconnect_reason(
                        (*event).event_info.disconnected.reason as u32
                    )
                }
            )
        }
        _ => SystemEvent::Unknown
    });

    /// release closure back to avoid destruction
    Box::into_raw(closure);
    esp_err_t_ESP_OK
}

pub fn set_event_loop<F>(loop_handler: F)
    where F: FnMut(SystemEvent) + Send + 'static
{
    unsafe {
        let closure_ptr = Box::into_raw(Box::new(loop_handler));

        esp_event_loop_init(Some(event_loop_wrapper::<F>), closure_ptr as *mut xtensa_void);
    }
}