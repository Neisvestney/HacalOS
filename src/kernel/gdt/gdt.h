#ifndef MODULETOS_GDT_H
#define MODULETOS_GDT_H

#include <stdint.h>

struct GDTDescriptor {
    uint16_t Size;
    uint64_t Offset;
} __attribute__((packed));

struct GDTEntry {
    uint16_t Limit0;
    uint16_t Base0;
    uint8_t Base1;
    uint8_t AccessByte;
    uint8_t Limit1: 4;
    uint8_t Flags : 4;
    uint8_t Base2;
}__attribute__((packed));

struct GDTSystemEntry {
    uint16_t Limit0;
    uint16_t Base0;
    uint8_t Base1;
    uint8_t AccessByte;
    uint8_t Limit1: 4;
    uint8_t Flags : 4;
    uint8_t Base2;
    uint32_t Base3;
    uint32_t Reserved;
    void SetBase(uint64_t base);
    void SetLimit(uint32_t limit);
}__attribute__((packed));

struct GDT {
    GDTEntry Null; //0x00
    GDTEntry KernelCode; //0x08
    GDTEntry KernelData; //0x10
    GDTEntry UserCode; //0x18
    GDTEntry UserData; //0x20
    GDTSystemEntry TSS; //0x28
} __attribute__((packed))
    __attribute((aligned(0x1000)));

extern GDT DefaultGDT;

extern "C" void LoadGDT(GDTDescriptor* gdtDescriptor);

#endif //MODULETOS_GDT_H
