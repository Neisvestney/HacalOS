#ifndef MODULETOS_EFIRUNTIMESERVICES_H
#define MODULETOS_EFIRUNTIMESERVICES_H

#include "efi.h"
#include "efiMemory.h"

namespace EFI {
    struct Time {
        uint16_t Year;       // 1998 - 20XX
        uint8_t Month;      // 1 - 12
        uint8_t Day;        // 1 - 31
        uint8_t Hour;       // 0 - 23
        uint8_t Minute;     // 0 - 59
        uint8_t Second;     // 0 - 59
        uint8_t Pad1;
        uint32_t Nanosecond; // 0 - 999,999,999
        uint16_t TimeZone;   // -1440 to 1440 or 2047
        uint8_t Daylight;
        uint8_t Pad2;
    };

    struct TimeCapabilities {
        uint32_t Resolution;     // 1e-6 parts per million
        uint32_t Accuracy;       // hertz
        bool SetsToZero;     // Set clears sub-second time
    };

#define EFI_VARIABLE_NON_VOLATILE                          0x00000001
#define EFI_VARIABLE_BOOTSERVICE_ACCESS                    0x00000002
#define EFI_VARIABLE_RUNTIME_ACCESS                        0x00000004
#define EFI_VARIABLE_HARDWARE_ERROR_RECORD                 0x00000008
#define EFI_VARIABLE_AUTHENTICATED_WRITE_ACCESS            0x00000010
#define EFI_VARIABLE_TIME_BASED_AUTHENTICATED_WRITE_ACCESS 0x00000020
#define EFI_VARIABLE_APPEND_WRITE                          0x00000040

    enum ResetType {
        EfiResetCold,
        EfiResetWarm,
        EfiResetShutdown
    };

    struct CapsuleHeader {
        EfiGuid CapsuleGuid;
        uint32_t HeaderSize;
        uint32_t Flags;
        uint32_t CapsuleImageSize;
    };

    struct RuntimeServices {
        EFITableHeader Hdr;

        __attribute__((ms_abi)) uint64_t (*GetTime)(Time *, TimeCapabilities *);

        __attribute__((ms_abi)) uint64_t (*SetTime)(Time *);

        __attribute__((ms_abi)) uint64_t (*GetWakeupTime)(Time *);

        __attribute__((ms_abi)) uint64_t (*SetWakeupTime)(bool enable, Time *);

        __attribute__((ms_abi)) uint64_t (*SetVirtualAddressMap)(
                uint64_t memoryMapSize,
                uint64_t descriptorSize,
                uint32_t descriptorVersion,
                EFI::MemoryDescriptor *descriptor
        );

        __attribute__((ms_abi)) uint64_t (*ConvertPointer)(uint64_t debugDisposition, void **address);

        __attribute__((ms_abi)) uint64_t (*GetVariable)(
                const wchar_t *variableName,
                EfiGuid *vendorGuid,
                uint32_t attributes,
                uint64_t dataSize,
                void *data
        );

        __attribute__((ms_abi)) uint64_t (*GetNextVariableName)(
                uint64_t *variableNameSize,
                const wchar_t *variableName,
                EfiGuid *vendorGuid
        );

        __attribute__((ms_abi)) uint64_t (*SetVariable)(
                const wchar_t *variableName,
                EfiGuid *vendorGuid,
                uint32_t attributes,
                uint64_t dataSize,
                void *data
        );

        __attribute__((ms_abi)) uint64_t (*GetNextHighMonoCount)(
                uint32_t *highCount
        );

        __attribute__((ms_abi)) uint64_t (*ResetSystem)(
                ResetType resetType,
                uint64_t resetStatus,
                uint64_t dataSize,
                char16_t *resetData
        );

        __attribute__((ms_abi)) uint64_t (*UpdateCapsule)(
                CapsuleHeader **capsuleHeaderArray,
                uint64_t capsuleCount,
                uint64_t dataSize,
                uint64_t scatterGatherList
        );

        __attribute__((ms_abi)) uint64_t (*QueryCapsuleCapabilities)(
                CapsuleHeader **capsuleHeaderArray,
                uint64_t capsuleCount,
                uint64_t maximumCapsuleSize,
                ResetType resetType
        );

        __attribute__((ms_abi)) uint64_t (*QueryVariableInfo)(
                uint32_t attributes,
                uint64_t *maximumVariableStorageSize,
                uint64_t *remainingVariableStorageSize,
                uint64_t *maximumVariableSize
        );
    };

    extern RuntimeServices *runtimeServices;

    double Timestamp();
}

#endif //MODULETOS_EFIRUNTIMESERVICES_H
