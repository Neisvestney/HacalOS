#ifndef MODULETOS_EFI_H
#define MODULETOS_EFI_H

#include "stdint.h"

//#define uefi_call_wrapper(func,va_num,...)                        \
//  __VA_ARG_NSUFFIX__(_cast64_efi_call, __VA_ARGS__) (func , ##__VA_ARGS__)

namespace EFI {

    struct EFITableHeader {
        uint64_t Signature;
        uint32_t Revision;
        uint32_t HeaderSize;
        uint32_t CRC32;
        uint32_t Reserved;
    };

    struct EfiGuid {
        uint32_t Data1;
        uint16_t Data2;
        uint16_t Data3;
        uint8_t  Data4[8];
    };

#define EFI_MODULETOS_GUID \
   { 0x80b2538b, 0x948e, 0x4bf2, {0x68, 0x61, 0x63, 0x61, 0x6C, 0x6F, 0x73, 0x00} }
}

#endif //MODULETOS_EFI_H
