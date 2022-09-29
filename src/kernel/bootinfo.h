#ifndef MODULETOS_BOOTINFO_H
#define MODULETOS_BOOTINFO_H

#include "efiMemory.h"
#include "acpi.h"

struct Framebuffer {
    void* baseAddress;
    unsigned long long bufferSize;
    unsigned int width;
    unsigned int height;
    unsigned int pixelsPerScanline;
};

#define PSF1_MAGIC0 0x36
#define PSF1_MAGIC1 0x04

struct PSF1Header {
    unsigned char magic[2];
    unsigned char mode;
    unsigned char charSize;
};

struct PSF1Font {
    PSF1Header* psf1Header;
    void* glyphBuffer;
};

struct BootInfo {
    Framebuffer* framebuffer;
    PSF1Font* psf1Font;
    MemoryMap* memoryMap;
    ACPI::RSDP2* rsdp;
};
#endif //MODULETOS_BOOTINFO_H
