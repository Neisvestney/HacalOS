#include "BasicRenderer.h"

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

void BasicRenderer::PutChar(char chr, uint32_t x, uint32_t y, uint32_t color) {
    this->color = color;
    PutChar(chr, x, y);
}

void BasicRenderer::Print(const char* string) {
    char* chr = (char*) string;
    while (*chr != 0) {
        if (*chr == '\n') {
            point.x = 8;
            point.y += 16;
        } else {
            PutChar(*chr, point.x, point.y);
            point.x += 8;
        }
        if (point.x > framebuffer->pixelsPerScanline) {
            point.x = 8;
            point.y += 16;
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
