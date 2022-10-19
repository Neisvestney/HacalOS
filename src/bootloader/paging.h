#ifndef HACALOS_PAGING_H
#define HACALOS_PAGING_H

#include <stdint.h>

enum PT_Flag {
    Present = 0,
    ReadWrite = 1,
    UserSuper = 2,
    WriteThrough = 3,
    CacheDisabled = 4,
    Accessed = 5,
    LargerPages = 7,
    Custom0 = 9,
    Custom1 = 10,
    Custom2 = 11,
    NX = 63 // only if supported
};

typedef struct {
    uint64_t value;
} PageDirectoryEntry;

struct PageTable {
    PageDirectoryEntry entries [512];
}__attribute__((aligned(0x1000)));

void SetFlag(PageDirectoryEntry* entry, enum PT_Flag flag, bool enabled);
bool GetFlag(PageDirectoryEntry* entry, enum PT_Flag flag);
void SetAddress(PageDirectoryEntry* entry, uint64_t address);
uint64_t GetAddress(PageDirectoryEntry* entry);

struct PageTable *ReplaceTable();
void MapMemory(struct PageTable *PML4, void *virtualMemory, void *physicalMemory);

#endif //HACALOS_PAGING_H
