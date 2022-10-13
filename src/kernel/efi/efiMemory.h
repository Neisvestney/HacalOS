#ifndef MODULETOS_EFIMEMORY_H
#define MODULETOS_EFIMEMORY_H

#include <stdint.h>

namespace EFI {

    struct MemoryDescriptor {
        uint32_t type;
        void *physAddr;
        void *virtAddr;
        uint64_t numPages;
        uint64_t attribs;
    };

    extern const char *EFI_MEMORY_TYPE_STRINGS[];

    struct MemoryMap {
        MemoryDescriptor *map;
        uint64_t mapSize;
        uint64_t descriptorSize;
    };

    uint64_t GetMemorySize(MemoryMap *mMap);

}

#endif //MODULETOS_EFIMEMORY_H
