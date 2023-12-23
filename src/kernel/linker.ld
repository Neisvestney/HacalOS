OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start)

SECTIONS
{
    . = 0xFFFF810000000000;

     . = ALIGN(1K);
    .text : {
        *(.text*)
    }

    . = ALIGN(1K);
    .rodata : {
        *(.rodata*)
    }

     . = ALIGN(1K);
    .data : {
        *(.data*)
    }

     . = ALIGN(1K);
	.bss : {
		*(COMMON)
		*(.bss)
	}
}