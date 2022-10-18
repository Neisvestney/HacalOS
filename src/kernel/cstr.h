#ifndef HACALOS_CSTR_H
#define HACALOS_CSTR_H

#include <stdint.h>

const char* toString(uint64_t value);
const char* toString(int64_t value);
const char* toString(double value, uint8_t decimalPlaces);
const char* toString(double value);

const char* toHexString(uint64_t value);
const char* toHexString(uint32_t value);
const char* toHexString(uint16_t value);
const char* toHexString(uint8_t value);

#endif //HACALOS_CSTR_H
