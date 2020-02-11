.PHONY: all amd64 crate_esp run clean

all: amd64 run

amd64:
	@cd bootloader && cargo xbuild --release
	@cd kernel && cargo xbuild --target .cargo/amd64.json --release
	@make crate_esp

# x86_64编译出的文件目录
crate_esp:
	@mkdir -p build/pc/esp/EFI/kernel build/pc/esp/EFI/Boot
	@cp bootloader/target/x86_64-unknown-uefi/release/bootloader.efi build/pc/esp/EFI/Boot/BootX64.efi
	@cp kernel/target/amd64/release/kernel build/pc/esp/EFI/kernel/kernel.elf

# QEMU运行x86_64
run:
	@qemu-system-x86_64 \
    -drive if=pflash,format=raw,file=bootloader/OVMF.fd,readonly=on \
    -drive format=raw,file=fat:rw:build/pc/esp \
    -m 1024 \
    -nographic \
    -no-fd-bootchk \
    -smp 2

clean:
	@cd bootloader && cargo clean
	@cd kernel && cargo clean
	@rm -rf build