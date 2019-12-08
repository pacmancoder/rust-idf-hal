use crate::{
    gpio::*,
    peripherals::UartPeripherals,
};

use idf_sys::{
    uart::*,
    ffi::*,
    error::*,
};
use core::{
    ptr::{ null_mut },
    marker::PhantomData,
};

pub enum UartConfigError {
    InvalidBaudRate,
    InvalidRxThreshold,
    InvalidRxBufferSize,
    InvalidTxBufferSize,
    Unknown,
    #[deprecated(note = "Check UartConfigError with default match clause (_ => {...})")]
    __NonExhaustive,
}

pub trait CaptureGpioPin {
    fn capture_pin(gpio_hw: &mut GpioHardware);
    fn release_pin(gpio_hw: &mut GpioHardware);
}

impl CaptureGpioPin for Gpio1 {
    fn capture_pin(gpio_hw: &mut GpioHardware) { gpio_hw.gpio1.take(); }
    fn release_pin(gpio_hw: &mut GpioHardware) { gpio_hw.gpio1.replace( Gpio1::new() ); }
}

impl CaptureGpioPin for Gpio2 {
    fn capture_pin(gpio_hw: &mut GpioHardware) { gpio_hw.gpio2.take(); }
    fn release_pin(gpio_hw: &mut GpioHardware) { gpio_hw.gpio2.replace( Gpio2::new() ); }
}

impl CaptureGpioPin for Gpio3 {
    fn capture_pin(gpio_hw: &mut GpioHardware) { gpio_hw.gpio3.take(); }
    fn release_pin(gpio_hw: &mut GpioHardware) { gpio_hw.gpio3.replace( Gpio3::new() ); }
}

impl CaptureGpioPin for Gpio13 {
    fn capture_pin(gpio_hw: &mut GpioHardware) { gpio_hw.gpio13.take(); }
    fn release_pin(gpio_hw: &mut GpioHardware) { gpio_hw.gpio13.replace( Gpio13::new() ); }
}

impl CaptureGpioPin for Gpio15 {
    fn capture_pin(gpio_hw: &mut GpioHardware) { gpio_hw.gpio15.take(); }
    fn release_pin(gpio_hw: &mut GpioHardware) { gpio_hw.gpio15.replace( Gpio15::new() ); }
}

impl CaptureGpioPin for PhantomPin {
    fn capture_pin(gpio_hw: &mut GpioHardware) {}
    fn release_pin(gpio_hw: &mut GpioHardware) {}
}


pub trait UartGpioPins {
    type TxPin : GpioPin + CaptureGpioPin;
    type RxPin : GpioPin + CaptureGpioPin;
    type CtsPin : GpioPin + CaptureGpioPin;
    type RtsPin : GpioPin + CaptureGpioPin;
}

pub struct Uart0GpioPins;
impl UartGpioPins for Uart0GpioPins {
    type TxPin = Gpio1;
    type RxPin = Gpio3;
    type CtsPin = Gpio13;
    type RtsPin = Gpio15;
}

pub struct Uart0AltGpioPins;
impl UartGpioPins for Uart0AltGpioPins {
    type TxPin = Gpio15;
    type RxPin = Gpio13;
    type CtsPin = PhantomPin;
    type RtsPin = PhantomPin;
}

pub struct Uart1GpioPins;
impl UartGpioPins for Uart1GpioPins {
    type TxPin = Gpio2;
    type RxPin = PhantomPin;
    type CtsPin = PhantomPin;
    type RtsPin = PhantomPin;
}

pub trait UartHaveHardwareFlow {}
pub trait UartCanRead {}
pub trait UartCanWrite {}

pub trait UartHardwareInstance {
    type Pins: UartGpioPins;
    type InitializedType : Uart;

    const UART_PORT_NUM: UartNumber;
}

pub struct Uart0Hardware { guard : () }
impl UartHardwareInstance for Uart0Hardware {
    type Pins = Uart0GpioPins;
    type InitializedType = Uart0;

    const UART_PORT_NUM: UartNumber = UartNumber::Uart0;
}

impl UartCanWrite for Uart0Hardware {}
impl UartCanRead for Uart0Hardware {}
impl UartHaveHardwareFlow for Uart0Hardware {}

impl Uart0Hardware {
    fn new() -> Self { Uart0Hardware { guard: () } }
    fn into_alternative_mode(self) -> Uart0AltHardware { Uart0AltHardware::new() }
}

pub struct Uart0AltHardware { guard : () }
impl UartHardwareInstance for Uart0AltHardware {
    type Pins = Uart0AltGpioPins;
    type InitializedType = Uart0Alt;

    const UART_PORT_NUM: UartNumber = UartNumber::Uart0;
}

impl UartCanWrite for Uart0AltHardware {}
impl UartCanRead for Uart0AltHardware {}

impl Uart0AltHardware {
    fn new() -> Self { Uart0AltHardware { guard: () } }
}

pub struct Uart1Hardware { guard : () }
impl UartHardwareInstance for Uart1Hardware {
    type Pins = Uart1GpioPins;
    type InitializedType = Uart1;

    const UART_PORT_NUM: UartNumber = UartNumber::Uart1;
}

impl UartCanWrite for Uart1Hardware {}

impl Uart1Hardware {
    fn new() -> Self { Uart1Hardware { guard: () } }
    fn into_normal_mode(self) -> Uart0Hardware { Uart0Hardware::new() }
}

#[derive(Eq, PartialEq)]
pub enum UartNumber {
    Uart0,
    Uart1,
}

pub enum UartDataBits {
    B5,
    B6,
    B7,
    B8,
}

pub enum UartParity {
    Disabled,
    Even,
    Odd,
}

pub enum UartStopBits {
    B1,
    B1_5,
    B2,
}

pub enum UartHwControlFlow {
    Disabled,
    Rts,
    Cts,
    CtsRts,
}

/// Represents all available mcu uart ports
pub struct UartHardware {
    pub uart0: Option<Uart0Hardware>,
    pub uart1: Option<Uart1Hardware>,
}


impl UartHardware {
    pub fn new(_peripherals: UartPeripherals) -> Self {
        Self {
            uart0 : Some(Uart0Hardware::new()),
            uart1: Some(Uart1Hardware::new()),
        }
    }
}


impl UartDataBits {
    fn mat_to_ffi(&self) -> uart_word_length_t {
        match self {
            UartDataBits::B5 => uart_word_length_t_UART_DATA_5_BITS,
            UartDataBits::B6 => uart_word_length_t_UART_DATA_6_BITS,
            UartDataBits::B7 => uart_word_length_t_UART_DATA_7_BITS,
            UartDataBits::B8 => uart_word_length_t_UART_DATA_8_BITS,
        }
    }
}

impl UartParity {
    fn map_to_ffi(&self) -> uart_parity_t {
        match self {
            UartParity::Disabled => uart_parity_t_UART_PARITY_DISABLE,
            UartParity::Even => uart_parity_t_UART_PARITY_EVEN,
            UartParity::Odd => uart_parity_t_UART_PARITY_ODD,
        }
    }
}

impl UartStopBits {
    fn map_to_ffi(&self) -> uart_stop_bits_t {
        match self {
            UartStopBits::B1 => uart_stop_bits_t_UART_STOP_BITS_1,
            UartStopBits::B1_5 => uart_stop_bits_t_UART_STOP_BITS_1_5,
            UartStopBits::B2 => uart_stop_bits_t_UART_STOP_BITS_2,
        }
    }
}

impl UartHwControlFlow {
    fn map_to_ffi(&self) -> uart_hw_flowcontrol_t {
        match self {
            UartHwControlFlow::Disabled => uart_hw_flowcontrol_t_UART_HW_FLOWCTRL_DISABLE,
            UartHwControlFlow::Cts => uart_hw_flowcontrol_t_UART_HW_FLOWCTRL_CTS,
            UartHwControlFlow::Rts => uart_hw_flowcontrol_t_UART_HW_FLOWCTRL_RTS,
            UartHwControlFlow::CtsRts => uart_hw_flowcontrol_t_UART_HW_FLOWCTRL_CTS_RTS,
        }
    }
}

impl UartNumber {
    fn map_to_ffi(&self) -> uart_port_t {
        match self {
            UartNumber::Uart0 => uart_port_t_UART_NUM_0,
            UartNumber::Uart1 => uart_port_t_UART_NUM_1,
        }
    }
}

pub struct UartInitializer<UartType : UartHardwareInstance> {
    config: uart_config_t,
    rx_buffer_size: usize,
    tx_buffer_size: usize,
    _data: PhantomData<UartType>,
}

impl<Uart: UartHardwareInstance> UartInitializer<Uart> {
    pub fn new(_uart: Uart) -> Self {
        Self {
            config: uart_config_t {
                baud_rate: 9600,
                data_bits: uart_word_length_t_UART_DATA_8_BITS,
                parity: uart_parity_t_UART_PARITY_DISABLE,
                stop_bits: uart_stop_bits_t_UART_STOP_BITS_1,
                flow_ctrl: uart_hw_flowcontrol_t_UART_HW_FLOWCTRL_DISABLE,
                rx_flow_ctrl_thresh: 0,
            },
            rx_buffer_size: if Uart::UART_PORT_NUM == UartNumber::Uart1 { 0 } else { 256 },
            tx_buffer_size: 0,
            _data: PhantomData
        }
    }

    pub fn set_baud_rate(&mut self, baud_rate: u32) -> Result<&mut Self, UartConfigError> {
        const MIN_BAUD_RATE : u32 = 300;
        const MAX_BAUD_RATE : u32 = 15200 * 40;


        if baud_rate < MIN_BAUD_RATE || baud_rate > MAX_BAUD_RATE {
            Err(UartConfigError::InvalidBaudRate)
        } else {
            self.config.baud_rate = baud_rate as xtensa_int;
            Ok(self)
        }
    }

    pub fn set_data_bits(&mut self, data_bits: UartDataBits) -> Result<&mut Self, UartConfigError> {
        self.config.data_bits = data_bits.mat_to_ffi();
        Ok(self)
    }

    pub fn set_parity(&mut self, parity: UartParity) -> Result<&mut Self, UartConfigError> {
        self.config.parity = parity.map_to_ffi();
        Ok(self)
    }

    pub fn set_stop_bits(&mut self, stop_bits: UartStopBits) -> Result<&mut Self, UartConfigError> {
        self.config.stop_bits = stop_bits.map_to_ffi();
        Ok(self)
    }

    pub fn set_hw_control_flow(&mut self, control_flow: UartHwControlFlow)
        -> Result<&mut Self, UartConfigError> where Uart: UartHaveHardwareFlow
    {
        self.config.flow_ctrl = control_flow.map_to_ffi();
        Ok(self)
    }

    pub fn set_hw_control_flow_rx_threshold(&mut self, threshold: u8)
        -> Result<&mut Self, UartConfigError> where Uart: UartHaveHardwareFlow + UartCanRead
    {
        const MAX_RX_THRESHOLD: u8 = 127;

        if threshold > MAX_RX_THRESHOLD {
            Err(UartConfigError::InvalidRxThreshold)
        } else {
            self.config.rx_flow_ctrl_thresh = threshold;
            Ok(self)
        }
    }

    pub fn set_rx_buffer_size(&mut self, size: usize)
        -> Result<&mut Self, UartConfigError> where Uart: UartCanRead
    {
        const MIN_RX_BUFFER_SIZE: usize = 256;
        if size < MIN_RX_BUFFER_SIZE {
            Err(UartConfigError::InvalidRxBufferSize)
        } else {
            self.rx_buffer_size = size;
            Ok(self)
        }
    }

    pub fn set_tx_buffer_size(&mut self, size: usize) -> Result<&mut Self, UartConfigError> {
        const MIN_TX_BUFFER_SIZE: usize = 256;
        if size < MIN_TX_BUFFER_SIZE && size != 0 {
            Err(UartConfigError::InvalidTxBufferSize)
        } else {
            self.tx_buffer_size = size;
            Ok(self)
        }
    }

    // TODO: capture pins
    pub fn initialize(mut self, gpio_hw: &mut GpioHardware)
        -> Result<Uart::InitializedType, UartConfigError>
    {
        unsafe {
            let uart_num = Uart::UART_PORT_NUM.map_to_ffi();

            if (uart_param_config(uart_num, &mut self.config) != esp_err_t_ESP_OK) {
                return Err(UartConfigError::Unknown);
            }

            if uart_driver_install(
                uart_num,
                self.rx_buffer_size as isize,
                self.tx_buffer_size as isize,
                0,
                null_mut()
            ) != esp_err_t_ESP_OK {
                return Err(UartConfigError::Unknown);
            }

            <<Uart as UartHardwareInstance>::Pins as UartGpioPins>::TxPin::capture_pin(gpio_hw);
            <<Uart as UartHardwareInstance>::Pins as UartGpioPins>::RxPin::capture_pin(gpio_hw);
            <<Uart as UartHardwareInstance>::Pins as UartGpioPins>::CtsPin::capture_pin(gpio_hw);
            <<Uart as UartHardwareInstance>::Pins as UartGpioPins>::RtsPin::capture_pin(gpio_hw);

            return Ok(Uart::InitializedType::build(UartInitializedMarker::new()));

        }
    }
}


pub struct UartInitializedMarker { guard: () }

impl UartInitializedMarker {
    fn new() -> Self { UartInitializedMarker { guard: () }}
}

pub trait Uart {
    type Hardware : UartHardwareInstance;

    fn build(_: UartInitializedMarker) -> Self;
}


pub struct Uart0 { guard: () }
impl Uart0 {
    fn new() -> Self { Uart0 { guard: () }}
}

pub struct Uart0Alt { guard: () }
impl Uart0Alt {
    fn new() -> Self { Uart0Alt { guard: () }}
}

pub struct Uart1 { guard: () }
impl Uart1 {
    fn new() -> Self { Uart1 { guard: () }}
}

impl Uart for Uart0 {
    type Hardware = Uart0Hardware;

    fn build(_: UartInitializedMarker) -> Self { Self::new() }
}
impl Uart for Uart0Alt {
    type Hardware = Uart0AltHardware;

    fn build(_: UartInitializedMarker) -> Self { Self::new() }
}
impl Uart for Uart1 {
    type Hardware = Uart1Hardware;

    fn build(_: UartInitializedMarker) -> Self { Self::new() }
}

pub enum WaitError {
    Timeout,
}

pub enum ReadError {
    Timeout,
}

pub trait TransmittingUart {
    fn write_bytes(&mut self, data: &[u8]) -> usize;
    fn wait_write_done(&mut self, ticks: usize) -> Result<(), WaitError>;
}

impl<T : Uart> TransmittingUart for T where <T as Uart>::Hardware: UartCanWrite {
    fn write_bytes(&mut self, data: &[u8]) -> usize {
        let uart_num = T::Hardware::UART_PORT_NUM.map_to_ffi();
        unsafe { uart_write_bytes(uart_num, data.as_ptr(), data.len()) as usize }
    }

    fn wait_write_done(&mut self, timeout: usize) -> Result<(), WaitError> {
        let uart_num = T::Hardware::UART_PORT_NUM.map_to_ffi();
        unsafe {
            if uart_wait_tx_done(uart_num, timeout) == esp_err_t_ESP_OK {
                Ok(())
            } else {
                Err(WaitError::Timeout)
            }
        }
    }
}

pub trait ReceivingUart {
    fn read_bytes(&mut self, buffer: &mut[u8], timeout: usize) -> Result<usize, ReadError>;
}

impl<T: Uart> ReceivingUart for T where <T as Uart>::Hardware: UartCanRead {
    fn read_bytes(&mut self, buffer: &mut[u8], timeout: usize) -> Result<usize, ReadError> {
        let uart_num = T::Hardware::UART_PORT_NUM.map_to_ffi();
        unsafe {
            let written_bytes =
                uart_read_bytes(uart_num, buffer.as_mut_ptr(), buffer.len() as u32, timeout);

            if (written_bytes < 0) {
                Err(ReadError::Timeout)
            } else {
                Ok(written_bytes as usize)
            }
        }
    }
}