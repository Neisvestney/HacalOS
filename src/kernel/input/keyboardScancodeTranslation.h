#ifndef HACALOS_KEYBOARDSCANCODETRANSLATION_H
#define HACALOS_KEYBOARDSCANCODETRANSLATION_H

#include <stdint.h>

namespace QWERTYKeyboard {

#define LeftShift 0x2A
#define RightShift 0x36
#define Enter 0x1C
#define BackSpace 0x0E
#define Spacebar 0x39
#define CapsLock 0x3A

    extern const char asciiTable[];
    extern const char shiftAsciiTable[];

    char Translate(uint8_t scancode, bool uppercase, bool shiftPressed);
}

#endif //HACALOS_KEYBOARDSCANCODETRANSLATION_H
