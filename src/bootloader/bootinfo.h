#ifndef HACALOS_BOOTINFO_H
#define HACALOS_BOOTINFO_H

#include "../gnu-efi/inc/efi.h"

typedef struct {
    void* baseAddress;
    unsigned long long bufferSize;
    unsigned int width;
    unsigned int height;
    unsigned int pixelsPerScanline;
} Framebuffer;

#define PSF1_MAGIC0 0x36
#define PSF1_MAGIC1 0x04

typedef struct {
    unsigned char magic[2];
    unsigned char mode;
    unsigned char charSize;
} PSF1Header;

typedef struct {
    PSF1Header* psf1Header;
    void* glyphBuffer;
} PSF1Font;

typedef struct {
    EFI_MEMORY_DESCRIPTOR* map;
    UINTN mapSize;
    UINTN descriptorSize;
} MemoryMap;

typedef struct {
    uint64_t virtualStart;
    uint64_t physicalStart;
    uint64_t pagesCount;
} KernelMapElement;

typedef struct {
    Framebuffer* framebuffer;
    PSF1Font* psf1Font;
    MemoryMap* memoryMap;
    void* rsdp;
    EFI_RUNTIME_SERVICES *RT;
    uint64_t kernelMapSize;
    KernelMapElement kernelMap[10];
} BootInfo;
#endif //HACALOS_BOOTINFO_H
