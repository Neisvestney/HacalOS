#include "acpi.h"

namespace ACPI {

    void *FindTable(ACPI::SDTHeader *sdtHeader, char *signature) {

        int entries = (sdtHeader->length - sizeof(ACPI::SDTHeader)) / 8;

        for (int t = 0; t < entries; t++) {
            ACPI::SDTHeader *newSDTHeader = (ACPI::SDTHeader *) *(uint64_t * )((uint64_t) sdtHeader + sizeof(ACPI::SDTHeader) + (t * 8));
            for (int i = 0; i < 4; i++) {
                if (newSDTHeader->signature[i] != signature[i]) {
                    break;
                }
                if (i == 3) return newSDTHeader;
            }
        }
        return 0;
    }

}