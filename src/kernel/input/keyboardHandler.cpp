#include "keyboardHandler.h"
#include "../io.h"


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
