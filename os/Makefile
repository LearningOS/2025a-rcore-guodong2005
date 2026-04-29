# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
FS_IMG := ../ci-user/user/build/fs.img
OBJCOPY := rust-objcopy --binary-architecture=riscv64

# BOARD
BOARD ?= qemu
SBI ?= default
KERNEL_ENTRY_PA := 0x80200000

ifeq ($(SBI), default)
	QEMU_BOOT_ARGS := -bios default -kernel $(KERNEL_ELF)
else
	BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin
	QEMU_BOOT_ARGS := -bios $(BOOTLOADER) -kernel $(KERNEL_ELF)
endif

fsimg:
	cd ../easy-fs-fuse && cargo run --release -- \
		-s ../ci-user/user/build/elf \
		-o $(FS_IMG)

$(KERNEL_BIN): kernel
	$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

kernel: fsimg
	cargo build --release

clean:
	cargo clean

run: $(KERNEL_BIN)
	timeout --foreground 60s qemu-system-riscv64 \
		-machine virt \
		-nographic \
		$(QEMU_BOOT_ARGS) \
		-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
		-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

.PHONY: build kernel clean run fsimg
