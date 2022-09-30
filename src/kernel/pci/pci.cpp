#include "pci.h"

namespace PCI{

    void EnumerateFunction(uint64_t deviceAddress, uint64_t function){
        uint64_t offset = function << 12;

        uint64_t functionAddress = deviceAddress + offset;
        kernelPageTableManager.MapMemory((void*)functionAddress, (void*)functionAddress);

        PCIDeviceHeader* pciDeviceHeader = (PCIDeviceHeader*)functionAddress;

        if (pciDeviceHeader->deviceId == 0) return;
        if (pciDeviceHeader->deviceId == 0xFFFF) return;

        basicRenderer.Print(GetVendorName(pciDeviceHeader->vendorId));
        basicRenderer.Print(" / ");
        basicRenderer.Print(GetDeviceName(pciDeviceHeader->vendorId, pciDeviceHeader->deviceId));
        basicRenderer.Print(" / ");
        basicRenderer.Print(deviceClasses[pciDeviceHeader->deviceClass]);
        basicRenderer.Print(" / ");
        basicRenderer.Print(GetSubclassName(pciDeviceHeader->deviceClass, pciDeviceHeader->subclass));
        basicRenderer.Print(" / ");
        basicRenderer.Print(GetProgIFName(pciDeviceHeader->deviceClass, pciDeviceHeader->subclass, pciDeviceHeader->progIF));
        basicRenderer.NextLine();

    }

    void EnumerateDevice(uint64_t busAddress, uint64_t device){
        uint64_t offset = device << 15;

        uint64_t deviceAddress = busAddress + offset;
        kernelPageTableManager.MapMemory((void*)deviceAddress, (void*)deviceAddress);

        PCIDeviceHeader* pciDeviceHeader = (PCIDeviceHeader*)deviceAddress;

        if (pciDeviceHeader->deviceId == 0) return;
        if (pciDeviceHeader->deviceId == 0xFFFF) return;

        for (uint64_t function = 0; function < 8; function++){
            EnumerateFunction(deviceAddress, function);
        }
    }

    void EnumerateBus(uint64_t baseAddress, uint64_t bus){
        uint64_t offset = bus << 20;

        uint64_t busAddress = baseAddress + offset;
        kernelPageTableManager.MapMemory((void*)busAddress, (void*)busAddress);

        PCIDeviceHeader* pciDeviceHeader = (PCIDeviceHeader*)busAddress;

        if (pciDeviceHeader->deviceId == 0) return;
        if (pciDeviceHeader->deviceId == 0xFFFF) return;

        for (uint64_t device = 0; device < 32; device++){
            EnumerateDevice(busAddress, device);
        }
    }

    void EnumeratePCI(ACPI::MCFGHeader* mcfg){
        int entries = ((mcfg->header.length) - sizeof(ACPI::MCFGHeader)) / sizeof(ACPI::DeviceConfig);
        for (int t = 0; t < entries; t++){
            ACPI::DeviceConfig* newDeviceConfig = (ACPI::DeviceConfig*)((uint64_t)mcfg + sizeof(ACPI::MCFGHeader) + (sizeof(ACPI::DeviceConfig) * t));
            for (uint64_t bus = newDeviceConfig->startBus; bus < newDeviceConfig->endBus; bus++){
                EnumerateBus(newDeviceConfig->baseAddress, bus);
            }
        }
    }
}
