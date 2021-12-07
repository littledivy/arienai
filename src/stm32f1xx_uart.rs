use stm32f1xx_hal::pac;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::serial::Config;
use stm32f1xx_hal::serial::Pins;
use stm32f1xx_hal::serial::Serial;

use stm32f1xx_hal::gpio::gpiob::PB10;
use stm32f1xx_hal::gpio::gpiob::PB11;
use stm32f1xx_hal::gpio::Alternate;
use stm32f1xx_hal::gpio::Input;
use stm32f1xx_hal::gpio::PushPull;
use stm32f1xx_hal::gpio::Floating;

pub struct UART {
  serial: Serial<pac::USART3, (PB10<Alternate<PushPull>>, PB11<Input<Floating>>)>,
}

pub fn init() -> UART {
  let p = pac::Peripherals::take().unwrap();
  let mut flash = p.FLASH.constrain();
  let rcc = p.RCC.constrain();
  let clocks = rcc.cfgr.freeze(&mut flash.acr);
  let mut afio = p.AFIO.constrain();
  let mut gpiob = p.GPIOB.split();

  let tx = gpiob.pb10.into_alternate_push_pull(&mut gpiob.crh);
  let rx = gpiob.pb11;

  let mut serial = Serial::usart3(
    p.USART3,
    (tx, rx),
    &mut afio.mapr,
    Config::default().baudrate(9600.bps()),
    clocks,
  );

  UART { serial }
}

impl UART {
  pub unsafe fn read_byte(&mut self) -> u8 {
    self.serial.read().unwrap()
  }

  pub unsafe fn read(&mut self, buf: &mut [u8]) {
    
  }

  pub unsafe fn write(&mut self, byte: u8) {
    self.serial.write(byte);
  }
}
