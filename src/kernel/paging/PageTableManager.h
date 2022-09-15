#ifndef MODULETOS_PAGETABLEMANAGER_H
#define MODULETOS_PAGETABLEMANAGER_H

#include "paging.h"

class PageTableManager {
public:
    PageTableManager(PageTable* PML4Address);
    PageTable* PML4;
    void MapMemory(void* virtualMemory, void* physicalMemory);
};

#endif //MODULETOS_PAGETABLEMANAGER_H
