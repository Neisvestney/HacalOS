R = $(shell pwd)/
rwildcard=$(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))

OUT = ../../build/kernel
CC = gcc
CFLAGS = -fpic -ffreestanding -fno-stack-protector -fno-stack-check -fno-exceptions -fshort-wchar -mno-red-zone -maccumulate-outgoing-args -c
LD = ld
LDFLAGS = -T link.ld -static -Bsymbolic -nostdlib --no-relax
AS = nasm
ASFLAGS = -f elf64

CPP_SRCS = $(call rwildcard,.,*.cpp)
ASM_SRCS = $(call rwildcard,.,*.asm)
OBJS = $(patsubst %.cpp, $(OUT)/%.o, $(CPP_SRCS)) $(patsubst %.asm, $(OUT)/%_asm.o, $(ASM_SRCS))

DEBUG = 0
CC_DEBUG_FLAGS =
ifeq ($(DEBUG),1)
	DEBUG = 1
    CC_DEBUG_FLAGS = -g
endif

all: kernel.elf

kernel.elf: $(OBJS)
	$(LD) $(LDFLAGS) $(OBJS) -o $(OUT)/kernel.elf

$(OUT)/./interrupts/interrupts.o: ./interrupts/interrupts.cpp
	@mkdir -p $(dir $@)
	$(CC) -mno-red-zone -mgeneral-regs-only -ffreestanding -c $^ -o $@

$(OUT)/%.o: %.cpp
	@mkdir -p $(dir $@)
	$(CC) $(CFLAGS) $(CC_DEBUG_FLAGS) $< -o $@

$(OUT)/%_asm.o: %.asm
	@mkdir -p $(dir $@)
	$(AS) $(ASFLAGS) $< -o $@

clean:
	rm -rf $(OUT)