#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate alloc;

use panic_itm as _;

use alloc_cortex_m::CortexMHeap;
use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use rand_core::SeedableRng;
use rand_hc::Hc128Rng;
use volatile_register::RO;
use volatile_register::RW;
use volatile_register::WO;

use alloc::vec;
use core::alloc::Layout;

use rsa::pkcs1::FromRsaPrivateKey;
use rsa::PaddingScheme;
use rsa::PublicKey;
use rsa::RsaPrivateKey;
use sha2::Sha256;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

// 64 Kib
const HEAP_SIZE: usize = 1024 * 64;

// TODO: Avoid runtime decoding using comtime macros.
// XXX: Replace with your private key.
const RSA_PRIVATE_KEY: &str = r"-----BEGIN RSA PRIVATE KEY-----
MIIEogIBAAKCAQEAnzyis1ZjfNB0bBgKFMSvvkTtwlvBsaJq7S5wA+kzeVOVpVWw
kWdVha4s38XM/pa/yr47av7+z3VTmvDRyAHcaT92whREFpLv9cj5lTeJSibyr/Mr
m/YtjCZVWgaOYIhwrXwKLqPr/11inWsAkfIytvHWTxZYEcXLgAXFuUuaS3uF9gEi
NQwzGTU1v0FqkqTBr4B8nW3HCN47XUu0t8Y0e+lf4s4OxQawWD79J9/5d3Ry0vbV
3Am1FtGJiJvOwRsIfVChDpYStTcHTCMqtvWbV6L11BWkpzGXSW4Hv43qa+GSYOD2
QU68Mb59oSk2OB+BtOLpJofmbGEGgvmwyCI9MwIDAQABAoIBACiARq2wkltjtcjs
kFvZ7w1JAORHbEufEO1Eu27zOIlqbgyAcAl7q+/1bip4Z/x1IVES84/yTaM8p0go
amMhvgry/mS8vNi1BN2SAZEnb/7xSxbflb70bX9RHLJqKnp5GZe2jexw+wyXlwaM
+bclUCrh9e1ltH7IvUrRrQnFJfh+is1fRon9Co9Li0GwoN0x0byrrngU8Ak3Y6D9
D8GjQA4Elm94ST3izJv8iCOLSDBmzsPsXfcCUZfmTfZ5DbUDMbMxRnSo3nQeoKGC
0Lj9FkWcfmLcpGlSXTO+Ww1L7EGq+PT3NtRae1FZPwjddQ1/4V905kyQFLamAA5Y
lSpE2wkCgYEAy1OPLQcZt4NQnQzPz2SBJqQN2P5u3vXl+zNVKP8w4eBv0vWuJJF+
hkGNnSxXQrTkvDOIUddSKOzHHgSg4nY6K02ecyT0PPm/UZvtRpWrnBjcEVtHEJNp
bU9pLD5iZ0J9sbzPU/LxPmuAP2Bs8JmTn6aFRspFrP7W0s1Nmk2jsm0CgYEAyH0X
+jpoqxj4efZfkUrg5GbSEhf+dZglf0tTOA5bVg8IYwtmNk/pniLG/zI7c+GlTc9B
BwfMr59EzBq/eFMI7+LgXaVUsM/sS4Ry+yeK6SJx/otIMWtDfqxsLD8CPMCRvecC
2Pip4uSgrl0MOebl9XKp57GoaUWRWRHqwV4Y6h8CgYAZhI4mh4qZtnhKjY4TKDjx
QYufXSdLAi9v3FxmvchDwOgn4L+PRVdMwDNms2bsL0m5uPn104EzM6w1vzz1zwKz
5pTpPI0OjgWN13Tq8+PKvm/4Ga2MjgOgPWQkslulO/oMcXbPwWC3hcRdr9tcQtn9
Imf9n2spL/6EDFId+Hp/7QKBgAqlWdiXsWckdE1Fn91/NGHsc8syKvjjk1onDcw0
NvVi5vcba9oGdElJX3e9mxqUKMrw7msJJv1MX8LWyMQC5L6YNYHDfbPF1q5L4i8j
8mRex97UVokJQRRA452V2vCO6S5ETgpnad36de3MUxHgCOX3qL382Qx9/THVmbma
3YfRAoGAUxL/Eu5yvMK8SAt/dJK6FedngcM3JEFNplmtLYVLWhkIlNRGDwkg3I5K
y18Ae9n7dHVueyslrb6weq7dTkYDi3iOYRW8HRkIQh06wEdbxt0shTzAJvvCQfrB
jg/3747WSsf/zBTcHihTRBdAv6OmdhV4/dD5YBfLAkLrd+mX7iE=
-----END RSA PRIVATE KEY-----";

#[repr(C)]
struct UART {
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

#[entry]
fn main() -> ! {
  unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE) }

  let signing_key = RsaPrivateKey::from_pkcs1_pem(RSA_PRIVATE_KEY)
    .expect("Invalid private key");

  unsafe {
    let uart = &mut *(0x4000C000 as usize as *mut UART);
    uart.ctl.write(uart.ctl.read() & 0xffff_fffe);
    uart.ibrd.write((uart.ibrd.read() & 0xffff_0000) | 0x000a);
    uart.fbrd.write((uart.fbrd.read() & 0xffff_0000) | 0x0036);
    uart.lcrh.write(0x60);
    uart.ctl.write(uart.ctl.read() | 0x01);

    loop {
      let byte = uart.dr.read() as u8;

      if byte == b's' {
        let mut digest = [0u8; 256 / 8];

        for b in digest.iter_mut() {
          while !(uart.fr.read() & 0x10 == 0) {}
          if uart.fr.read() & 0x10 == 0 {
            *b = uart.dr.read() as u8;
          }
        }

        let mut rng = Hc128Rng::from_seed([0; 32]);
        let padding = PaddingScheme::new_pss_with_salt::<Sha256, _>(rng, 32);

        // 256 bytes
        let signature = signing_key.sign(padding, &digest).unwrap();

        for b in signature {
          unsafe {
            uart.dr.write(b.into());
          }
        }
      }

      if byte == b'v' {
        let mut digest = [0u8; 256 / 8];
        let mut signature = [0u8; 256];
        for b in digest.iter_mut() {
          while !(uart.fr.read() & 0x10 == 0) {}
          if uart.fr.read() & 0x10 == 0 {
            *b = uart.dr.read() as u8;
          }
        }

        for b in signature.iter_mut() {
          while !(uart.fr.read() & 0x10 == 0) {}
          if uart.fr.read() & 0x10 == 0 {
            *b = uart.dr.read() as u8;
          }
        }

        let mut rng = Hc128Rng::from_seed([0; 32]);
        let padding = PaddingScheme::new_pss_with_salt::<Sha256, _>(rng, 32);

        let verification =
          signing_key.verify(padding, &digest, &signature).is_ok();

        uart.dr.write(if verification { 1 } else { 0 });
      }
    }
  }
}

#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
  hprintln!("{}", "Out of memory.").unwrap();
  asm::bkpt();

  loop {}
}
