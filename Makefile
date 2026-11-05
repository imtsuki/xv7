TARGET_KERNEL := x86_64-unknown-none
TARGET_UEFI   := x86_64-unknown-uefi
PROFILE       ?= debug
CARGO_FLAGS   := $(if $(filter release,$(PROFILE)),--release,)

ESP ?= target/esp

QEMU_X86_64 ?= qemu-system-x86_64
QEMU_X86_64_OPT := \
	-drive if=pflash,format=raw,file=.qemu/OVMF-pure-efi.fd,readonly=on \
	-device isa-debug-exit,iobase=0xf4,iosize=0x04 \
	-serial stdio \
	-m 1G \
	-net none

.PHONY: all build user kernel bootloader compose-esp qemu qemu-kvm qemu-gdb doc clean

all: build

user:
	cargo build -p xv7-user --target $(TARGET_KERNEL) $(CARGO_FLAGS)

kernel: user
	cargo build -p xv7-kernel --target $(TARGET_KERNEL) $(CARGO_FLAGS)

bootloader:
	cargo build -p xv7-bootloader-uefi --target $(TARGET_UEFI) $(CARGO_FLAGS)

build: user kernel bootloader

compose-esp: build
	mkdir -p $(ESP)/EFI/Boot
	cp target/$(TARGET_UEFI)/$(PROFILE)/xv7-bootloader-uefi.efi $(ESP)/EFI/Boot/BootX64.efi
	mkdir -p $(ESP)/EFI/xv7
	cp target/$(TARGET_KERNEL)/$(PROFILE)/xv7-kernel $(ESP)/EFI/xv7/kernel
	rust-objdump \
		--disassemble \
		--source \
		--dynamic-reloc \
		$(ESP)/EFI/xv7/kernel | \
		rustfilt > \
		$(ESP)/EFI/xv7/kernel.asm 2>/dev/null || true

qemu: compose-esp
	$(QEMU_X86_64) $(QEMU_X86_64_OPT) \
		-drive format=raw,file=fat:rw:$(ESP)

qemu-kvm: compose-esp
	$(QEMU_X86_64) $(QEMU_X86_64_OPT) \
		-drive format=raw,file=fat:rw:$(ESP) \
		-enable-kvm

qemu-gdb: compose-esp
	$(QEMU_X86_64) $(QEMU_X86_64_OPT) \
		-drive format=raw,file=fat:rw:$(ESP) \
		-S -gdb tcp::1234

doc:
	cargo doc --no-deps --open

clean:
	cargo clean
