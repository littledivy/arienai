use volatile_register::RO;
use volatile_register::RW;
use volatile_register::WO;

#[repr(C)]
pub struct UART {
  /// Data register
  dr: RW<u32>,
  /// Receive status register
  rsr: RW<u32>,
  /// A reserved region with no explicit use
  reserved1: [u8; 16],
  /// Flag register
  fr: RO<u32>,
  /// A reserved region with no explicit use
  reserved2: [u8; 4],
  /// UART IrDA low-power register
  ilpr: RW<u32>,
  /// Integer baud rate divisor register
  ibrd: RW<u32>,
  /// Fractional baud rate divisor register
  fbrd: RW<u32>,
  /// UART line control
  lcrh: RW<u32>,
  /// Control register
  ctl: RW<u32>,
  /// Interrupt FIFO level select register
  ifls: RW<u32>,
  /// Interrupt mask set/clear register
  im: RW<u32>,
  /// Raw interrupt status register
  ris: RO<u32>,
  /// Masked interrupt status register
  mis: RO<u32>,
  /// Interrupt clear register
  icr: WO<u32>,
  /// UART DMA control
  dmactl: RW<u32>,
}

pub unsafe fn init() -> &'static mut UART {
  let uart = &mut *(0x4000C000 as usize as *mut UART);
  uart.ctl.write(uart.ctl.read() & 0xffff_fffe);
  uart.ibrd.write((uart.ibrd.read() & 0xffff_0000) | 0x000a);
  uart.fbrd.write((uart.fbrd.read() & 0xffff_0000) | 0x0036);
  uart.lcrh.write(0x60);
  uart.ctl.write(uart.ctl.read() | 0x01);

  uart
}

impl UART {
  pub unsafe fn read_byte(&mut self) -> u8 {
    self.dr.read() as u8
  }

  pub unsafe fn read(&mut self, buf: &mut [u8]) {
    for b in buf.iter_mut() {
      while !(self.fr.read() & 0x10 == 0) {}
      if self.fr.read() & 0x10 == 0 {
        *b = self.dr.read() as u8;
      }
    }
  }

  pub unsafe fn write(&mut self, byte: u8) {
    self.dr.write(byte.into());
  }
}
