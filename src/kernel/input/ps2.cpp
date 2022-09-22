#include "ps2.h"

void waitPS2ControllerReady() {
    uint64_t timeout = 100000;
    while (timeout--){
        if ((inb(0x64) & 0b10) == 0){
            return;
        }
    }
}

void waitPS2ControllerInputReady(){
    uint64_t timeout = 100000;
    while (timeout--){
        if (inb(0x64) & 0b1){
            return;
        }
    }
}

void sendPS2Command(uint8_t command, uint8_t data) {
    waitPS2ControllerReady();
    outb(PS2_COMMAND, command);
    waitPS2ControllerReady();
    outb(PS2_DATA, data);
}

uint8_t readPS2() {
    waitPS2ControllerInputReady();
    return inb(PS2_DATA);
}