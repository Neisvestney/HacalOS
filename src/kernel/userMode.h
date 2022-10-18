#ifndef HACALOS_USERMODE_H
#define HACALOS_USERMODE_H

#include <stdint.h>

extern "C" void goToUserMode(void(*address)());

#endif //HACALOS_USERMODE_H
