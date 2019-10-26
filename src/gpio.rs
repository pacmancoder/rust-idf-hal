use core::marker::PhantomData;

use idf_sys::gpio::*;
use crate::peripherals::GpioPeripherals;

pub struct GpioHardware {
    pub gpio0 : Gpio0,
    pub gpio1 : Gpio1,
    pub gpio2 : Gpio2,
    pub gpio3 : Gpio3,
    pub gpio4 : Gpio4,
    pub gpio5 : Gpio5,
    pub gpio12 : Gpio12,
    pub gpio13 : Gpio13,
    pub gpio14 : Gpio14,
    pub gpio15 : Gpio15,
    pub gpio16 : Gpio16,

    _data : PhantomData<()>,
}

impl GpioHardware {
    pub fn new(_peripherals: GpioPeripherals) -> Self {
        GpioHardware {
            gpio0 : Gpio0 { _data: PhantomData },
            gpio1 : Gpio1 { _data: PhantomData },
            gpio2 : Gpio2 { _data: PhantomData },
            gpio3 : Gpio3 { _data: PhantomData },
            gpio4 : Gpio4 { _data: PhantomData },
            gpio5 : Gpio5 { _data: PhantomData },
            gpio12 : Gpio12 { _data: PhantomData },
            gpio13 : Gpio13 { _data: PhantomData },
            gpio14 : Gpio14 { _data: PhantomData },
            gpio15 : Gpio15 { _data: PhantomData },
            gpio16 : Gpio16 { _data: PhantomData },

            _data : PhantomData,
        }
    }
}

type PinMask = u32;
type PinId = u8;

pub trait GpioPin {
    const PIN_NUM : PinId;

    fn get_pin_id() -> PinId {
        return Self::PIN_NUM;
    }

    fn get_pin_mask() -> PinMask {
        return 1 << Self::PIN_NUM as PinMask;
    }
}

macro_rules! define_gpio_pins {
    ($($type:ident : $id:expr),+) => {$(
        pub struct $type {
            pub(crate) _data: PhantomData<()>
        }

        impl GpioPin for $type {
            const PIN_NUM : PinId = $id;
        }
    )+}
}

define_gpio_pins!(
    Gpio0 : 0,
    Gpio1 : 1,
    Gpio2 : 2,
    Gpio3 : 3,
    Gpio4 : 4,
    Gpio5 : 5,
    Gpio12 : 12,
    Gpio13 : 13,
    Gpio14 : 14,
    Gpio15 : 15,
    Gpio16 : 16
);


pub trait OutputPinMarker {}
pub trait InputPinMarker {}
pub trait OpenDrainPinMarker {}
pub trait PullDownPinMarker {}
pub trait PullUpPinMarker {}
pub trait InterruptPinMarker {}

macro_rules! impl_interrupt_pin_for {
    ($($type:ident),+) => { $(impl InterruptPinMarker for $type {})+ };
}

// All pins except Gpio16 can be configured as interrupt pins
impl_interrupt_pin_for!(
    Gpio0, Gpio1, Gpio2, Gpio3, Gpio4, Gpio5, Gpio12, Gpio13, Gpio14, Gpio15
);


macro_rules! impl_input_pin_for {
    ($($type:ident),+) => { $(impl InputPinMarker for $type {})+ };
}

// All pins can be configured as interrupt pins
impl_input_pin_for!(
    Gpio0, Gpio1, Gpio2, Gpio3, Gpio4, Gpio5, Gpio12, Gpio13, Gpio14, Gpio15, Gpio16
);

macro_rules! impl_output_pin_for {
    ($($type:ident),+) => { $(impl OutputPinMarker for $type {})+ };
}

// All pins can be configured as output pins
impl_output_pin_for!(
    Gpio0, Gpio1, Gpio2, Gpio3, Gpio4, Gpio5, Gpio12, Gpio13, Gpio14, Gpio15, Gpio16
);

macro_rules! impl_open_drain_pin_for {
    ($($type:ident),+) => { $(impl OpenDrainPinMarker for $type {})+ };
}

// All pins except Gpio16 can be configured as open drain pins
impl_open_drain_pin_for!(
    Gpio0, Gpio1, Gpio2, Gpio3, Gpio4, Gpio5, Gpio12, Gpio13, Gpio14, Gpio15
);

macro_rules! impl_pull_down_pin_for {
    ($($type:ident),+) => { $(impl PullDownPinMarker for $type {})+ };
}

// Only Gpio16 can be configured as pull down pin
impl_pull_down_pin_for!(Gpio16);

macro_rules! impl_pull_up_pin_for {
    ($($type:ident),+) => { $(impl PullUpPinMarker for $type {})+ };
}

// All pins except Gpio16 can be configured as pull up pins
impl_pull_up_pin_for!(Gpio0, Gpio1, Gpio2, Gpio3, Gpio4, Gpio5, Gpio12, Gpio13, Gpio14, Gpio15);

#[derive(Copy, Clone)]
pub enum PinInterruptMode {
    Disabled,
    PositiveEdge,
    NegativeEdge,
    AnyEdge,
    LowLevel,
    HighLevel,
}

impl PinInterruptMode {
    fn to_raw(self) -> gpio_int_type_t {
        match self {
            PinInterruptMode::Disabled => gpio_int_type_t_GPIO_INTR_DISABLE,
            PinInterruptMode::PositiveEdge => gpio_int_type_t_GPIO_INTR_POSEDGE,
            PinInterruptMode::NegativeEdge => gpio_int_type_t_GPIO_INTR_NEGEDGE,
            PinInterruptMode::AnyEdge => gpio_int_type_t_GPIO_INTR_ANYEDGE,
            PinInterruptMode::LowLevel => gpio_int_type_t_GPIO_INTR_LOW_LEVEL,
            PinInterruptMode::HighLevel => gpio_int_type_t_GPIO_INTR_HIGH_LEVEL,
        }
    }
}


pub struct PinInitializer<T : GpioPin> {
    _pin: PhantomData<T>,
    config: gpio_config_t,
}

pub struct InitializedPin<T : GpioPin> {
    _pin: PhantomData<T>,
}

impl<T: GpioPin> InitializedPin<T> {
    fn enable_pull_up(&mut self) -> &mut Self where T: PullUpPinMarker {
        unsafe { gpio_pullup_en(T::get_pin_id() as gpio_num_t); };
        self
    }

    fn disable_pull_up(&mut self) -> &mut Self where T: PullUpPinMarker {
        unsafe { gpio_pullup_dis(T::get_pin_id() as gpio_num_t); };
        self
    }

    fn enable_pull_down(&mut self) -> &mut Self where T: PullDownPinMarker {
        unsafe { gpio_pulldown_en(T::get_pin_id() as gpio_num_t); };
        self
    }

    fn disable_pull_down(&mut self) -> &mut Self where T: PullDownPinMarker {
        unsafe { gpio_pulldown_dis(T::get_pin_id() as gpio_num_t); };
        self
    }

    fn configure_as_input(&mut self) -> &mut Self where T: InputPinMarker {
        unsafe { gpio_set_direction(T::get_pin_id() as gpio_num_t, gpio_mode_t_GPIO_MODE_INPUT); };
        self
    }

    fn configure_as_output(&mut self) -> &mut Self where T: OutputPinMarker {
        unsafe { gpio_set_direction(T::get_pin_id() as gpio_num_t, gpio_mode_t_GPIO_MODE_OUTPUT); };
        self
    }

    fn configure_as_open_drain(&mut self) -> &mut Self where T: OpenDrainPinMarker {
        unsafe {
            gpio_set_direction(T::get_pin_id() as gpio_num_t, gpio_mode_t_GPIO_MODE_OUTPUT_OD);
        };
        self
    }

    fn set_interrupt_mode(&mut self, mode: PinInterruptMode) -> &mut Self where T: InterruptPinMarker {
        unsafe { gpio_set_intr_type(T::get_pin_id() as gpio_num_t, mode.to_raw()); };
        self
    }
}

pub trait InputPin {
    fn get_level(&self) -> bool;
}

impl<T> InputPin for InitializedPin<T> where T: GpioPin + InputPinMarker {
    fn get_level(&self) -> bool {
        (unsafe { gpio_get_level(T::get_pin_id() as gpio_num_t) }) != 0
    }
}

pub trait OutputPin {
    fn set_level(&mut self, value: bool);
}

impl<T> OutputPin for InitializedPin<T> where T: GpioPin + OutputPinMarker {
    fn set_level(&mut self, value: bool) {
        unsafe { gpio_set_level(T::get_pin_id() as gpio_num_t, value as u32) };
    }
}


impl<T : GpioPin> PinInitializer<T> {
    pub fn new(_pin: T) -> Self {
        Self {
            config: gpio_config_t {
                pin_bit_mask: T::get_pin_mask(),
                mode: gpio_mode_t_GPIO_MODE_DISABLE,
                pull_up_en: gpio_pullup_t_GPIO_PULLUP_DISABLE,
                pull_down_en: gpio_pulldown_t_GPIO_PULLDOWN_DISABLE,
                intr_type: gpio_int_type_t_GPIO_INTR_DISABLE,
            },
            _pin: PhantomData
        }
    }

    pub fn enable_pull_up(mut self) -> Self where T: PullUpPinMarker {
        self.config.pull_up_en = gpio_pullup_t_GPIO_PULLUP_ENABLE;
        self
    }

    pub fn enable_pull_down(mut self) -> Self where T: PullDownPinMarker {
        self.config.pull_down_en = gpio_pulldown_t_GPIO_PULLDOWN_ENABLE;
        self
    }

    pub fn configure_as_input(mut self) -> Self where T: InputPinMarker {
        self.config.mode = gpio_mode_t_GPIO_MODE_INPUT;
        self
    }

    pub fn configure_as_output(mut self) -> Self where T: OutputPinMarker {
        self.config.mode = gpio_mode_t_GPIO_MODE_OUTPUT;
        self
    }

    pub fn configure_as_open_drain(mut self) -> Self where T: OpenDrainPinMarker {
        self.config.mode = gpio_mode_t_GPIO_MODE_OUTPUT_OD;
        self
    }

    pub fn set_interrupt_mode(mut self, mode: PinInterruptMode) -> Self where T: InterruptPinMarker {
        self.config.intr_type = mode.to_raw();
        self
    }

    pub fn init(self) -> InitializedPin<T> {
        unsafe { gpio_config(&self.config); };
        InitializedPin { _pin : PhantomData }
    }
}