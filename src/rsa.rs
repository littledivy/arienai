use alloc::vec;
use alloc::vec::Vec;

use core::ops::Shl;
use crypto_bigint::prelude::ArrayEncoding;
use crypto_bigint::subtle::Choice;
use crypto_bigint::Integer;
use crypto_bigint::Limb;
use crypto_bigint::LimbUInt;
use crypto_bigint::U4096;

use sha2_const::Sha256;
const EM_LEN: usize = (4095 + 7) / 8;
const EM_BITS: usize = 4095;

// signPSSWithSalt calculates the signature of hashed using PSS [1] with specified salt.
/// Note that hashed must be the result of hashing the input message using the
/// given hash function. salt is a random sequence of bytes whose length will be
/// later used to verify the signature.
pub fn sign_pss_with_salt(
  hashed: &[u8],
  salt: &[u8],
  d: &[LimbUInt; 4096 / Limb::BIT_SIZE],
  n: &U4096,
) -> Result<[u8; 512], ()> {
  let em = emsa_pss_encode(hashed, salt)?;

  let c = U4096::from_be_slice(&em);
  let m = decrypt(&c, d, n);

  let mut m_bytes = m.to_be_byte_array();
  let plaintext = left_pad(m_bytes.as_slice());

  Ok(plaintext)
}

// n (in bits) = 4096
fn emsa_pss_encode(m_hash: &[u8], salt: &[u8]) -> Result<[u8; EM_LEN], ()> {
  // See [1], section 9.1.1
  let h_len = 256 / 8;
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

  let mut hashed = Sha256::new()
    .update(&prefix)
    .update(m_hash)
    .update(salt)
    .finalize();

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
  mgf1_xor(db, &h);

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
pub fn mgf1_xor(out: &mut [u8], seed: &[u8]) {
  let mut counter = [0u8; 4];
  let mut i = 0;
  while i < out.len() {
    let mut digest_input = [0u8; 32 + 4];
    digest_input[0..seed.len()].copy_from_slice(seed);
    digest_input[seed.len()..].copy_from_slice(&counter);

    let digest_output =
      Sha256::new().update(digest_input.as_slice()).finalize();
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

#[inline(always)]
fn add_mul_vvw(z: &mut [LimbUInt], x: &[LimbUInt], y: LimbUInt) -> LimbUInt {
  let mut c = 0;
  for (zi, xi) in z.iter_mut().zip(x.iter()) {
    let (z1, z0) = mul_add_www(*xi, y, *zi);
    let (c_, zi_) = add_ww(z0, c, 0);
    *zi = zi_;
    c = c_ + z1;
  }

  c
}

#[inline(always)]
fn add_ww(x: LimbUInt, y: LimbUInt, c: LimbUInt) -> (LimbUInt, LimbUInt) {
  let yc = y.wrapping_add(c);
  let z0 = x.wrapping_add(yc);
  let z1 = if z0 < x || yc < y { 1 } else { 0 };

  (z1, z0)
}

/// z1 << _W + z0 = x * y + c
#[inline(always)]
fn mul_add_www(x: LimbUInt, y: LimbUInt, c: LimbUInt) -> (LimbUInt, LimbUInt) {
  let z: u64 = x as u64 * y as u64 + c as u64;
  ((z >> Limb::BIT_SIZE) as u32, z as u32)
}

/// The resulting carry c is either 0 or 1.
#[inline(always)]
fn sub_vv(z: &mut [LimbUInt], x: &[LimbUInt], y: &[LimbUInt]) -> LimbUInt {
  let mut c = 0;
  for (i, (xi, yi)) in x.iter().zip(y.iter()).enumerate().take(z.len()) {
    let zi = xi.wrapping_sub(*yi).wrapping_sub(c);
    z[i] = zi;
    // see "Hacker's Delight", section 2-12 (overflow detection)
    c = ((yi & !xi) | ((yi | !xi) & zi)) >> (Limb::BIT_SIZE - 1)
  }

  c
}

/// Computes z mod m = x * y * 2 ** (-n*_W) mod m
/// assuming k = -1/m mod 2**_W
/// See Gueron, "Efficient Software Implementations of Modular Exponentiation".
/// https://eprint.iacr.org/2011/239.pdf
/// In the terminology of that paper, this is an "Almost Montgomery Multiplication":
/// x and y are required to satisfy 0 <= z < 2**(n*_W) and then the result
/// z is guaranteed to satisfy 0 <= z < 2**(n*_W), but it may not be < m.
fn montgomery(x: &U4096, y: &U4096, m: &U4096, k: LimbUInt, n: usize) -> U4096 {
  // This code assumes x, y, m are all the same length, n.
  // (required by addMulVVW and the for loop).
  // It also assumes that x, y are already reduced mod m,
  // or else the result will not be properly reduced.

  let mut z = [0 as LimbUInt; (4096 / Limb::BIT_SIZE) * 2];

  let x_data = x.to_uint_array();
  let y_data = y.to_uint_array();
  let m_data = m.to_uint_array();

  let mut c: LimbUInt = 0;
  for i in 0..n {
    let c2 = add_mul_vvw(&mut z[i..n + i], &x_data, y_data[i]);
   
    let t = z[i].wrapping_mul(k);
    let c3 = add_mul_vvw(&mut z[i..n + i], &m_data, t);
    
    let cx = c.wrapping_add(c2);
    let cy = cx.wrapping_add(c3);
    z[n + i] = cy;

    if cx < c2 || cy < c3 {
      c = 1;
    } else {
      c = 0;
    }
  }
  
  if c == 0 {
    let mut a = [0 as LimbUInt; (4096 / Limb::BIT_SIZE)];
    a.copy_from_slice(&z[n..]);
    U4096::from_uint_array(a)
  } else {
    {
      let (mut first, second) = z.split_at_mut(n);
      sub_vv(&mut first, &second, &m_data);
    }
    let mut a = [0 as LimbUInt; (4096 / Limb::BIT_SIZE)];
    a.copy_from_slice(&z[..n]);
    U4096::from_uint_array(a)
  }
}

struct MontyReducer {
  n0inv: LimbUInt,
}

// k0 = -m**-1 mod 2**BITS. Algorithm from: Dumas, J.G. "On Newton–Raphson
// Iteration for Multiplicative Inverses Modulo Prime Powers".
fn inv_mod_alt(b: LimbUInt) -> LimbUInt {
  assert_ne!(b & 1, 0);

  let mut k0 = 2 - b;
  let mut t = (b - 1);
  let mut i = 1;
  while i < Limb::BIT_SIZE {
    t = t.wrapping_mul(t);
    k0 = k0.wrapping_mul(t + 1);

    i <<= 1;
  }
  k0 as LimbUInt
}

impl MontyReducer {
  fn new(n: &U4096) -> Self {
    let n0inv = inv_mod_alt(n.to_uint_array()[0]);
    MontyReducer { n0inv }
  }
}

/// Performs raw RSA decryption with no padding, resulting in a plaintext `BigUint`.
#[inline]
pub fn decrypt(
  base: &U4096,
  exp_data: &[LimbUInt; 4096 / Limb::BIT_SIZE],
  modulus: &U4096,
) -> U4096 {
  // if odd, monty_modpow
  if modulus.is_odd().into() {
    // x, exponent, modulus
    let x = base;
    let y = exp_data;
    let m = modulus;
    let mr = MontyReducer::new(m);
    let num_words = 4096 / Limb::BIT_SIZE;

    let mut rr = U4096::from_u8(1u8);
    rr = (rr.shl(2 * num_words * Limb::BIT_SIZE)).wrapping_rem(m);

    let one = U4096::from_u8(1u8);

    // powers[i] contains x^i
    let mut powers = [U4096::default(); 1 << 4];

    let mut i = 0;
    powers[i] = montgomery(&one, &rr, &base, mr.n0inv, num_words);
    i += 1;

    powers[i] = montgomery(&x, &rr, &base, mr.n0inv, num_words);
    i += 1;

    for idx in 2..1 << 4 {
      let r = montgomery(&powers[idx - 1], &powers[1], m, mr.n0inv, num_words);
      powers[i] = r;
      i += 1;
    }
    // initialize z = 1 (Montgomery 1)
    let mut z = powers[0].clone();
    let mut zz = U4096::default();

    // same windowed exponent, but with Montgomery multiplications
    for i in (0..y.len()).rev() {
      let mut yi = y[i];
      let mut j = 0;
      while j < Limb::BIT_SIZE {
        if i != y.len() - 1 || j != 0 {
          zz = montgomery(&z, &z, m, mr.n0inv, num_words);
          z = montgomery(&zz, &zz, m, mr.n0inv, num_words);
          zz = montgomery(&z, &z, m, mr.n0inv, num_words);
          z = montgomery(&zz, &zz, m, mr.n0inv, num_words);
        }
        zz = montgomery(
          &z,
          &powers[(yi >> (Limb::BIT_SIZE - 4)) as usize],
          m,
          mr.n0inv,
          num_words,
        );
        core::mem::swap(&mut z, &mut zz);
        yi <<= 4;
        j += 4;
      }
    }

    // convert to regular number
    zz = montgomery(&z, &one, m, mr.n0inv, num_words);
    // One last reduction, just in case.
    // See golang.org/issue/13907.
    if zz >= *m {
      // Common case is m has high bit set; in that case,
      // since zz is the same length as m, there can be just
      // one multiple of m to remove. Just subtract.
      // We think that the subtract should be sufficient in general,
      // so do that unconditionally, but double-check,
      // in case our beliefs are wrong.
      // The div is not expected to be reached.
      zz = zz.wrapping_sub(m);
      if zz >= *m {
        zz = zz.wrapping_rem(m);
      }
    }

    return zz;
  }

  // plain_modpow
  let i = match exp_data.iter().position(|&r| r != 0) {
    None => {
      return U4096::from_u8(1u8);
    }
    Some(i) => i,
  };

  let mut base = base.wrapping_rem(modulus);

  for _ in 0..i {
    for _ in 0..Limb::BIT_SIZE {
      base = base.wrapping_mul(&base).wrapping_rem(modulus);
    }
  }

  let mut r = exp_data[i];
  let mut b = 0u8;
  while r % 2 == 0 {
    base = base.wrapping_mul(&base).wrapping_rem(modulus);
    r >>= 1;
    b += 1;
  }

  let mut exp_iter = exp_data[i + 1..].iter();
  if exp_iter.len() == 0 && r == 1 {
    return base;
  }

  let mut acc = base.clone();
  r >>= 1;
  b += 1;

  {
    let mut unit = |exp_is_odd| {
      base = base.wrapping_mul(&base).wrapping_rem(modulus);
      if exp_is_odd {
        acc = acc.wrapping_mul(&base);
        acc = acc.wrapping_rem(modulus);
      }
    };

    if let Some(&last) = exp_iter.next_back() {
      // consume exp_data[i]
      for _ in b as usize..Limb::BIT_SIZE {
        r >>= 1;
      }

      // consume all other digits before the last
      for &r in exp_iter {
        let mut r = r;
        for _ in 0..Limb::BIT_SIZE {
          r >>= 1;
        }
      }
      r = last;
    }

    while r != 0 {
      r >>= 1;
    }
  }
  acc
}

#[inline]
pub fn left_pad(input: &[u8]) -> [u8; 512] {
  let n = if input.len() > 512 { 512 } else { input.len() };

  let mut out = [0u8; 512];
  out[512 - n..].copy_from_slice(input);
  out
}
