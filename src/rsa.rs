#![allow(arithmetic_overflow)]

use alloc::vec;
use alloc::vec::Vec;
use num_bigint::BigUint;
use num_bigint::ModInverse;
use sha2::digest::DynDigest;

const EM_LEN: usize = (4096 - 1) + 7 / 8;
const EM_BITS: usize = 4096;

// signPSSWithSalt calculates the signature of hashed using PSS [1] with specified salt.
/// Note that hashed must be the result of hashing the input message using the
/// given hash function. salt is a random sequence of bytes whose length will be
/// later used to verify the signature.
pub fn sign_pss_with_salt(
  hashed: &[u8],
  salt: &[u8],
  digest: &mut dyn DynDigest,

  d: &BigUint,
  n: &BigUint,
  key_size: usize,
) -> Result<Vec<u8>, ()> {
  let em = emsa_pss_encode(hashed, salt, digest)?;

  let mut c = BigUint::from_bytes_be(&em);
  let mut m = decrypt(d, n, &c);

  let mut m_bytes = m.to_bytes_be();
  let plaintext = left_pad(&m_bytes, key_size);

  Ok(plaintext)
}

// n (in bits) = 4096
fn emsa_pss_encode(
  m_hash: &[u8],
  salt: &[u8],
  hash: &mut dyn DynDigest,
) -> Result<[u8; EM_LEN], ()> {
  // See [1], section 9.1.1
  let h_len = hash.output_size();
  let s_len = salt.len();

  // 1. If the length of M is greater than the input limitation for the
  //     hash function (2^61 - 1 octets for SHA-1), output "message too
  //     long" and stop.
  //
  // 2.  Let mHash = Hash(M), an octet string of length hLen.
  if m_hash.len() != h_len {
    return Err(());
  }

  // 3. If em_len < h_len + s_len + 2, output "encoding error" and stop.
  if EM_LEN < h_len + s_len + 2 {
    // TODO: Key size too small
    return Err(());
  }

  let mut em = [0; EM_LEN];

  let (db, h) = em.split_at_mut(EM_LEN - h_len - 1);
  let h = &mut h[..(EM_LEN - 1) - db.len()];

  // 4. Generate a random octet string salt of length s_len; if s_len = 0,
  //     then salt is the empty string.
  //
  // 5.  Let
  //       M' = (0x)00 00 00 00 00 00 00 00 || m_hash || salt;
  //
  //     M' is an octet string of length 8 + h_len + s_len with eight
  //     initial zero octets.
  //
  // 6.  Let H = Hash(M'), an octet string of length h_len.
  let prefix = [0u8; 8];

  hash.update(&prefix);
  hash.update(m_hash);
  hash.update(salt);

  let hashed = hash.finalize_reset();
  h.copy_from_slice(&hashed);

  // 7.  Generate an octet string PS consisting of em_len - s_len - h_len - 2
  //     zero octets. The length of PS may be 0.
  //
  // 8.  Let DB = PS || 0x01 || salt; DB is an octet string of length
  //     emLen - hLen - 1.
  db[EM_LEN - s_len - h_len - 2] = 0x01;
  db[EM_LEN - s_len - h_len - 1..].copy_from_slice(salt);

  // 9.  Let dbMask = MGF(H, emLen - hLen - 1).
  //
  // 10. Let maskedDB = DB \xor dbMask.
  mgf1_xor(db, hash, &h);

  // 11. Set the leftmost 8 * em_len - em_bits bits of the leftmost octet in
  //     maskedDB to zero.
  db[0] &= 0xFF >> (8 * EM_LEN - EM_BITS);

  // 12. Let EM = maskedDB || H || 0xbc.
  em[EM_LEN - 1] = 0xBC;

  Ok(em)
}

/// Mask generation function.
///
/// Panics if out is larger than 2**32. This is in accordance with RFC 8017 - PKCS #1 B.2.1
pub fn mgf1_xor(out: &mut [u8], digest: &mut dyn DynDigest, seed: &[u8]) {
  let mut counter = [0u8; 4];
  let mut i = 0;

  const MAX_LEN: u64 = core::u32::MAX as u64 + 1;
  assert!(out.len() as u64 <= MAX_LEN);

  while i < out.len() {
    let mut digest_input = alloc::vec![0u8; seed.len() + 4];
    digest_input[0..seed.len()].copy_from_slice(seed);
    digest_input[seed.len()..].copy_from_slice(&counter);

    digest.update(digest_input.as_slice());
    let digest_output = &*digest.finalize_reset();
    let mut j = 0;
    loop {
      if j >= digest_output.len() || i >= out.len() {
        break;
      }

      out[i] ^= digest_output[j];
      j += 1;
      i += 1;
    }
    inc_counter(&mut counter);
  }
}

fn inc_counter(counter: &mut [u8; 4]) {
  for i in (0..4).rev() {
    counter[i] = counter[i].wrapping_add(1);
    if counter[i] != 0 {
      // No overflow
      return;
    }
  }
}

/// Raw RSA encryption of m with the public key. No padding is performed.
#[inline]
pub fn encrypt(e: &BigUint, n: &BigUint, m: &BigUint) -> BigUint {
  m.modpow(e, n)
}

/// Performs raw RSA decryption with no padding, resulting in a plaintext `BigUint`.
#[inline]
pub fn decrypt(d: &BigUint, n: &BigUint, c: &BigUint) -> BigUint {
  c.modpow(d, n)
}

#[inline]
pub fn left_pad(input: &[u8], size: usize) -> Vec<u8> {
  let n = if input.len() > size {
    size
  } else {
    input.len()
  };

  let mut out = vec![0u8; size];
  out[size - n..].copy_from_slice(input);
  out
}
