use crate::gpio::*;
use idf_sys::uart::*;
use core::marker::PhantomData;

pub trait UartGpioPins {
    type TxPin : GpioPin;
    type RxPin : GpioPin;
    type DtrPin : GpioPin;
    type CtsPin : GpioPin;
    type DsrPin : GpioPin;
    type RtsPin : GpioPin;
}

struct Uart0GpioPins;
impl UartGpioPins for Uart0GpioPins {
    type TxPin = Gpio1;
    type RxPin = Gpio3;
    type DtrPin = Gpio12;
    type CtsPin = Gpio13;
    type DsrPin = Gpio14;
    type RtsPin = Gpio15;
}

struct Uart0AltGpioPins;
impl UartGpioPins for Uart0AltGpioPins {
    type TxPin = Gpio15;
    type RxPin = Gpio13;
    type DtrPin = Gpio12;
    type CtsPin = PhantomPin;
    type DsrPin = Gpio14;
    type RtsPin = PhantomPin;
}

struct Uart1GpioPins;
impl UartGpioPins for Uart1GpioPins {
    type TxPin = Gpio2;
    type RxPin = PhantomPin;
    type DtrPin = PhantomPin;
    type CtsPin = PhantomPin;
    type DsrPin = PhantomPin;
    type RtsPin = PhantomPin;
}

pub trait Uart {
    type Pins: UartGpioPins;
    const UART_PORT_NUM: uart_port_t;
}

struct Uart0 { pub(crate) _data: PhantomData<()> }
impl Uart for Uart0 {
    type Pins = Uart0GpioPins;
    const UART_PORT_NUM: uart_port_t = uart_port_t_UART_NUM_0;
}

struct Uart0Alt { pub(crate) _data: PhantomData<()> }
impl Uart for Uart0Alt {
    type Pins = Uart0AltGpioPins;
    const UART_PORT_NUM: uart_port_t = uart_port_t_UART_NUM_0;
}

impl Uart0 {
    fn to_alternative_mode(self) -> Uart0Alt {
        Uart0Alt { _data: PhantomData }
    }
}

struct Uart1 { pub(crate) _data: PhantomData<()> }
impl Uart for Uart1 {
    type Pins = Uart1GpioPins;
    const UART_PORT_NUM: uart_port_t = uart_port_t_UART_NUM_1;
}

/// Represents all available mcu uart ports
struct UartHardware {
    pub uart0: Option<Uart0>,
    pub uart1: Option<Uart1>,
}

struct UartConfigurator<UartType : Uart> {
    uart: UartType
}