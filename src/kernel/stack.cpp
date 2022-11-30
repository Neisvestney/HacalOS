#include "stack.h"
#include "paging/PageTableManager.h"
#include "paging/PageFrameAllocator.h"

void initializeStack(void* stackAddress, size_t pagesCount) {
    void* pos = (void*)((size_t)stackAddress - 0x1000);

    for (size_t i = 0; i < pagesCount; ++i) {
        kernelPageTableManager.MapMemory(pos, globalPageFrameAllocator.RequestPage());
        pos = (void*)((size_t)pos - 0x1000);
    }
}