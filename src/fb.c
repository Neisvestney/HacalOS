#include "fb.h"
#include "io.h"

/* The I/O ports */
#define FB_COMMAND_PORT         0x3D4
#define FB_DATA_PORT            0x3D5

/* The I/O port commands */
#define FB_HIGH_BYTE_COMMAND    14
#define FB_LOW_BYTE_COMMAND     15

u32int* fb;
//char *fb = (char *) 0xA0000;

void fb_init(u64int address) {
    u32int addr = (u32int)address;
    fb = (u32int*)addr;
}

void fb_move_cursor(unsigned short pos)
{
    outb(FB_COMMAND_PORT, FB_HIGH_BYTE_COMMAND);
    outb(FB_DATA_PORT,    ((pos >> 8) & 0x00FF));
    outb(FB_COMMAND_PORT, FB_LOW_BYTE_COMMAND);
    outb(FB_DATA_PORT,    pos & 0x00FF);
}

void fb_put_pixel(s32int pos, u32int color) {
    fb[pos] = color;
}


void fb_write_cell(unsigned int i, char c, unsigned char fg, unsigned char bg)
{
    fb[i * 2] = c;
    fb[i * 2 + 1] = ((bg & 0x0F) << 4) | (fg & 0x0F);
}

void fb_write_color(char *c, int x, int y, unsigned char fg, unsigned char bg) {
    int i = 0;
    while (c[i]) {
        fb_write_cell(y * 80 + x + i, c[i], fg, bg);
        i++;
    }
}

void fb_write(char *c, int x, int y) {
    fb_write_color(c, x, y, FB_COLOR_WHITE, FB_COLOR_BLACK);
}