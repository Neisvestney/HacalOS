#ifndef HACALOS_PAGEFRAMEALLOCATOR_H
#define HACALOS_PAGEFRAMEALLOCATOR_H

#include <stdint.h>
#include "../Bitmap.h"
#include "../efi/efiMemory.h"
#include "../bootinfo.h"
#include "../memory.h"

class PageFrameAllocator {
public:
    void ReadEFIMemoryMap(EFI::MemoryMap* mMap);
    Bitmap pageBitmap;
    uint64_t pageBitmapIndex {0};
    void FreePage(void* address);
    void FreePages(void* address, uint64_t pageCount);
    void LockPage(void* address);
    void LockPages(void* address, uint64_t pageCount);
    void* RequestPage(bool clearPage = true);
    void ReservePage(void* address);
    void ReservePages(void* address, uint64_t pageCount);
    uint64_t GetFreeRAM();
    uint64_t GetUsedRAM();
    uint64_t GetReservedRAM();

private:
    void InitBitmap(size_t bitmapSize, void* bufferAddress);
    void UnreservePage(void* address);
    void UnreservePages(void* address, uint64_t pageCount);
    uint64_t freeMemory = 0;
    uint64_t reservedMemory = 0;
    uint64_t usedMemory = 0;
    bool Initialized = false;
};

extern PageFrameAllocator globalPageFrameAllocator;

#endif //HACALOS_PAGEFRAMEALLOCATOR_H
