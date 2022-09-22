#include "BasicRenderer.h"

BasicRenderer basicRenderer = BasicRenderer(nullptr, nullptr);

BasicRenderer::BasicRenderer(Framebuffer* framebuffer, PSF1Font* psf1Font) {
    this->framebuffer = framebuffer;
    this->psf1Font = psf1Font;
    this->point = {8, 8};
    this->color = 0xffffff;
}

void BasicRenderer::PutPixel(uint32_t x, uint32_t y, uint32_t color) {
    *((uint32_t *) ((char*)framebuffer->baseAddress + 4 * framebuffer->pixelsPerScanline * y + 4 * x)) = color;
}

void BasicRenderer::PutChar(char chr, uint32_t x, uint32_t y) {
    char* fontPtr = (char*) psf1Font->glyphBuffer + (chr * psf1Font->psf1Header->charSize);
    for (unsigned long yp = y; yp < y + 16; ++yp) {
        for (unsigned long xp = x; xp < x + 8; ++xp) {
            if ((*fontPtr & (0b10000000 >> (xp - x))) > 0) {
                PutPixel(xp, yp, color);
            }
        }
        fontPtr++;
    }
}

void BasicRenderer::PutChar(char chr) {
    PutChar(chr, point.x, point.y);
    point.x += 8;
    if (point.x + 8 > framebuffer->width){
        NextLine();
    }
}

void BasicRenderer::PutChar(char chr, uint32_t x, uint32_t y, uint32_t color) {
    this->color = color;
    PutChar(chr, x, y);
}


void BasicRenderer::NextLine() {
    point.x = 8;
    point.y += 16;
}

void BasicRenderer::Print(const char* string) {
    char* chr = (char*) string;
    while (*chr != 0) {
        if (*chr == '\n') {
            NextLine();
        } else {
            PutChar(*chr, point.x, point.y);
            point.x += 8;
        }
        if (point.x > framebuffer->width + 8) {
            NextLine();
        }
        chr++;
    }
}

void BasicRenderer::Print(const char *str, uint32_t color) {
    this->color = color;
    Print(str);
}

void BasicRenderer::Printl(const char *str) {
    Print(str);
    Print("\n");
}

void BasicRenderer::Printl(const char *str, uint32_t color) {
    Print(str, color);
    Print("\n");
}

void BasicRenderer::ClearChar() {
    if (point.x == 0){
        point.x = framebuffer->width;
        point.y -= 16;
        if (point.y < 0) point.y = 0;
    }

    unsigned int xOff = point.x;
    unsigned int yOff = point.y;

    unsigned int* pixPtr = (unsigned int*)framebuffer->baseAddress;
    for (unsigned long y = yOff; y < yOff + 16; y++){
        for (unsigned long x = xOff - 8; x < xOff; x++){
            *(unsigned int*)(pixPtr + x + (y * framebuffer->pixelsPerScanline)) = 0x000000;
        }
    }

    point.x -= 8;

    if (point.x < 0){
        point.x = framebuffer->width;
        point.y -= 16;
        if (point.y < 0) point.y = 0;
    }
}
