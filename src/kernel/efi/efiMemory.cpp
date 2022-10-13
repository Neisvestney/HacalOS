#include "efiMemory.h"

namespace EFI {

    const char *EFI_MEMORY_TYPE_STRINGS[]{
            "EfiReservedMemoryType",
            "EfiLoaderCode",
            "EfiLoaderData",
            "EfiBootServicesCode",
            "EfiBootServicesData",
            "EfiRuntimeServicesCode",
            "EfiRuntimeServicesData",
            "EfiConventionalMemory",
            "EfiUnusableMemory",
            "EfiACPIReclaimMemory",
            "EfiACPIMemoryNVS",
            "EfiMemoryMappedIO",
            "EfiMemoryMappedIOPortSpace",
            "EfiPalCode",
            "EfiPersistentMemory",
            "EfiMaxMemoryType"
    };

    uint64_t stats[16] = {0};

    uint64_t GetMemorySize(MemoryMap *mMap) {
        static uint64_t memorySizeBytes = 0;
        if (memorySizeBytes > 0) return memorySizeBytes;

        for (int i = 0; i < mMap->mapSize / mMap->descriptorSize; i++) {
            MemoryDescriptor *desc = (MemoryDescriptor *) ((uint64_t) mMap->map + (i * mMap->descriptorSize));
            memorySizeBytes += desc->numPages * 4096;
            stats[desc->type] += desc->numPages;
        }

        return memorySizeBytes;
    }

}