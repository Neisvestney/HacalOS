global loader                   ; the entry symbol for ELF
extern kmain

MAGIC_NUMBER equ 0x1BADB002       ; define the magic number constant
FLAGS        equ 7                ; multiboot flags
CHECKSUM     equ -(MAGIC_NUMBER + FLAGS) ; calculate the checksum
                                  ; (magic number + checksum + flags should equal 0)
MODE_TYPE equ 0
WIDTH equ 1024
HEIGHT equ 768
; WIDTH equ 640
; HEIGHT equ 480
DEPTH equ 32

HEADER_ADDR equ 0
LOAD_ADDR equ 0
LOAD_END_ADDR equ 0
BSS_END_ADDR equ 0
ENTRY_ADDR equ 0

KERNEL_STACK_SIZE equ 4096        ; size of stack in bytes

section .bss
align 4                          ; align at 4 bytes
kernel_stack:                    ; label points to beginning of memory
    resb KERNEL_STACK_SIZE       ; reserve stack for the kernel

section .text                   ; start of the text (code) section
align 4                         ; the code must be 4 byte aligned
    dd MAGIC_NUMBER             ; write the magic number to the machine code,
    dd FLAGS                    ; the flags,
    dd CHECKSUM                 ; and the checksum
    dd HEADER_ADDR
    dd LOAD_ADDR
    dd LOAD_END_ADDR
    dd BSS_END_ADDR
    dd ENTRY_ADDR
    dd MODE_TYPE
    dd WIDTH
    dd HEIGHT
    dd DEPTH

loader:                         ; the loader label (defined as entry point in linker script)
    mov esp, kernel_stack + KERNEL_STACK_SIZE
    push ebx
    cli
    call kmain
.loop:
    jmp .loop                   ; loop forever
