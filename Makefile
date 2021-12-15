build:
		~/riscv64-unknown-elf-gcc-8.1.0-2019.01.0-x86_64-linux-ubuntu14/bin/riscv64-unknown-elf-objcopy -O binary target/riscv32imac-unknown-none-elf/release/arienai firmware.bin
		~/dfu-util-0.11-binaries/linux-amd64/dfu-util  -a 0 -s 0x08000000:leave -D firmware.bin
