import { assertEquals } from "https://deno.land/std@0.117.0/testing/asserts.ts";

const fd = await Deno.open("/dev/pts/3", { read: true, write: true });

const SIGN = new Uint8Array([0x00]);
const VERIFY = new Uint8Array([0x01]);

async function sign(message: Uint8Array): Promise<Uint8Array> {
  const hashed = await crypto.subtle.digest("SHA-256", message);
  await fd.write(SIGN);
  await fd.write(new Uint8Array(hashed));

  let signature: number[] = [];
  let read = 0;
  while (read !== 256) {
    let buf = new Uint8Array(1);
    
    read += await fd.read(buf) || 0;
    console.log(buf)
    signature = [...signature, ...buf];
  }

  return new Uint8Array(signature);
}

async function verify(
  data: Uint8Array,
  signature: Uint8Array,
): Promise<boolean> {
  assertEquals(signature.byteLength, 256);

  const hashed = await crypto.subtle.digest("SHA-256", data);
  await fd.write(VERIFY);
  await fd.write(new Uint8Array(hashed));
  await fd.write(signature);

  let result = new Uint8Array(1);
  assertEquals(await fd.read(result), 1);

  return result[0] == 1;
}

// Test sign
const message = new Uint8Array(256 / 8);
const signature = await sign(message);
assertEquals(signature.byteLength, 256);

// Test verify
assertEquals(
  await verify(message, signature),
  true,
);

assertEquals(
  await verify(
    await crypto.getRandomValues(new Uint8Array(256 / 8)),
    signature,
  ),
  false,
);
