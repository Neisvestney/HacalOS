OUT         := ./build
OVMF        := /usr/share/ovmf/OVMF.fd

DEBUG       ?= 0
RELEASE     ?= 0

ifeq ($(DEBUG),1)
ifeq ($(RELEASE),1)
$(error DEBUG and RELEASE cannot both be 1)
endif
QEMU_DEBUG_FLAGS := -s -S
endif

ifeq ($(RELEASE),1)
CARGO_FLAGS := --release
CARGO_DIR   := release
else
CARGO_FLAGS :=
CARGO_DIR   := debug
endif

BOOTLOADER_EFI := $(CURDIR)/src/bootloader/target/x86_64-unknown-uefi/$(CARGO_DIR)/bootloader.efi
KERNEL_ELF     := $(CURDIR)/src/kernel/target/x86_64-hacal_os/$(CARGO_DIR)/kernel

BOOTLOADER_SRCS := $(shell find src/bootloader/src -name '*.rs') \
                   src/bootloader/Cargo.toml                     \
                   src/bootloader/Cargo.lock

KERNEL_SRCS     := $(shell find src/kernel/src -name '*.rs') \
                   src/kernel/Cargo.toml                     \
                   src/kernel/Cargo.lock

APP_INIT_ELF     := $(CURDIR)/src/apps/target/x86_64-unknown-none/$(CARGO_DIR)/init
APP_TEST_BIN_ELF     := $(CURDIR)/src/apps/target/x86_64-unknown-none/$(CARGO_DIR)/test_bin

APPS_SRCS       := $(shell find src/apps/init) \
				   $(shell find src/apps/test_bin) \
				   src/apps/Cargo.toml    \
				   src/apps/Cargo.lock

-include src/bootloader/target/x86_64-unknown-uefi/$(CARGO_DIR)/bootloader.d
-include src/kernel/target/x86_64-hacal_os/$(CARGO_DIR)/kernel.d

-include src/apps/target/x86_64-unknown-none/$(CARGO_DIR)/init.d
-include src/apps/target/x86_64-unknown-none/$(CARGO_DIR)/test_bin.d

.PHONY: all bootloader kernel run clean help os.img os.iso os.vdi

all: os.iso

## --- Build targets ---

$(BOOTLOADER_EFI): $(BOOTLOADER_SRCS)
	cd src/bootloader && cargo build $(CARGO_FLAGS)

$(KERNEL_ELF): $(KERNEL_SRCS)
	cd src/kernel && cargo build $(CARGO_FLAGS)

$(APP_INIT_ELF) $(APP_TEST_BIN_ELF): $(APPS_SRCS)
	cd src/apps && cargo build $(CARGO_FLAGS)

bootloader: $(BOOTLOADER_EFI)
kernel: $(KERNEL_ELF)

$(OUT)/part.img: $(BOOTLOADER_EFI) $(KERNEL_ELF) ./src/files/spleen-8x16-v2.psf $(OUT)/inithfs | $(OUT)
	truncate -s $$((91669 * 512)) $@
	mformat -i $@ -h 32 -t 32 -n 64 -c 1
	mmd -i $@ ::/EFI ::/EFI/BOOT
	mcopy -i $@ $(BOOTLOADER_EFI) ::/EFI/BOOT/BOOTX64.EFI
	mcopy -i $@ $(KERNEL_ELF) ::kernel.elf
	mcopy -i $@ ./src/files/spleen-8x16-v2.psf ::
	mcopy -i $@ $(OUT)/inithfs ::

$(OUT)/os.img: $(OUT)/part.img | $(OUT)
	truncate -s $$((93750 * 512)) $@
	parted $@ -s -a minimal mklabel gpt
	parted $@ -s -a minimal mkpart EFI FAT16 2048s 93716s
	parted $@ -s -a minimal toggle 1 boot
	dd if=$(OUT)/part.img of=$@ bs=1M conv=notrunc seek=1


$(OUT)/inithfs: $(APP_INIT_ELF) $(APP_TEST_BIN_ELF)
	mkdir -p $(OUT)/inithfs-root
	cp $(APP_INIT_ELF) $(OUT)/inithfs-root
	cp $(APP_TEST_BIN_ELF) $(OUT)/inithfs-root
	cd src/create-inithfs && cargo run --bin create-inithfs -- ../../$(OUT)/inithfs-root ../../$(OUT)/inithfs

## --- Image formats ---

os.img: $(OUT)/os.img

os.iso: $(OUT)/os.img | $(OUT)/iso
	cp $(OUT)/os.img $(OUT)/iso/
	xorriso -as mkisofs -R -f --efi-boot os.img -o $(OUT)/os.iso $(OUT)/iso

os.vdi: $(OUT)/os.img
	VBoxManage convertfromraw --format VDI $< $(OUT)/os.vdi

## --- Utility ---

run: $(OUT)/os.img
	qemu-system-x86_64 -machine q35 $(QEMU_DEBUG_FLAGS) -m 256M \
	    --bios $(OVMF) -cpu qemu64 -smp 4 \
	    -drive file=$< -serial stdio -usb

clean:
	rm -rf $(OUT)
	rm -rf $(CURDIR)/src/bootloader/target
	rm -rf $(CURDIR)/src/kernel/target
	rm -rf $(CURDIR)/src/create-inihfs/target
	rm -rf $(CURDIR)/src/apps/target

$(OUT) $(OUT)/iso:
	mkdir -p $@

help:
	@echo "Targets: all, bootloader, kernel, os.img, os.iso, os.vdi, run, clean"
	@echo "Options: DEBUG=1   — enable QEMU GDB stub"
	@echo "         RELEASE=1 — build with --release"
