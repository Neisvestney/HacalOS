#ifndef HACALOS_PS2_H
#define HACALOS_PS2_H

#include <stdint.h>
#include "../io.h"

#define PS2_DATA 0x60
#define PS2_COMMAND 0x64

#define READ_CONFIG_BYTE_COMMAND 0x20
#define WRITE_CONFIG_BYTE_COMMAND 0x60

void waitPS2ControllerReady();
void waitPS2ControllerInputReady();
void sendPS2Command(uint8_t command, uint8_t data);
uint8_t readPS2();

#endif //HACALOS_PS2_H
