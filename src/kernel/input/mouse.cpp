#include "mouse.h"
#include "ps2.h"
#include "../math.h"
#include "../BasicRenderer.h"
#include "../cstr.h"

uint8_t mouseMode = 0;
uint8_t mouseCycle = 0;
uint8_t mousePacket[4];
bool first = false;

void handlePS2Mouse(uint8_t data) {
    if (mouseCycle == 0 && (data & 0b00001000) == 0) return;
    mousePacket[mouseCycle] = data;
    mouseCycle++;
    if (mouseCycle > 3) {
        processMousePacket();
        mouseCycle = 0;
    }
}

SPoint mousePosition;
SPoint mousePositionOld;

void processMousePacket() {
    bool xNegative, yNegative, xOverflow, yOverflow;

    xNegative = (mousePacket[0] & PS2_X_SIGN) != 0;
    yNegative = (mousePacket[0] & PS2_Y_SIGN) != 0;
    xOverflow = (mousePacket[0] & PS2_X_OVERFLOW) != 0;
    yOverflow = (mousePacket[0] & PS2_Y_OVERFLOW) != 0;

    //mousePosition.x += (xNegative ? -1 : 1) * (int16_t)(mousePacket[1]);
    //mousePosition.y += (yNegative ? -1 : 1) * (int16_t)(mousePacket[2]);

    xNegative ? mousePosition.x -= 256 - mousePacket[1] : mousePosition.x += mousePacket[1];
    yNegative ? mousePosition.y += 256 - mousePacket[2] : mousePosition.y -= mousePacket[2];

    if (mousePosition.x < 0) mousePosition.x = 0;
    if (mousePosition.x > basicRenderer.framebuffer->width-1) mousePosition.x = basicRenderer.framebuffer->width-1;

    if (mousePosition.y < 0) mousePosition.y = 0;
    if (mousePosition.y > basicRenderer.framebuffer->height-1) mousePosition.y = basicRenderer.framebuffer->height-1;

    //basicRenderer.Print(toHexString(mousePacket[0]));
//    basicRenderer.Print(toString((uint64_t)xNegative));
//    basicRenderer.Print(" ");
//    basicRenderer.Print(toString((uint64_t)mousePacket[1]));
//    basicRenderer.Print(" ");

    if (mousePacket[0] & PS2_LEFT_BUTTON) basicRenderer.PutPixel(mousePosition.x, mousePosition.y, 0xffffff);
    if (mousePacket[3] & PS2_5TH_BUTTON) basicRenderer.PutPixel(mousePosition.x, mousePosition.y, 0x00ff00);
}

void initPS2Mouse() {
    outb(PS2_COMMAND, ENABLE_MOUSE_COMMAND);
    waitPS2ControllerReady();
    outb(PS2_COMMAND, READ_CONFIG_BYTE_COMMAND);
    waitPS2ControllerInputReady();
    uint8_t status = inb(PS2_DATA);
    status |= 0b10;
    sendPS2Command(WRITE_CONFIG_BYTE_COMMAND, status);

    sendPS2Command(WRITE_MOUSE, 0xF6);
    readPS2();

    sendPS2Command(WRITE_MOUSE, 0xF4);
    readPS2();

    sendPS2Command(WRITE_MOUSE, 0xF2);
    readPS2();

    // Magic for init 4 byte mode https://www.scs.stanford.edu/10wi-cs140/pintos/specs/kbd/scancodes-12.html
    sendPS2Command(WRITE_MOUSE, 0xF3);
    readPS2();
    sendPS2Command(WRITE_MOUSE, 0xC8);
    readPS2();

    sendPS2Command(WRITE_MOUSE, 0xF3);
    readPS2();
    sendPS2Command(WRITE_MOUSE, 0x64);
    readPS2();

    sendPS2Command(WRITE_MOUSE, 0xF3);
    readPS2();
    sendPS2Command(WRITE_MOUSE, 0x50);
    readPS2();

    sendPS2Command(WRITE_MOUSE, 0xF2);
    readPS2();
    mouseMode = readPS2();
}