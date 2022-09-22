#ifndef MODULETOS_KEYBOARD_H
#define MODULETOS_KEYBOARD_H

#include <stdint.h>
#include "keyboardScancodeTranslation.h"
#include "../BasicRenderer.h"

#define SET_LED_COMMAND 0xED

void handleKeyboard(uint8_t scancode);
void SetLED(bool scrollLock, bool numLock, bool capsLock);

#endif //MODULETOS_KEYBOARD_H
