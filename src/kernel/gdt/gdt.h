#ifndef MODULETOS_GDT_H
#define MODULETOS_GDT_H

#include <stdint.h>

struct GDTDescriptor {
    uint16_t Size;
    uint64_t Offset;
} __attribute__((packed));

struct GDTEntry {
    uint16_t limit0;
    uint16_t base0;
    uint8_t base1;
    uint8_t accessByte;
    uint8_t limit1: 4;
    uint8_t flags : 4;
    uint8_t base2;
}__attribute__((packed));

struct GDTSystemEntry {
    uint16_t limit0;
    uint16_t base0;
    uint8_t base1;
    uint8_t accessByte;
    uint8_t limit1: 4;
    uint8_t flags : 4;
    uint8_t base2;
    uint32_t base3;
    uint32_t reserved;
    void SetBase(uint64_t base);
    void SetLimit(uint32_t limit);
}__attribute__((packed));

struct GDT {
    GDTEntry null; //0x00
    GDTEntry kernelCode; //0x08
    GDTEntry kernelData; //0x10
    GDTEntry userCode; //0x18
    GDTEntry userData; //0x20
    GDTSystemEntry tss; //0x28
} __attribute__((packed))
    __attribute((aligned(0x1000)));

extern GDT defaultGDT;

extern "C" void loadGDT(GDTDescriptor* gdtDescriptor);

#endif //MODULETOS_GDT_H
