#ifndef MODULETOS_BOOTINFO_H
#define MODULETOS_BOOTINFO_H

#include "efi/efiMemory.h"
#include "acpi.h"
#include "efi/efiRuntimeServices.h"

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

struct KernelMapElement {
    uint64_t virtualStart;
    uint64_t physicalStart;
    uint64_t pagesCount;
};

struct BootInfo {
    Framebuffer* framebuffer;
    PSF1Font* psf1Font;
    EFI::MemoryMap* memoryMap;
    ACPI::RSDP2* rsdp;
    EFI::RuntimeServices* runtimeServices;
    uint64_t kernelMapSize;
    KernelMapElement kernelMap[10];
};
#endif //MODULETOS_BOOTINFO_H
