#ifndef MODULETOS_PCI_H
#define MODULETOS_PCI_H

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

    void EnumeratePCI(ACPI::MCFGHeader* mcfg);

    extern const char* deviceClasses[];

    const char* GetVendorName(uint16_t vendorID);
    const char* GetDeviceName(uint16_t vendorID, uint16_t deviceID);
    const char* GetSubclassName(uint8_t classCode, uint8_t subclassCode);
    const char* GetProgIFName(uint8_t classCode, uint8_t subclassCode, uint8_t progIF);
}
#endif //MODULETOS_PCI_H
