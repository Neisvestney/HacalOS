#ifndef MODULETOS_KERNELENTRY_H
#define MODULETOS_KERNELENTRY_H
typedef struct {
    void* BaseAddress;
    unsigned long long BufferSize;
    unsigned int Width;
    unsigned int Height;
    unsigned int PixelsPerScanline;
} FRAMEBUFFER;

#define PSF1_MAGIC0 0x36
#define PSF1_MAGIC1 0x04

typedef struct {
    unsigned char Magic[2];
    unsigned char Mode;
    unsigned char CharSize;
} PSF1_HEADER;

typedef struct {
    PSF1_HEADER* PSF1Header;
    void* GlyphBuffer;
} PSF1_FONT;

#endif //MODULETOS_KERNELENTRY_H
