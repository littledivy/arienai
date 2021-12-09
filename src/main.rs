#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate alloc;

use panic_halt as _;

mod heap;
mod msg;
mod rsa;
mod uart;

use riscv::asm;
use riscv_rt::entry;

use rand::Rng;
use rand_core::SeedableRng;
use rand_hc::Hc128Rng;

use core::alloc::Layout;
use core::convert::TryFrom;

use msg::Message;
use sha2::Digest;
use sha2::Sha256;

#[entry]
fn main() -> ! {
  heap::init();
  // XXX: Replace with your private key.
  let n = num_bigint::BigUint::parse_bytes(b"009f3ca2b356637cd0746c180a14c4afbe44edc25bc1b1a26aed2e7003e933795395a555b091675585ae2cdfc5ccfe96bfcabe3b6afefecf75539af0d1c801dc693f76c214441692eff5c8f99537894a26f2aff32b9bf62d8c26555a068e608870ad7c0a2ea3ebff5d629d6b0091f232b6f1d64f165811c5cb8005c5b94b9a4b7b85f60122350c33193535bf416a92a4c1af807c9d6dc708de3b5d4bb4b7c6347be95fe2ce0ec506b0583efd27dff9777472d2f6d5dc09b516d189889bcec11b087d50a10e9612b537074c232ab6f59b57a2f5d415a4a73197496e07bf8dea6be19260e0f6414ebc31be7da12936381f81b4e2e92687e66c610682f9b0c8223d33", 16).unwrap();
  let d = num_bigint::BigUint::parse_bytes(b"288046adb0925b63b5c8ec905bd9ef0d4900e4476c4b9f10ed44bb6ef338896a6e0c8070097babeff56e2a7867fc75215112f38ff24da33ca748286a6321be0af2fe64bcbcd8b504dd920191276ffef14b16df95bef46d7f511cb26a2a7a791997b68dec70fb0c9797068cf9b725502ae1f5ed65b47ec8bd4ad1ad09c525f87e8acd5f4689fd0a8f4b8b41b0a0dd31d1bcabae7814f0093763a0fd0fc1a3400e04966f78493de2cc9bfc88238b483066cec3ec5df7025197e64df6790db50331b3314674a8de741ea0a182d0b8fd16459c7e62dca469525d33be5b0d4bec41aaf8f4f736d45a7b51593f08dd750d7fe15f74e64c9014b6a6000e58952a44db09", 16).unwrap();

  unsafe {
    let mut uart = uart::init();

    loop {
      let byte = uart.read_byte();

      match Message::try_from(byte) {
        Ok(Message::Sign) => {
          let mut digest = [0u8; 256 / 8];
          uart.read(&mut digest);

          let mut rng = Hc128Rng::from_seed([0; 32]);
          // let padding = PaddingScheme::new_pss_with_salt::<Sha256, _>(rng, 32);
          let mut sha256 = Sha256::new();

          let mut salt = [0u8; 32];
          rng.fill(&mut salt[..]);

          // 256 bytes
          let signature = rsa::sign_pss_with_salt(
            &digest,
            &salt,
            &mut sha256,
            &n,
            &d,
            n.bits() as usize,
          )
          .unwrap();

          for b in signature {
            uart.write(b);
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

#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
  loop {}
}
