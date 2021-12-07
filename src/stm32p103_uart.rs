// Refer https://datasheet.octopart.com/STM32-P103-Olimex-datasheet-41569145.pdf
use volatile_register::RW;

#[repr(C)]
pub struct UART {
  /// Control register 1 (offset: 0x00)
  cr1: RW<u32>,
  /// Control register 2 (offset: 0x04)
  cr2: RW<u32>,
  /// Control register 3 (offset: 0x08)
  cr3: RW<u32>,
  /// Baud rate register (offset: 0x0C)
  brr: RW<u32>,
  /// Guard time and prescaler register (offset: 0x10)
  gtpr: RW<u32>,
  /// Receiver timeout register (offset: 0x14)
  rtor: RW<u32>,
  /// Request register (offset: 0x18)
  rqr: RW<u32>,
  /// Interrupt & status register (offset: 0x1C)
  isr: RW<u32>,
  /// Interrupt flag clear register (offset: 0x20)
  icr: RW<u32>,
  /// Receive data register (offset: 0x24)
  rdr: RW<u32>,
  /// Transmit data register (offset: 0x28)
  tdr: RW<u32>,
}

pub unsafe fn init() -> &'static mut UART {
  let uart = &mut *(0x4001_3800 as usize as *mut UART);
  uart
}

// XXX Implement
impl UART {
  pub unsafe fn read_byte(&mut self) -> u8 {
    self.rdr.read() as u8
  }

  pub unsafe fn read(&mut self, buf: &mut [u8]) {
    for b in buf.iter_mut() {
        *b = self.rdr.read() as u8;
      }
  }

  pub unsafe fn write(&mut self, byte: u8) {
    self.tdr.write(byte.into());
  }
}
