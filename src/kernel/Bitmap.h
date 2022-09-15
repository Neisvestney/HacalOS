#ifndef MODULETOS_BITMAP_H
#define MODULETOS_BITMAP_H

#include <stddef.h>
#include <stdint.h>

class Bitmap {
public:
    size_t size;
    uint8_t* buffer;
    bool operator[](uint64_t index);
    bool Set(uint64_t index, bool value);
};


#endif //MODULETOS_BITMAP_H
