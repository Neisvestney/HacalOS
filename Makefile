OUT = ./build

DEBUG = 0
QEMU_DEBUG_FLAGS =
ifeq ($(DEBUG),1)
	DEBUG = 1
    QEMU_DEBUG_FLAGS = -s -S
endif

all: os.iso

loader:
	$(MAKE) -C src/bootloader

kernel:
	$(MAKE) DEBUG=$(DEBUG) -C src/kernel

os.img: loader kernel
	dd if=/dev/zero of=$(OUT)/os.img bs=1k count=1440
	mformat -i  $(OUT)/os.img -f 1440 ::
	mmd -i  $(OUT)/os.img ::/EFI
	mmd -i  $(OUT)/os.img ::/EFI/BOOT
	mcopy -i  $(OUT)/os.img  $(OUT)/BOOTX64.EFI ::/EFI/BOOT
	mcopy -i  $(OUT)/os.img  $(OUT)/kernel.elf ::
	mcopy -i  $(OUT)/os.img  $(OUT)/zap-light16.psf ::
	mcopy -i  $(OUT)/os.img  $(OUT)/zap-ext-light16.psf ::

os.iso: os.img
	@mkdir -p $(OUT)/iso
	cp $(OUT)/os.img  $(OUT)/iso
	xorriso -as mkisofs -R -f --efi-boot os.img -o $(OUT)/os.iso $(OUT)/iso

run: os.img
	export DISPLAY=$(ip route|awk '/^default/{print $3}'):0.0
	#qemu-system-x86_64 $(QEMU_DEBUG_FLAGS) -machine q35 -drive file=$(OUT)/os.img -m 256M -cpu qemu64 -drive if=pflash,format=raw,unit=0,file="/usr/share/OVMF/OVMF_CODE.fd",readonly=on -drive if=pflash,format=raw,unit=1,file="OVMF_VARS.fd" -net none -serial stdio
	qemu-system-x86_64 -machine q35 $(QEMU_DEBUG_FLAGS) -m 256M --bios /usr/share/ovmf/OVMF.fd -cpu qemu64 -cdrom $(OUT)/os.img -serial stdio -usb


clean:
	$(MAKE) -C src/bootloader clean
	rm -rf $(OUT)