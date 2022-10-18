#ifndef HACALOS_PCI_H
#define HACALOS_PCI_H

#include <stdint.h>
#include "../acpi.h"
#include "../paging/PageTableManager.h"
#include "../BasicRenderer.h"
#include "../cstr.h"

namespace PCI{
    struct PCIDeviceHeader{
        uint16_t vendorId;
        uint16_t deviceId;
        uint16_t command;
        uint16_t status;
        uint8_t revisionId;
        uint8_t progIF;
        uint8_t subclass;
        uint8_t deviceClass;
        uint8_t cacheLineSize;
        uint8_t latencyTimer;
        uint8_t headerType;
        uint8_t bist;
    };

    struct PCIDeviceHeader0 {
        PCIDeviceHeader header;
        uint32_t bar0;
        uint32_t bar1;
        uint32_t bar2;
        uint32_t bar3;
        uint32_t bar4;
        uint32_t bar5;
        uint32_t cardbusCisPtr;
        uint16_t subsystemVendorId;
        uint16_t subsystemId;
        uint32_t expansionRomBaseAddr;
        uint8_t capabilitiesPtr;
        uint8_t rsv0;
        uint16_t rsv1;
        uint32_t rsv2;
        uint8_t interruptLine;
        uint8_t interruptPin;
        uint8_t minGrant;
        uint8_t maxLatency;
    };

    void EnumeratePCI(ACPI::MCFGHeader* mcfg);

    extern const char* deviceClasses[];

    const char* GetVendorName(uint16_t vendorID);
    const char* GetDeviceName(uint16_t vendorID, uint16_t deviceID);
    const char* GetSubclassName(uint8_t classCode, uint8_t subclassCode);
    const char* GetProgIFName(uint8_t classCode, uint8_t subclassCode, uint8_t progIF);
}
#endif //HACALOS_PCI_H
