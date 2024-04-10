OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start)

SECTIONS
{
    . = 0xFFFF810000000000;

    . = ALIGN(4K);
    .text : {
      *(.text*)
      *(.rodata*)
      *(.data*)
      *(.got*)
      *(.eh_frame_hdr*)
      *(.eh_frame*)
      *(.bss*)
    }
}