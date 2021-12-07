#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate alloc;

use panic_itm as _;

mod heap;
mod lm3s6965_uart;
mod msg;

use cortex_m::asm;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use rand_core::SeedableRng;
use rand_hc::Hc128Rng;

use core::alloc::Layout;
use core::convert::TryFrom;

use msg::Message;
use rsa::pkcs1::FromRsaPrivateKey;
use rsa::PaddingScheme;
use rsa::PublicKey;
use rsa::RsaPrivateKey;
use sha2::Sha256;

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

#[entry]
fn main() -> ! {
  heap::init();

  let signing_key = RsaPrivateKey::from_pkcs1_pem(RSA_PRIVATE_KEY)
    .expect("Invalid private key");

  unsafe {
    let uart = lm3s6965_uart::init();

    loop {
      let byte = uart.read_byte();

      match Message::try_from(byte) {
        Ok(Message::Sign) => {
          let mut digest = [0u8; 256 / 8];
          uart.read(&mut digest);

          let rng = Hc128Rng::from_seed([0; 32]);
          let padding = PaddingScheme::new_pss_with_salt::<Sha256, _>(rng, 32);

          // 256 bytes
          let signature = signing_key.sign(padding, &digest).unwrap();

          for b in signature {
            uart.write(b);
          }
        }
        Ok(Message::Verify) => {
          let mut digest = [0u8; 256 / 8];
          uart.read(&mut digest);

          let mut signature = [0u8; 256];
          uart.read(&mut signature);

          let rng = Hc128Rng::from_seed([0; 32]);
          let padding = PaddingScheme::new_pss_with_salt::<Sha256, _>(rng, 32);

          let verification =
            signing_key.verify(padding, &digest, &signature).is_ok();

          uart.write(if verification { 1 } else { 0 });
        }
        Ok(Message::GetAddress) => {}
        Ok(Message::GetOwner) => {}
        Err(_) => {}
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
