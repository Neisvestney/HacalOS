#include <stdbool.h>
#include "paging.h"
#include "../gnu-efi/inc/efi.h"
#include "../gnu-efi/inc/efilib.h"

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

struct PageTable *ReplaceTable() {
    uint64_t PML4Address = 0;
    asm(
            "mov %%cr3, %%rax\n\t"
            "mov %%rax, %0\n\t"
            :"=m" (PML4Address)
            : /* no input */
            : "%rax"
            );
    struct PageTable *PML4 = (struct PageTable *) PML4Address;

    struct PageTable *NewPML4;
    uefi_call_wrapper(ST->BootServices->AllocatePages, 4, AllocateAnyPages, EfiLoaderData, 1, (EFI_PHYSICAL_ADDRESS*)&NewPML4);
    CopyMem(NewPML4, PML4, 4096);
    asm ("mov %0, %%cr3" : : "r" (NewPML4));

    return NewPML4;
}

void MapMemory(struct PageTable *PML4, void *virtualMemory, void *physicalMemory) {
    uint64_t virtualAddress = (uint64_t) virtualMemory;
    virtualAddress = (virtualAddress >> 12);
    uint64_t P_i = (virtualAddress & 0x1ff);
    virtualAddress = (virtualAddress >> 9);
    uint64_t PT_i = (virtualAddress & 0x1ff);
    virtualAddress = (virtualAddress >> 9);
    uint64_t PD_i = (virtualAddress & 0x1ff);
    virtualAddress = (virtualAddress >> 9);
    uint64_t PDP_i = (virtualAddress & 0x1ff);

//    Print(L"%d\n", P_i);
//    Print(L"%d\n", PT_i);
//    Print(L"%d\n", PD_i);
//    Print(L"%d\n", PDP_i);

    PageDirectoryEntry PDE;

    PDE = PML4->entries[PDP_i];
    struct PageTable* PDP;
    if (!GetFlag(&PDE, Present)){
        PDP = (struct PageTable*)NULL;
        EFI_STATUS status = uefi_call_wrapper(ST->BootServices->AllocatePages, 4, AllocateAnyPages, EfiLoaderData, 1, (EFI_PHYSICAL_ADDRESS*)&PDP);
        SetMem(PDP, 4096, 0);
        SetAddress(&PDE, (uint64_t)PDP >> 12);
        SetFlag(&PDE, Present, true);
        SetFlag(&PDE, ReadWrite, true);
        SetFlag(&PDE, UserSuper, false);
        PML4->entries[PDP_i] = PDE;
    }
    else
    {
        PDP = (struct PageTable*)((uint64_t)GetAddress(&PDE) << 12);
    }


    PDE = PDP->entries[PD_i];
    struct PageTable* PD;
    if (!GetFlag(&PDE, Present)){
        PD = (struct PageTable*)NULL;
        EFI_STATUS status = uefi_call_wrapper(ST->BootServices->AllocatePages, 4, AllocateAnyPages, EfiLoaderData, 1, (EFI_PHYSICAL_ADDRESS*)&PD);
        SetMem(PD, 4096, 0);
        SetAddress(&PDE, (uint64_t)PD >> 12);
        SetFlag(&PDE, Present, true);
        SetFlag(&PDE, ReadWrite, true);
        SetFlag(&PDE, UserSuper, false);
        PDP->entries[PD_i] = PDE;
    }
    else
    {
        PD = (struct PageTable*)((uint64_t)GetAddress(&PDE) << 12);
    }

    PDE = PD->entries[PT_i];
    struct PageTable* PT;
    if (!GetFlag(&PDE, Present)){
        PT = (struct PageTable*)NULL;
        EFI_STATUS status = uefi_call_wrapper(ST->BootServices->AllocatePages, 4, AllocateAnyPages, EfiLoaderData, 1, (EFI_PHYSICAL_ADDRESS*)&PT);
        SetMem(PT, 4096, 0);
        SetAddress(&PDE, (uint64_t)PT >> 12);
        SetFlag(&PDE, Present, true);
        SetFlag(&PDE, ReadWrite, true);
        SetFlag(&PDE, UserSuper, false);
        PD->entries[PT_i] = PDE;
    }
    else
    {
        PT = (struct PageTable*)((uint64_t)GetAddress(&PDE) << 12);
    }

    PDE = PT->entries[P_i];
    SetAddress(&PDE, (uint64_t)physicalMemory >> 12);
    SetFlag(&PDE, Present, true);
    SetFlag(&PDE, ReadWrite, true);
    SetFlag(&PDE, UserSuper, false);
    PT->entries[P_i] = PDE;
}
