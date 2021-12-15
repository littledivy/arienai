#![feature(alloc_error_handler)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc;

mod heap;
mod msg;
mod rsa;
mod uart;

use rand::Rng;
use rand_core::SeedableRng;
use rand_hc::Hc128Rng;
use riscv::asm;
use riscv::interrupt;
use riscv_rt::entry;

use core::alloc::Layout;
use core::convert::TryFrom;
use core::panic::PanicInfo;

use msg::Message;

use crypto_bigint::Encoding;
use embedded_graphics::image::Image;
use embedded_graphics::image::ImageRaw;
use embedded_graphics::mono_font::ascii::FONT_7X14;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::raw::LittleEndian;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use gd32vf103xx_hal::gpio::Alternate;
use gd32vf103xx_hal::gpio::Floating;
use gd32vf103xx_hal::gpio::Input;
use gd32vf103xx_hal::gpio::PushPull;
use gd32vf103xx_hal::serial::Serial;
use gd32vf103xx_hal::serial::{self, Config, Parity, Rx, StopBits, Tx};
use longan_nano::hal::{pac, prelude::*};
use longan_nano::{lcd, lcd_pins};

const ARWEAVE_LOGO: &[u8] = include_bytes!("verto.raw");

static mut STDOUT: Option<uart::UART> = None;

#[entry]
fn main() -> ! {
  heap::init();
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

  let gpiob = dp.GPIOB.split(&mut rcu);

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

  // XXX: Replace with your private key.

  let n = crypto_bigint::U4096::from_be_bytes([
    166, 230, 69, 172, 111, 80, 151, 174, 220, 180, 194, 167, 201, 216, 120,
    59, 82, 61, 30, 114, 171, 145, 243, 141, 166, 220, 180, 154, 232, 22, 133,
    247, 171, 49, 91, 118, 225, 143, 56, 41, 219, 67, 87, 141, 9, 210, 82, 47,
    45, 170, 246, 224, 180, 55, 207, 126, 255, 44, 167, 41, 134, 190, 132, 174,
    35, 146, 51, 215, 162, 101, 12, 91, 80, 223, 139, 85, 212, 41, 187, 13, 8,
    214, 67, 225, 209, 46, 196, 187, 100, 147, 20, 48, 229, 73, 217, 242, 209,
    224, 88, 137, 1, 104, 60, 53, 135, 200, 113, 15, 74, 134, 156, 155, 51, 73,
    165, 162, 66, 53, 53, 246, 174, 80, 201, 240, 43, 182, 224, 149, 161, 169,
    234, 127, 57, 215, 98, 121, 75, 147, 17, 81, 123, 222, 11, 25, 11, 3, 85,
    23, 156, 80, 12, 101, 171, 122, 143, 115, 248, 116, 192, 173, 42, 128, 136,
    155, 38, 20, 128, 94, 38, 222, 124, 212, 113, 228, 40, 199, 121, 181, 219,
    82, 108, 228, 69, 152, 40, 173, 249, 111, 153, 243, 29, 97, 249, 126, 177,
    207, 247, 151, 234, 9, 79, 209, 67, 247, 199, 157, 26, 92, 11, 215, 167,
    111, 63, 49, 22, 25, 106, 7, 129, 39, 164, 127, 220, 65, 120, 176, 30, 255,
    52, 157, 210, 150, 12, 177, 198, 129, 102, 242, 44, 172, 38, 111, 79, 134,
    25, 230, 160, 184, 98, 106, 116, 131, 92, 109, 5, 89, 36, 104, 25, 153, 69,
    38, 36, 180, 144, 184, 77, 216, 70, 206, 85, 108, 211, 26, 119, 212, 120,
    66, 53, 233, 196, 255, 213, 192, 209, 56, 100, 35, 99, 1, 152, 20, 183,
    206, 184, 185, 68, 6, 2, 230, 243, 158, 151, 216, 158, 145, 130, 140, 16,
    4, 62, 42, 200, 108, 72, 157, 67, 82, 3, 249, 80, 34, 72, 249, 12, 24, 61,
    85, 68, 255, 171, 109, 245, 61, 180, 247, 79, 100, 36, 117, 0, 46, 156,
    207, 83, 210, 231, 240, 147, 119, 68, 48, 37, 216, 181, 113, 235, 17, 139,
    192, 217, 218, 127, 116, 148, 25, 199, 62, 225, 166, 135, 54, 142, 36, 18,
    210, 123, 173, 12, 114, 91, 114, 225, 125, 48, 242, 95, 238, 245, 137, 88,
    242, 49, 131, 57, 62, 242, 137, 66, 132, 15, 22, 25, 93, 117, 201, 154,
    142, 164, 250, 173, 249, 101, 113, 12, 116, 80, 18, 139, 101, 191, 187,
    255, 190, 152, 130, 32, 220, 235, 107, 142, 206, 241, 87, 228, 199, 84, 32,
    175, 6, 99, 129, 241, 110, 32, 130, 223, 25, 153, 132, 18, 176, 63, 7, 203,
    3, 45, 242, 96, 116, 186, 66, 10, 232, 172, 109, 220, 202, 73, 252, 1, 52,
    40, 212, 170, 128, 192, 130, 0, 63, 35, 37, 27, 155, 243, 234, 191, 122,
    85, 55, 42, 26, 26, 9, 124, 109, 184, 55, 243, 77, 61, 182, 212, 229, 48,
    193, 237, 65, 136, 111, 106, 193,
  ]);
  let d = crypto_bigint::U4096::from_be_bytes([
    166, 230, 69, 172, 111, 80, 151, 174, 220, 180, 194, 167, 201, 216, 120,
    59, 82, 61, 30, 114, 171, 145, 243, 141, 166, 220, 180, 154, 232, 22, 133,
    247, 171, 49, 91, 118, 225, 143, 56, 41, 219, 67, 87, 141, 9, 210, 82, 47,
    45, 170, 246, 224, 180, 55, 207, 126, 255, 44, 167, 41, 134, 190, 132, 174,
    35, 146, 51, 215, 162, 101, 12, 91, 80, 223, 139, 85, 212, 41, 187, 13, 8,
    214, 67, 225, 209, 46, 196, 187, 100, 147, 20, 48, 229, 73, 217, 242, 209,
    224, 88, 137, 1, 104, 60, 53, 135, 200, 113, 15, 74, 134, 156, 155, 51, 73,
    165, 162, 66, 53, 53, 246, 174, 80, 201, 240, 43, 182, 224, 149, 161, 169,
    234, 127, 57, 215, 98, 121, 75, 147, 17, 81, 123, 222, 11, 25, 11, 3, 85,
    23, 156, 80, 12, 101, 171, 122, 143, 115, 248, 116, 192, 173, 42, 128, 136,
    155, 38, 20, 128, 94, 38, 222, 124, 212, 113, 228, 40, 199, 121, 181, 219,
    82, 108, 228, 69, 152, 40, 173, 249, 111, 153, 243, 29, 97, 249, 126, 177,
    207, 247, 151, 234, 9, 79, 209, 67, 247, 199, 157, 26, 92, 11, 215, 167,
    111, 63, 49, 22, 25, 106, 7, 129, 39, 164, 127, 220, 65, 120, 176, 30, 255,
    52, 157, 210, 150, 12, 177, 198, 129, 102, 242, 44, 172, 38, 111, 79, 134,
    25, 230, 160, 184, 98, 106, 116, 131, 92, 109, 5, 89, 36, 104, 25, 153, 69,
    38, 36, 180, 144, 184, 77, 216, 70, 206, 85, 108, 211, 26, 119, 212, 120,
    66, 53, 233, 196, 255, 213, 192, 209, 56, 100, 35, 99, 1, 152, 20, 183,
    206, 184, 185, 68, 6, 2, 230, 243, 158, 151, 216, 158, 145, 130, 140, 16,
    4, 62, 42, 200, 108, 72, 157, 67, 82, 3, 249, 80, 34, 72, 249, 12, 24, 61,
    85, 68, 255, 171, 109, 245, 61, 180, 247, 79, 100, 36, 117, 0, 46, 156,
    207, 83, 210, 231, 240, 147, 119, 68, 48, 37, 216, 181, 113, 235, 17, 139,
    192, 217, 218, 127, 116, 148, 25, 199, 62, 225, 166, 135, 54, 142, 36, 18,
    210, 123, 173, 12, 114, 91, 114, 225, 125, 48, 242, 95, 238, 245, 137, 88,
    242, 49, 131, 57, 62, 242, 137, 66, 132, 15, 22, 25, 93, 117, 201, 154,
    142, 164, 250, 173, 249, 101, 113, 12, 116, 80, 18, 139, 101, 191, 187,
    255, 190, 152, 130, 32, 220, 235, 107, 142, 206, 241, 87, 228, 199, 84, 32,
    175, 6, 99, 129, 241, 110, 32, 130, 223, 25, 153, 132, 18, 176, 63, 7, 203,
    3, 45, 242, 96, 116, 186, 66, 10, 232, 172, 109, 220, 202, 73, 252, 1, 52,
    40, 212, 170, 128, 192, 130, 0, 63, 35, 37, 27, 155, 243, 234, 191, 122,
    85, 55, 42, 26, 26, 9, 124, 109, 184, 55, 243, 77, 61, 182, 212, 229, 48,
    193, 237, 65, 136, 111, 106, 193,
  ]);

  let lcd_pins = lcd_pins!(gpioa, gpiob);
  let mut lcd = lcd::configure(dp.SPI0, lcd_pins, &mut afio, &mut rcu);
  let (width, height) = (lcd.size().width as i32, lcd.size().height as i32);

  // Clear screen
  Rectangle::new(Point::new(0, 0), Size::new(width as u32, height as u32))
    .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
    .draw(&mut lcd)
    .unwrap();

  let raw_image: ImageRaw<Rgb565, LittleEndian> =
    ImageRaw::new(&ARWEAVE_LOGO, 50);

  Image::new(&raw_image, Point::new(5, height / 2 - 32))
    .draw(&mut lcd)
    .unwrap();
  let style = MonoTextStyleBuilder::new()
    .font(&FONT_7X14)
    .text_color(Rgb565::BLACK)
    .background_color(Rgb565::GREEN)
    .build();

  interrupt::free(|_| unsafe {
    STDOUT.replace(uart::UART { tx, rx });
  });

  unsafe {
    loop {
      if let Some(uart) = STDOUT.as_mut() {
        if let Some(byte) = uart.read_byte() {
          match Message::try_from(byte) {
            Ok(Message::Sign) => {
              // Text::new("Recv", Point::new(40, 35), style)
              // .draw(&mut lcd)
              // .unwrap();
              let mut digest = [0u8; 256 / 8];
              uart.read(&mut digest);

              let mut rng = Hc128Rng::from_seed([0; 32]);

              let mut salt = [0u8; 32];
              rng.fill(&mut salt[..]);
              Text::new("Signing", Point::new(40, 35), style)
                .draw(&mut lcd)
                .unwrap();
              // 256 bytes
              match rsa::sign_pss_with_salt(
                &digest,
                &salt,
                &d.to_uint_array(),
                &n,
              ) {
                Ok(signature) => {
                  Text::new("Sending", Point::new(40, 35), style)
                    .draw(&mut lcd)
                    .unwrap();

                  for b in signature {
                    uart.write(b);
                  }
                }
                Err(e) => {
                  Text::new("Error", Point::new(40, 35), style)
                    .draw(&mut lcd)
                    .unwrap();
                  uart.write(b'E');
                }
              }
            }
            Ok(Message::Verify) => {
              // let mut digest = [0u8; 256 / 8];
              // uart.read(&mut digest);

              // let mut signature = [0u8; 256];
              // uart.read(&mut signature);

              // let rng = Hc128Rng::from_seed([0; 32]);
              // let padding = PaddingScheme::new_pss_with_salt::<Sha256, _>(rng, 32);

              // let verification =
              //   signing_key.verify(padding, &digest, &signature).is_ok();

              // uart.write(if verification { 1 } else { 0 });
            }
            Ok(Message::GetAddress) => {}
            Ok(Message::GetOwner) => {}
            Err(_) => {}
          }
        }
      }
    }
  }
}

#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
  interrupt::free(|_| unsafe {
    if let Some(mut uart) = STDOUT.as_mut() {
      uart.write(b'S');
    }
  });
  loop {}
}

#[inline(never)]
pub unsafe extern "C" fn __read32(_default: usize, addr: usize) -> u32 {
  let ptr = addr as *const u32;
  ptr.read_volatile()
}

#[export_name = "trap_handler"]
fn trap_handler() {
  use riscv::register::{
    mcause,
    mcause::{Exception, Trap},
    mepc, mtval,
  };
  let ld_insn_addr = __read32 as *const () as usize;

  let mcause = mcause::read();
  let mepc = mepc::read();

  if mepc == ld_insn_addr
    && mcause.cause() == Trap::Exception(Exception::LoadFault)
  {
    mepc::write(mepc + 2);
    return;
  }

  interrupt::free(|_| unsafe {
    if let Some(mut uart) = STDOUT.as_mut() {
      uart.write(b'T');
    }
  });

  loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  interrupt::free(|_| unsafe {
    if let Some(mut uart) = STDOUT.as_mut() {
      uart.write(b'P');
    }
  });
  loop {}
}
