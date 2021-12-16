use gd32vf103xx_hal::gpio::gpioa::PA10;
use gd32vf103xx_hal::gpio::gpioa::PA9;
use gd32vf103xx_hal::gpio::Alternate;
use gd32vf103xx_hal::gpio::Floating;
use gd32vf103xx_hal::gpio::Input;
use gd32vf103xx_hal::gpio::PushPull;
use gd32vf103xx_hal::serial::Serial;
use gd32vf103xx_hal::{
  pac::USART0,
  serial::{self, Config, Parity, Rx, StopBits, Tx},
};
use longan_nano::hal::{pac, prelude::*};
use nb::block;

pub struct UART {
  pub tx: Tx<USART0>,
  pub rx: Rx<USART0>,
}

pub fn init() -> UART {
  let dp = pac::Peripherals::take().unwrap();
  // Configure clocks
  let mut rcu = dp
    .RCU
    .configure()
    .ext_hf_clock(8.mhz())
    .sysclk(108.mhz())
    .freeze();

  let mut afio = dp.AFIO.constrain(&mut rcu);

  let gpioa = dp.GPIOA.split(&mut rcu);

  let tx = gpioa.pa9.into_alternate_push_pull();
  let rx = gpioa.pa10.into_floating_input();

  let config = Config {
    baudrate: 115_200.bps(),
    parity: Parity::ParityNone,
    stopbits: StopBits::STOP1,
  };

  let serial =
    serial::Serial::new(dp.USART0, (tx, rx), config, &mut afio, &mut rcu);

  let (tx, rx) = serial.split();
  UART { tx, rx }
}

impl UART {
  pub unsafe fn read_byte(&mut self) -> Option<u8> {
    self.rx.read().ok()
  }

  pub unsafe fn read(&mut self, buf: &mut [u8]) {
    for byte in buf {
      *byte = block!(self.rx.read()).unwrap();
    }
  }

  pub unsafe fn write(&mut self, byte: u8) {
    block!(self.tx.write(byte));
  }
}
