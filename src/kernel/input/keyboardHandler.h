#ifndef MODULETOS_KEYBOARDHANDLER_H
#define MODULETOS_KEYBOARDHANDLER_H

#include <stdint.h>
#include "keyboardScancodeTranslation.h"
#include "../BasicRenderer.h"

#define PS2_DATA 0x60
#define PS2_COMMAND 0x64

void handleKeyboard(uint8_t scancode);

#endif //MODULETOS_KEYBOARDHANDLER_H
