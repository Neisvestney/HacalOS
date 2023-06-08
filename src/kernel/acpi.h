#ifndef HACALOS_ACPI_H
#define HACALOS_ACPI_H

#include <stdint.h>

namespace ACPI {
    struct SDTHeader {
        unsigned char signature[4];
        uint32_t length;
        uint8_t revision;
        uint8_t checksum;
        uint8_t oemId[6];
        uint8_t oemTableId[8];
        uint32_t oemRevision;
        uint32_t creatorId;
        uint32_t creatorRevision;
    }__attribute__((packed));

    struct RSDP2 {
        unsigned char signature[8];
        uint8_t checksum;
        uint8_t oemId[6];
        uint8_t revision;
        uint32_t rsdtAddress;
        uint32_t length;
        SDTHeader* xsdtAddress;
        uint8_t extendedChecksum;
        uint8_t reserved[3];
    } __attribute__((packed));

    struct MCFGHeader {
        SDTHeader header;
        uint64_t reserved;
    } __attribute__((packed));

    struct DeviceConfig {
        uint64_t baseAddress;
        uint16_t pciSegGroup;
        uint8_t startBus;
        uint8_t endBus;
        uint32_t reserved;
    }__attribute__((packed));

    // Multiple APIC Description Table
    struct MADTHeader {
        SDTHeader header;
        uint32_t localAPICAddress;
        uint32_t flags;
    } __attribute__((packed));

    struct MADTEntryHeader {
        uint8_t type;
        uint8_t length;
    } __attribute__((packed));

    struct MADTEntryLocalAPIC {
        MADTEntryHeader header;
        uint8_t processorID;
        uint8_t apicID;
        uint32_t flags;
    } __attribute__((packed));

    void *FindTable(SDTHeader *sdtHeader, char *signature);
}
#endif //HACALOS_ACPI_H
