#ifndef MODULETOS_BASICRENDERER_H
#define MODULETOS_BASICRENDERER_H

#include "math.h"
#include "bootinfo.h"

class BasicRenderer{
public:
    Framebuffer* framebuffer;
    PSF1Font* psf1Font;
    Point point;
    uint32_t color;

    BasicRenderer(Framebuffer* framebuffer, PSF1Font* font);
    void PutPixel(uint32_t x, uint32_t y, uint32_t color);
    void PutChar(char chr);
    void PutChar(char chr, uint32_t x, uint32_t y);
    void PutChar(char chr, uint32_t x, uint32_t y,  uint32_t color);
    void NextLine();
    void Print(const char* str);
    void Print(const char* str,  uint32_t color);
    void Printl(const char* str);
    void Printl(const char* str,  uint32_t color);
    void ClearChar();
};

extern BasicRenderer basicRenderer;

#endif //MODULETOS_BASICRENDERER_H
