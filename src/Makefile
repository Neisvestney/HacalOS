OBJECTS = loader.o kmain.o io.o fb.o serial.o
OUT = ./build
CC = gcc
CFLAGS = -m32 -nostdlib -nostdinc -fno-builtin -fno-stack-protector \
		 -nostartfiles -nodefaultlibs -Wall -Wextra -Werror -c
LDFLAGS = -melf_i386
AS = nasm
ASFLAGS = -f elf

export R=$(shell pwd)

all: kernel.elf

kernel.elf: $(OBJECTS)
	cd $(OUT) && ld $(LDFLAGS) -T $(R)/link.ld $(OBJECTS) -o kernel.elf

os.iso: kernel.elf
	cp -rf iso $(OUT)
	cp $(OUT)/kernel.elf $(OUT)/iso/boot/kernel.elf
	# grub-efi-amd64-bin, grub-pc-bin and mtools
	grub-mkrescue -o $(OUT)/os.iso $(OUT)/iso
	# grub-mkimage --format=x86_64-efi -p $(OUT)/iso/boot/grub -o $(OUT)/os.img multiboot fat

run: os.iso
	 export DISPLAY=$(ip route|awk '/^default/{print $3}'):0.0
	 #qemu-system-x86_64 -bios /usr/share/ovmf/OVMF.fd -cpu qemu64 -cdrom $(OUT)/os.iso -serial stdio
	 qemu-system-x86_64 -cpu qemu64 -cdrom $(OUT)/os.iso -serial stdio

%.o: %.c
	mkdir -p $(OUT)
	$(CC) $(CFLAGS)  $< -o $(OUT)/$@

%.o: %.asm
	mkdir -p $(OUT)
	$(AS) $(ASFLAGS) $< -o $(OUT)/$@

clean:
	rm -rf $(OUT)