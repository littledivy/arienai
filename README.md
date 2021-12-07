 Hardware wallet for Arweave. For Cortex M3 processors.

Supports key storage, signing and verifying RSA signatures.

**WIP Use at your own risk.**

## Building

```bash
$ rustup target add thumbv7m-none-eabi
$ sudo apt install qemu-system-arm

# `cargo run` starts QEMU emulation for lm3s6965evb
# QEMU redirects serial communication to a `/dev/pts/` device.
$ cargo run --release
# Run the test runner with Deno.
$ deno run --allow-read --allow-write test_runner.ts
```

## Supported microcontrollers

- lm3s6965 (Tested on lm3s6965evb QEMU)

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
