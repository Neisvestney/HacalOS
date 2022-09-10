#include "cstr.h"

char uintToStringBuffer[128];
const char* toString(uint64_t value) {
    uint8_t size = 0;
    uint16_t sizeTest = value;
    while (sizeTest / 10 > 0) {
        sizeTest /= 10;
        size++;
    }

    uint8_t index = 0;
    while (value / 10 > 0) {
        uint8_t remainder = value % 10;
        value /= 10;
        uintToStringBuffer[size - index] = remainder + '0';
        index++;
    }
    uint8_t remainder = value % 10;
    uintToStringBuffer[size - index] = remainder + '0';
    uintToStringBuffer[size + 1] = 0;

    return uintToStringBuffer;
}

char intToStringBuffer[128];
const char *toString(int64_t value) {
    uint8_t isNegative = 0;
    if (value < 0) {
        isNegative = 1;
        value *= -1;
        intToStringBuffer[0] = '-';
    }

    uint8_t size = 0;
    uint16_t sizeTest = value;
    while (sizeTest / 10 > 0) {
        sizeTest /= 10;
        size++;
    }

    uint8_t index = 0;
    while (value / 10 > 0) {
        uint8_t remainder = value % 10;
        value /= 10;
        intToStringBuffer[isNegative + size - index] = remainder + '0';
        index++;
    }
    uint8_t remainder = value % 10;
    intToStringBuffer[isNegative + size - index] = remainder + '0';
    intToStringBuffer[isNegative + size + 1] = 0;

    return intToStringBuffer;
}

char doubleToStringBuffer[128];
const char* toString(double value, uint8_t decimalPlaces) {
    char* intPtr = (char*) toString((int64_t)value);
    char* doublePtr = doubleToStringBuffer;

    if (value < 0) value *= -1;

    while (*intPtr != 0) {
        *doublePtr = *intPtr;
        intPtr++;
        doublePtr++;
    }

    *doublePtr = '.';
    doublePtr++;

    double newValue = value - (int)value;
    for (uint8_t i = 0; i < decimalPlaces; ++i) {
        newValue *= 10;
        *doublePtr = (int)newValue + '0';
        newValue -= (int)newValue;
        doublePtr++;
    }

    *doublePtr = 0;
    return doubleToStringBuffer;
}

const char* toString(double value) {
    return toString(value, 2);
}

char hexToStringBuffer[128];
const char *toHexStringBase(uint64_t value, uint8_t size) {
    uint64_t* valPtr = &value;
    uint8_t* ptr;
    uint8_t tmp;
    for (uint8_t i = 0; i < size; ++i) {
        ptr = ((uint8_t*)valPtr + i);
        tmp = ((*ptr & 0xF0) >> 4);
        hexToStringBuffer[size - (i * 2 + 1)] = tmp + (tmp > 9 ? 55 : '0');
        tmp = ((*ptr & 0x0F));
        hexToStringBuffer[size - (i * 2 + 0)] = tmp + (tmp > 9 ? 55 : '0');
    }
    hexToStringBuffer[size + 1] = 0;

    return hexToStringBuffer;
}

const char *toHexString(uint64_t value) {
    return toHexStringBase(value, 8 * 2 -1);
}

const char *toHexString(uint32_t value) {
    return toHexStringBase(value, 4 * 2 -1);
}

const char *toHexString(uint16_t value) {
    return toHexStringBase(value, 2 * 2 -1);
}

const char *toHexString(uint8_t value) {
    return toHexStringBase(value, 1 * 2 -1);
}
