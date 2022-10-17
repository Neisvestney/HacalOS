#include <stdbool.h>
#include "paging.h"

void SetFlag(PageDirectoryEntry* entry, enum PT_Flag flag, bool enabled){
    uint64_t bitSelector = (uint64_t)1 << flag;
    entry->value &= ~bitSelector;
    if (enabled){
        entry->value |= bitSelector;
    }
}

bool GetFlag(PageDirectoryEntry* entry, enum PT_Flag flag){
    uint64_t bitSelector = (uint64_t)1 << flag;
    return ((entry->value) & bitSelector) > 0 ? true : false;
}

uint64_t GetAddress(PageDirectoryEntry* entry){
    return (entry->value & 0x000ffffffffff000) >> 12;
}

void SetAddress(PageDirectoryEntry* entry, uint64_t address){
    address &= 0x000000ffffffffff;
    entry->value &= 0xfff0000000000fff;
    entry->value |= (address << 12);
}