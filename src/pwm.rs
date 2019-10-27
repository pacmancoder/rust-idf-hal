use crate::gpio::*;

use idf_sys::{
    error::*,
    pwm::*
};

const MAX_PWM_CHANNELS : usize = 8;

#[derive(Copy, Clone)]
struct PwmChannel {
    pin: PinId,
    duty: u32,
}

#[derive(Copy, Clone)]
pub enum PwmInitializationError {
    TooManyChannels,
    TooShortPeriod,
    DutyExceedsPeriod,
    PeriodNotSet,
}

#[derive(Copy, Clone)]
pub enum PwmConfigurationError {
    InvalidChannel,
    TooShortPeriod,
    DutyExceedsPeriod,
    InvalidPhase,
}

pub struct PwmConfiguration {
    pub(crate) channel_count: u8,
    pub(crate) period: u32,
    pub(crate) stop_level: u8,
}

impl PwmConfiguration {
    fn assert_channel(&self, channel: u8) -> Result<(), PwmConfigurationError> {
        if channel >= self.channel_count {
            Err(PwmConfigurationError::InvalidChannel)
        } else {
            Ok(())
        }
    }

    pub fn set_stop_level(&mut self, channel: u8, level: bool)
        -> Result<&mut Self, PwmConfigurationError>
    {
        self.assert_channel(channel)?;

        let bit_mask = 1u8 << channel;
        let bit_value = (level as u8) << channel;

        self.stop_level &= !bit_mask;
        self.stop_level |= bit_value;

        Ok(self)
    }

    pub fn set_inverted_state(&mut self, channel: u8, is_inverted: bool)
                          -> Result<&mut Self, PwmConfigurationError>
    {
        self.assert_channel(channel)?;
        if is_inverted {
            unsafe { pwm_set_channel_invert(1u16 << channel as u16) };
        } else {
            unsafe { pwm_clear_channel_invert(1u16 << channel as u16) };
        }
        Ok(self)
    }

    pub fn set_period(&mut self, period: u32) -> Result<&mut Self, PwmConfigurationError> {
        if unsafe { pwm_set_period(period)} != esp_err_t_ESP_OK {
            return Err(PwmConfigurationError::TooShortPeriod)
        }

        self.period = period;
        Ok(self)
    }

    pub fn set_duty(&mut self, channel: u8, duty: u32) -> Result<&mut Self, PwmConfigurationError> {
        if duty > self.period {
            return Err(PwmConfigurationError::DutyExceedsPeriod);
        }
        if unsafe { pwm_set_duty(channel, duty) } != esp_err_t_ESP_OK {
            return Err(PwmConfigurationError::InvalidChannel);
        }
        Ok(self)
    }

    pub fn set_phase(&mut self, channel: u8, phase: i16)
        -> Result<&mut Self, PwmConfigurationError>
    {
        if phase < -180 || phase > 180 {
            return Err(PwmConfigurationError::InvalidPhase);
        }

        if unsafe { pwm_set_phase(channel, phase) } != esp_err_t_ESP_OK {
            return Err(PwmConfigurationError::InvalidChannel);
        }

        Ok(self)
    }
}

pub struct Pwm {
    configuration: PwmConfiguration,
}

impl Pwm {
    pub(crate) fn new(channel_count: u8, period: u32) -> Self {
        Self {
            configuration: PwmConfiguration {
                channel_count,
                period,
                stop_level: 0,
            }
        }
    }

    pub fn configure<F>(&mut self, configure: F) -> Result<&mut Self, PwmConfigurationError>
        where F: FnOnce(&mut PwmConfiguration) -> Result<(), PwmConfigurationError>
    {
        configure(&mut self.configuration)?;
        Ok(self)
    }

    pub fn start(&mut self) -> &mut Self {
        unsafe { pwm_start(); }
        self
    }

    pub fn stop(&mut self) -> &mut Self {
        unsafe { pwm_stop(self.configuration.stop_level as u32); }
        self
    }

    pub fn deinitialize(mut self) {
        self.stop();
        unsafe { pwm_deinit() };
    }
}

pub struct PwmInitializer {
    channels_count: u8,
    channels: [PwmChannel; MAX_PWM_CHANNELS],
    period: Option<u32>,
}

impl PwmInitializer {
    pub fn new() -> Self {
        Self {
            channels_count: 0,
            channels: [PwmChannel { pin: 0, duty: 0 }; MAX_PWM_CHANNELS],
            period: None,
        }
    }

    pub fn add_channel<Pin : GpioPin + PwmPinMarker>(mut self, _pin: Pin, duty: u32)
        -> Result<Self, PwmInitializationError>
    {
        if self.channels_count < 8 {
            self.channels[self.channels_count as usize] = PwmChannel {
                pin: Pin::get_pin_id(),
                duty
            };
            self.channels_count += 1;
            Ok(self)
        } else {
            Err(PwmInitializationError::TooManyChannels)
        }
    }

    pub fn set_period(mut self, period: u32) -> Result<Self, PwmInitializationError> {
        if period < 10 {
            Err(PwmInitializationError::TooShortPeriod)
        } else {
            self.period = Some(period);
            Ok(self)
        }
    }

    pub fn initialize(self) -> Result<Pwm, (PwmInitializationError, Self)> {
        if let None = self.period {
            return Err((PwmInitializationError::PeriodNotSet, self));
        }


        let period = self.period.unwrap();
        let mut duties : [u32; MAX_PWM_CHANNELS] = [0; MAX_PWM_CHANNELS];
        let mut pins : [u32; MAX_PWM_CHANNELS] = [0; MAX_PWM_CHANNELS];

        for i in 0..MAX_PWM_CHANNELS {
            if self.channels[i].duty > period {
                return Err((PwmInitializationError::DutyExceedsPeriod, self));
            }

            duties[i] = self.channels[i].duty;
            pins[i] = self.channels[i].pin as u32;
        }

        unsafe { pwm_init(period, duties.as_mut_ptr(), self.channels_count, pins.as_mut_ptr()) };

        Ok(Pwm::new(self.channels_count, period))
    }
}


