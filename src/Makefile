OBJECTS = kmain.o io.o serial.o
OUT = ./build
CC = gcc
CFLAGS = -Ignu-efi/inc -fpic -ffreestanding -fno-stack-protector -fno-stack-check -fshort-wchar -mno-red-zone -maccumulate-outgoing-args -c
LDFLAGS = -shared -Bsymbolic -L../gnu-efi/x86_64/lib -L../gnu-efi/x86_64/gnuefi -T../gnu-efi/gnuefi/elf_x86_64_efi.lds
AS = nasm
ASFLAGS = -f elf64

export R=$(shell pwd)

all:BOOTX64.EFI

BOOTX64.EFI: $(OBJECTS)
	cd $(OUT) && ld $(LDFLAGS) ../gnu-efi/x86_64/gnuefi/crt0-efi-x86_64.o $(OBJECTS) -o main.so -lgnuefi -lefi
	objcopy -j .text -j .sdata -j .data -j .dynamic -j .dynsym  -j .rel -j .rela -j .rel.* -j .rela.* -j .reloc --target efi-app-x86_64 --subsystem=10  $(OUT)/main.so  $(OUT)/BOOTX64.EFI


os.img: BOOTX64.EFI
	dd if=/dev/zero of=$(OUT)/os.img bs=1k count=1440
	mformat -i  $(OUT)/os.img -f 1440 ::
	mmd -i  $(OUT)/os.img ::/EFI
	mmd -i  $(OUT)/os.img ::/EFI/BOOT
	mcopy -i  $(OUT)/os.img  $(OUT)/BOOTX64.EFI ::/EFI/BOOT

run: os.img
	mkdir -p $(OUT)/iso
	cp $(OUT)/os.img  $(OUT)/iso
	xorriso -as mkisofs -R -f -e os.img -no-emul-boot -o $(OUT)/cdimage.iso $(OUT)/iso

	 export DISPLAY=$(ip route|awk '/^default/{print $3}'):0.0
	 qemu-system-x86_64 -bios /usr/share/ovmf/OVMF.fd -cpu qemu64 -cdrom $(OUT)/cdimage.iso -serial stdio
	 #qemu-system-x86_64 -cpu qemu64 -cdrom $(OUT)/os.iso -serial stdio

%.o: %.c
	mkdir -p $(OUT)
	$(CC) $(CFLAGS) $< -o $(OUT)/$@

%.o: %.asm
	mkdir -p $(OUT)
	$(AS) $(ASFLAGS) $< -o $(OUT)/$@

clean:
	rm -rf $(OUT)