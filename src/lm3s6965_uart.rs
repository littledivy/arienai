use core::mem::MaybeUninit;
use volatile_register::RO;
use volatile_register::RW;
use volatile_register::WO;

// Refer https://www.ti.com/lit/ds/symlink/lm3s6965.pdf
#[repr(C)]
pub struct UART {
  /// UART Data (offset: 0x000)
  dr: RW<u32>,
  /// UART recieve status (offset: 0x004)
  rsr: RW<u32>,
  reserved1: MaybeUninit<[u8; 16]>,
  /// UART flag (offset: 0x018)
  fr: RO<u32>,
  reserved2: MaybeUninit<[u8; 4]>,
  /// UART IrDA low-power register
  ilpr: RW<u32>,
  /// Integer baud-rate divisor register
  ibrd: RW<u32>,
  /// Fractional baud-rate divisor register
  fbrd: RW<u32>,
  /// UART line control
  lcrh: RW<u32>,
  /// UART Control
  ctl: RW<u32>,
  /// UART Interrupt FIFO level select
  ifls: RW<u32>,
  /// UART Interrupt mask
  im: RW<u32>,
  /// UART Raw interrupt status
  ris: RO<u32>,
  /// UART Masked interrupt status
  mis: RO<u32>,
  /// UART Interrupt clear
  icr: WO<u32>,
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
