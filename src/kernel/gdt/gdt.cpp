#include "gdt.h"

__attribute__((aligned(0x1000)))
GDT defaultGDT = {
        {0, 0, 0, 0x00, 0x00, 0}, // null
        {0xFFF, 0, 0, 0x9a, 0xF, 0xA, 0}, // kernel code segment
        {0xFFF, 0, 0, 0x92, 0xF, 0xC, 0}, // kernel data segment
        {0xFFF, 0, 0, 0xfA, 0xF, 0xA, 0}, // user code segment
        {0xFFF, 0, 0, 0xF2, 0xF, 0xC, 0}, // user data segment
        {0, 0, 0, 0x89, 0, 0, 0}, // TSS
};

void GDTSystemEntry::SetBase(uint64_t base) {
    base0 = (uint16_t)(base & 0x000000000000ffff);
    base1 = (uint8_t)((base & 0x0000000000ff0000) >> 16);
    base2 = (uint8_t)((base & 0x00000000ff000000) >> 24);
    base3 = (uint32_t)((base & 0xffffffff00000000) >> 32);
}

void GDTSystemEntry::SetLimit(uint32_t limit) {
    limit0 = (uint16_t)(limit & 0x0000ffff);
    limit1 = (uint8_t)((limit & 0x000f0000) >> 16);
}
