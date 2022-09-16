#ifndef MODULETOS_IO_H
#define MODULETOS_IO_H

#include <stdint.h>

void outb (uint16_t port, uint8_t value);
uint8_t inb(uint16_t port);
void io_wait();

#endif //MODULETOS_IO_H
