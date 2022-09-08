R = $(shell pwd)/

OBJECTS = main.o
OUT = ../../build
CC = gcc
CFLAGS = -Ignu-efi/inc -fpic -ffreestanding -fno-stack-protector -fno-stack-check -fshort-wchar -mno-red-zone -maccumulate-outgoing-args -c
LD = ld
LDFLAGS = -shared -Bsymbolic -L$(R)../gnu-efi/x86_64/lib -L$(R)../gnu-efi/x86_64/gnuefi -T$(R)../gnu-efi/gnuefi/elf_x86_64_efi.lds
AS = nasm
ASFLAGS = -f elf64


all: BOOTX64.EFI

BOOTX64.EFI: $(OBJECTS)
	cd $(OUT) && $(LD) $(LDFLAGS) $(R)../gnu-efi/x86_64/gnuefi/crt0-efi-x86_64.o $(OBJECTS) -o main.so -lgnuefi -lefi
	objcopy -j .text -j .sdata -j .data -j .dynamic -j .dynsym  -j .rel -j .rela -j .rel.* -j .rela.* -j .reloc --target efi-app-x86_64 --subsystem=10  $(OUT)/main.so  $(OUT)/BOOTX64.EFI

%.o: %.c
	@mkdir -p $(OUT)
	$(CC) $(CFLAGS) $< -o $(OUT)/$@

%.o: %.asm
	@mkdir -p $(OUT)
	$(AS) $(ASFLAGS) $< -o $(OUT)/$@

clean:
	rm -rf $(OUT)