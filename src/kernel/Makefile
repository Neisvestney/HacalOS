R = $(shell pwd)/

OUT = ../../build
CC = gcc
CFLAGS = -fpic -ffreestanding -fno-stack-protector -fno-stack-check -fshort-wchar -mno-red-zone -maccumulate-outgoing-args -c
LD = ld
LDFLAGS = -T link.ld -static -Bsymbolic -nostdlib
AS = nasm
ASFLAGS = -f elf64

CSRCPP = $(wildcard *.cpp)
ASSRC = $(wildcard *.asm)
OBJS = $(patsubst %.cpp, $(OUT)/%.o, $(CSRCPP)) $(patsubst %.asm, $(OUT)/%.o, $(ASSRC))

DEBUG = 0
CC_DEBUG_FLAGS =
ifeq ($(DEBUG),1)
	DEBUG = 1
    CC_DEBUG_FLAGS = -g
endif

all: kernel.elf

kernel.elf: $(OBJS)
	$(LD) $(LDFLAGS) $(OBJS) -o $(OUT)/kernel.elf

$(OUT)/%.o: %.cpp
	@mkdir -p $(OUT)
	$(CC) $(CFLAGS) $(CC_DEBUG_FLAGS) $< -o $@

$(OUT)/%.o: %.asm
	@mkdir -p $(OUT)
	$(AS) $(ASFLAGS) $< -o $@

clean:
	rm -rf $(OUT)