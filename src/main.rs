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
    179, 191, 18, 125, 114, 171, 181, 58, 71, 241, 14, 101, 93, 155, 45, 208,
    179, 241, 73, 89, 229, 21, 228, 200, 114, 57, 136, 3, 78, 206, 137, 126,
    108, 197, 243, 99, 41, 57, 185, 184, 72, 31, 219, 192, 214, 70, 135, 192,
    75, 217, 197, 60, 36, 140, 184, 48, 52, 22, 186, 79, 8, 221, 147, 134, 155,
    115, 120, 148, 169, 240, 21, 138, 187, 41, 250, 100, 97, 255, 173, 175, 4,
    134, 60, 73, 20, 136, 5, 173, 195, 137, 223, 23, 239, 200, 124, 237, 58,
    176, 67, 46, 209, 195, 191, 43, 8, 81, 94, 205, 232, 218, 209, 73, 37, 46,
    227, 250, 29, 207, 122, 172, 201, 232, 21, 72, 37, 4, 193, 79, 22, 87, 108,
    14, 125, 72, 195, 220, 80, 223, 161, 142, 233, 172, 104, 183, 55, 59, 205,
    222, 101, 45, 219, 211, 110, 213, 153, 189, 234, 91, 223, 70, 196, 24, 126,
    176, 184, 169, 80, 98, 34, 129, 136, 12, 4, 193, 235, 222, 195, 131, 12,
    251, 92, 41, 92, 2, 105, 59, 103, 54, 8, 61, 87, 79, 143, 54, 154, 33, 114,
    169, 182, 205, 143, 199, 212, 196, 145, 213, 102, 185, 208, 22, 83, 22,
    176, 67, 252, 141, 68, 124, 88, 62, 131, 138, 105, 149, 60, 0, 22, 153,
    100, 115, 249, 190, 55, 33, 140, 237, 232, 246, 249, 181, 205, 54, 62, 223,
    186, 210, 21, 154, 235, 60, 34, 59, 222, 219, 190, 217, 124, 77, 64, 52,
    102, 11, 159, 98, 26, 195, 134, 168, 60, 28, 119, 27, 6, 253, 70, 228, 214,
    30, 75, 39, 67, 255, 76, 3, 192, 147, 39, 136, 88, 113, 140, 128, 24, 124,
    142, 61, 202, 122, 230, 128, 12, 243, 137, 90, 176, 227, 221, 184, 81, 86,
    172, 179, 47, 255, 106, 24, 202, 73, 229, 222, 53, 201, 136, 141, 152, 198,
    33, 111, 157, 221, 247, 231, 208, 75, 208, 25, 115, 228, 185, 221, 231, 42,
    155, 59, 107, 242, 224, 218, 4, 127, 28, 23, 244, 92, 31, 48, 183, 196, 86,
    41, 28, 182, 73, 215, 128, 169, 179, 123, 15, 132, 89, 147, 113, 127, 50,
    125, 126, 95, 70, 255, 175, 118, 229, 34, 126, 31, 1, 12, 94, 244, 94, 198,
    202, 83, 237, 15, 11, 218, 9, 243, 161, 15, 2, 35, 7, 172, 36, 84, 194,
    232, 18, 213, 224, 163, 10, 201, 250, 4, 128, 0, 55, 1, 60, 0, 80, 193, 30,
    166, 111, 230, 216, 21, 13, 128, 87, 215, 130, 35, 248, 54, 186, 15, 228,
    172, 207, 237, 244, 250, 236, 106, 49, 3, 75, 61, 58, 77, 255, 184, 50,
    232, 58, 254, 122, 46, 189, 175, 163, 164, 224, 229, 44, 174, 155, 244, 69,
    89, 13, 91, 131, 128, 170, 201, 91, 210, 147, 105, 126, 154, 183, 92, 180,
    202, 190, 253, 70, 222, 211, 232, 209, 182, 9, 82, 24, 111, 123, 26, 77,
    20, 137, 16, 128, 175,
  ]);
  let d = crypto_bigint::U4096::from_be_bytes([
    34, 139, 63, 15, 114, 225, 67, 3, 255, 93, 121, 105, 203, 178, 141, 252,
    133, 8, 131, 19, 78, 174, 133, 120, 108, 83, 88, 43, 98, 146, 216, 227,
    190, 29, 208, 231, 166, 189, 156, 78, 169, 53, 206, 50, 226, 59, 77, 205,
    140, 6, 63, 28, 142, 221, 168, 108, 67, 38, 119, 9, 199, 103, 101, 249,
    193, 152, 80, 125, 41, 167, 165, 76, 51, 42, 31, 31, 249, 161, 124, 140,
    157, 46, 251, 25, 4, 100, 27, 203, 72, 64, 15, 234, 246, 191, 46, 27, 29,
    99, 80, 150, 7, 228, 57, 178, 24, 120, 34, 227, 41, 180, 27, 242, 149, 189,
    204, 60, 126, 76, 40, 132, 90, 141, 74, 193, 193, 179, 135, 63, 30, 201,
    16, 80, 60, 141, 166, 110, 137, 240, 96, 137, 41, 169, 99, 186, 138, 87,
    232, 249, 171, 178, 67, 131, 255, 209, 247, 41, 3, 136, 66, 129, 196, 31,
    253, 1, 19, 140, 30, 145, 173, 149, 253, 66, 106, 11, 166, 187, 250, 17,
    14, 134, 164, 48, 162, 169, 39, 246, 45, 160, 185, 182, 168, 55, 247, 11,
    4, 86, 23, 70, 60, 134, 134, 242, 93, 150, 165, 124, 64, 176, 199, 39, 243,
    49, 242, 16, 46, 210, 43, 110, 70, 59, 69, 102, 109, 98, 43, 212, 204, 131,
    16, 70, 37, 162, 3, 208, 99, 216, 57, 36, 117, 219, 21, 164, 46, 51, 43,
    33, 66, 219, 178, 7, 173, 128, 154, 20, 219, 57, 120, 100, 182, 149, 31,
    143, 135, 243, 39, 138, 41, 211, 47, 72, 47, 79, 255, 39, 114, 242, 189,
    98, 245, 124, 119, 199, 135, 67, 118, 240, 192, 5, 176, 65, 146, 183, 93,
    243, 46, 30, 251, 145, 53, 85, 85, 205, 141, 112, 99, 242, 54, 23, 57, 237,
    190, 192, 207, 236, 244, 106, 236, 65, 232, 251, 131, 107, 17, 77, 197, 76,
    165, 210, 129, 29, 205, 136, 251, 119, 157, 42, 175, 137, 23, 141, 132,
    123, 195, 134, 199, 209, 31, 170, 243, 151, 250, 243, 191, 127, 108, 161,
    219, 110, 80, 126, 65, 199, 15, 84, 205, 148, 186, 130, 6, 127, 9, 208, 28,
    31, 16, 155, 198, 91, 25, 130, 153, 139, 25, 52, 159, 133, 227, 155, 223,
    49, 125, 177, 126, 200, 221, 51, 225, 72, 240, 61, 177, 137, 165, 80, 74,
    65, 129, 61, 152, 169, 51, 38, 48, 44, 213, 250, 48, 114, 47, 189, 81, 182,
    220, 236, 239, 95, 121, 83, 90, 56, 133, 164, 240, 23, 49, 119, 160, 139,
    200, 6, 228, 29, 204, 246, 172, 89, 51, 141, 151, 201, 114, 249, 194, 198,
    157, 223, 192, 160, 50, 97, 198, 66, 90, 153, 154, 69, 3, 250, 127, 207,
    162, 160, 7, 164, 53, 42, 109, 175, 228, 196, 207, 159, 113, 109, 217, 119,
    76, 64, 64, 186, 220, 38, 159, 115, 191, 98, 80, 235, 68, 241, 72, 163,
    212, 59, 205, 219, 139, 218, 139, 205, 251, 121,
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
fn panic(info: &PanicInfo) -> ! {
  interrupt::free(|_| unsafe {
    if let Some(mut uart) = STDOUT.as_mut() {
      match info.location() {
        Some(location) => {
          let f = location.file();
          let l = location.line();
          uart.write(f.len() as u8 + 1);
          for c in f.chars() {
            uart.write(c as u8);
          }
          uart.write(l as u8);
        }
        None => {
          uart.write(b'P');
          return;
        }
      }
    }
  });
  loop {}
}
