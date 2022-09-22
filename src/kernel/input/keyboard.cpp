#include "keyboard.h"
#include "../io.h"
#include "ps2.h"


bool isLeftShiftPressed;
bool isRightShiftPressed;
bool caps;

void handleKeyboard(uint8_t scancode){
    switch (scancode){
        case LeftShift:
            isLeftShiftPressed = true;
            return;
        case LeftShift + 0x80:
            isLeftShiftPressed = false;
            return;
        case RightShift:
            isRightShiftPressed = true;
            return;
        case RightShift + 0x80:
            isRightShiftPressed = false;
            return;
        case CapsLock:
            caps = !caps;
            SetLED(false, false, caps);
            return;
        case Enter:
            basicRenderer.NextLine();
            return;
        case Spacebar:
            basicRenderer.Print(" ");
            return;
        case BackSpace:
            basicRenderer.ClearChar();
            return;
    }

    char ascii = QWERTYKeyboard::Translate(scancode, (isLeftShiftPressed || isRightShiftPressed) ^ caps, isLeftShiftPressed || isRightShiftPressed);

    if (ascii != 0){
        basicRenderer.PutChar(ascii);
    }
}

void SetLED(bool scrollLock, bool numLock, bool capsLock) {
    uint8_t data = 0;
    data |= scrollLock ? 0b10000000 : 0;
    data |= numLock    ? 0b01000000 : 0;
    data |= capsLock   ? 0b00100000 : 0;
    sendPS2Command(SET_LED_COMMAND, data);
}