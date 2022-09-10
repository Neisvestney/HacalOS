#ifndef MODULETOS_KERNELENTRY_H
#define MODULETOS_KERNELENTRY_H
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

#endif //MODULETOS_KERNELENTRY_H
