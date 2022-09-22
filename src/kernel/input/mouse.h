#ifndef MODULETOS_MOUSE_H
#define MODULETOS_MOUSE_H

#include "stdint.h"

#define ENABLE_MOUSE_COMMAND 0xA8
#define WRITE_MOUSE 0xD4

#define PS2_LEFT_BUTTON   0b00000001
#define PS2_RIGHT_BUTTON  0b00000010
#define PS2_MIDDLE_BUTTON 0b00000100
#define PS2_5TH_BUTTON    0b00100000
#define PS2_4TH_BUTTON    0b00010000
#define PS2_X_SIGN        0b00010000
#define PS2_Y_SIGN        0b00100000
#define PS2_X_OVERFLOW    0b01000000
#define PS2_Y_OVERFLOW    0b10000000

void handlePS2Mouse(uint8_t data);
void processMousePacket();
void initPS2Mouse();

#endif //MODULETOS_MOUSE_H
