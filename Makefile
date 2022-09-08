OUT = ./build

all: os.iso

loader:
	$(MAKE) -C src/bootloader

kernel:
	$(MAKE) -C src/kernel

os.img: loader kernel
	dd if=/dev/zero of=$(OUT)/os.img bs=1k count=1440
	mformat -i  $(OUT)/os.img -f 1440 ::
	mmd -i  $(OUT)/os.img ::/EFI
	mmd -i  $(OUT)/os.img ::/EFI/BOOT
	mcopy -i  $(OUT)/os.img  $(OUT)/BOOTX64.EFI ::/EFI/BOOT
	mcopy -i  $(OUT)/os.img  $(OUT)/kernel.elf ::
	mcopy -i  $(OUT)/os.img  $(OUT)/zap-light16.psf ::

os.iso: os.img
	@mkdir -p $(OUT)/iso
	cp $(OUT)/os.img  $(OUT)/iso
	xorriso -as mkisofs -R -f -e os.img -no-emul-boot -o $(OUT)/cdimage.iso $(OUT)/iso

run: os.iso
	 export DISPLAY=$(ip route|awk '/^default/{print $3}'):0.0
	 qemu-system-x86_64 -bios /usr/share/ovmf/OVMF.fd -cpu qemu64 -cdrom $(OUT)/cdimage.iso -serial stdio
	 #qemu-system-x86_64 -cpu qemu64 -cdrom $(OUT)/os.iso -serial stdio


clean:
	$(MAKE) -C src/bootloader clean
	rm -rf $(OUT)