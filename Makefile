OUT = ./build

DEBUG = 0
RELEASE = 0
QEMU_DEBUG_FLAGS =
CARGO_RELEASE_FLAGS =
CARGO_DIR = debug
ifeq ($(DEBUG),1)
	DEBUG = 1
    QEMU_DEBUG_FLAGS = -s -S
endif

ifeq ($(RELEASE),1)
	RELEASE = 1
    CARGO_RELEASE_FLAGS = --release
    CARGO_DIR = release
endif

all: os.iso

bootloader:
	cd src/bootloader && cargo build $(CARGO_RELEASE_FLAGS)

kernel:
	cd src/kernel && cargo build $(CARGO_RELEASE_FLAGS)

os.img: bootloader kernel
	mkdir -p $(OUT)

	dd if=/dev/zero of=$(OUT)/os.img bs=512 count=93750
	parted $(OUT)/os.img -s -a minimal mklabel gpt
	parted $(OUT)/os.img -s -a minimal mkpart EFI FAT16 2048s 93716s
	parted $(OUT)/os.img -s -a minimal toggle 1 boot

	dd if=/dev/zero of=$(OUT)/part.img bs=512 count=91669
	mformat -i $(OUT)/part.img -h 32 -t 32 -n 64 -c 1

	mmd -i  $(OUT)/part.img ::/EFI
	mmd -i  $(OUT)/part.img ::/EFI/BOOT
	mcopy -i  $(OUT)/part.img  ./src/bootloader/target/x86_64-unknown-uefi/$(CARGO_DIR)/bootloader.efi ::/EFI/BOOT/BOOTX64.EFI
	mcopy -i  $(OUT)/part.img  src/kernel/target/x86_64-hacal_os/$(CARGO_DIR)/kernel ::kernel.elf
#	mcopy -i  $(OUT)/part.img  $(OUT)/bootloader/zap-light16.psf ::
	mcopy -i  $(OUT)/part.img  ./src/files/spleen-8x16-v2.psf ::

	dd if=$(OUT)/part.img of=$(OUT)/os.img bs=512 count=91669 seek=2048 conv=notrunc

os.iso: os.img
	@mkdir -p $(OUT)/iso
	cp $(OUT)/os.img  $(OUT)/iso
	xorriso -as mkisofs -R -f --efi-boot os.img -o $(OUT)/os.iso $(OUT)/iso

os.vdi: os.img
	VBoxManage convertfromraw --format VDI $(OUT)/os.img $(OUT)/os.vdi

run: os.img
	#export DISPLAY=localhost:0.0
	#qemu-system-x86_64 $(QEMU_DEBUG_FLAGS) -machine q35 -drive file=$(OUT)/os.img -m 256M -cpu qemu64 -drive if=pflash,format=raw,unit=0,file="/usr/share/OVMF/OVMF_CODE.fd",readonly=on -drive if=pflash,format=raw,unit=1,file="OVMF_VARS.fd" -net none -serial stdio
	qemu-system-x86_64 -machine q35 $(QEMU_DEBUG_FLAGS) -m 256M --bios /usr/share/ovmf/OVMF.fd -cpu qemu64 -smp 2 -drive file=$(OUT)/os.img -serial stdio -usb


clean:
	#$(MAKE) -C src/bootloader clean
	rm -rf $(OUT)