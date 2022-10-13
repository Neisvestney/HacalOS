//#include <efi.h>
//#include <efilib.h>

#include <elf.h>
#include "../gnu-efi/inc/efi.h"
#include "../gnu-efi/inc/efilib.h"
#include "bootinfo.h"

typedef unsigned long long size_t;

Framebuffer framebuffer;
MemoryMap memoryMap;
BootInfo bootInfo;

EFI_FILE* LoadFile(EFI_FILE* Directory, CHAR16* Path, EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE* SystemTable){
    EFI_FILE* LoadedFile;

    EFI_LOADED_IMAGE_PROTOCOL* LoadedImage;
    uefi_call_wrapper(SystemTable->BootServices->HandleProtocol, 3, ImageHandle, &gEfiLoadedImageProtocolGuid, (void**)&LoadedImage);

    EFI_SIMPLE_FILE_SYSTEM_PROTOCOL* FileSystem;
    uefi_call_wrapper(SystemTable->BootServices->HandleProtocol, 3, LoadedImage->DeviceHandle, &gEfiSimpleFileSystemProtocolGuid, (void**)&FileSystem);

    if (Directory == NULL){
        uefi_call_wrapper(FileSystem->OpenVolume, 2, FileSystem, &Directory);
    }

    EFI_STATUS s = uefi_call_wrapper(Directory->Open, 5, Directory, &LoadedFile, Path, EFI_FILE_MODE_READ, EFI_FILE_READ_ONLY);
    if (s != EFI_SUCCESS){
        return NULL;
    }
    return LoadedFile;
}

PSF1Font* LoadPSF1Font(EFI_FILE* Directory, CHAR16* Path, EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE* SystemTable) {
    EFI_FILE* font = LoadFile(Directory, Path, ImageHandle, SystemTable);
    if (font == NULL) return NULL;

    PSF1Header* fontHeader;
    uefi_call_wrapper(SystemTable->BootServices->AllocatePool, 3, EfiLoaderData, sizeof(PSF1Header), (void**)&fontHeader);
    UINTN size = sizeof(PSF1Header);
    uefi_call_wrapper(font->Read, 3, font, &size, fontHeader);
    if (fontHeader->magic[0] != PSF1_MAGIC0 || fontHeader->magic[1] != PSF1_MAGIC1) return NULL;

    UINTN glyphBufferSize = fontHeader->charSize * 256;
    if (fontHeader->mode == 1) glyphBufferSize = fontHeader->charSize * 512;
    if (fontHeader->mode == 3) glyphBufferSize = fontHeader->charSize * 512;

    void* glyphBuffer;
    uefi_call_wrapper(font->SetPosition, 2, font, sizeof(PSF1Header));
    uefi_call_wrapper(SystemTable->BootServices->AllocatePool, 3, EfiLoaderData, glyphBufferSize, (void**)&glyphBuffer);
    uefi_call_wrapper(font->Read, 3, font, &glyphBufferSize, glyphBuffer);

    PSF1Font* finishedFont;
    uefi_call_wrapper(SystemTable->BootServices->AllocatePool, 3, EfiLoaderData, sizeof(PSF1Font), (void**)&finishedFont);
    finishedFont->psf1Header = fontHeader;
    finishedFont->glyphBuffer = glyphBuffer;

    return finishedFont;
}

int memcmp(const void* aptr, const void* bptr, size_t n){
    const unsigned char* a = aptr, *b = bptr;
    for (size_t i = 0; i < n; i++){
        if (a[i] < b[i]) return -1;
        else if (a[i] > b[i]) return 1;
    }
    return 0;
}

UINTN strcmp(CHAR8* a, CHAR8* b, UINTN length){
    for (UINTN i = 0; i < length; i++){
        if (*a != *b) return 0;
        a++;
        b++;
    }
    return 1;
}

EFI_STATUS efi_main (EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE *SystemTable)
{
    InitializeLib(ImageHandle, SystemTable);

    ST = SystemTable;

    //#region GOP Setup
    Print(L"(Bootloader) Gop setup");

    EFI_STATUS status;
    EFI_GUID gopGuid = EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID;
    EFI_GRAPHICS_OUTPUT_PROTOCOL *gop;

    status = uefi_call_wrapper(BS->LocateProtocol, 3, &gopGuid, NULL, (void**)&gop);
    if(EFI_ERROR(status))
        Print(L" [WARN] Unable to locate GOP");

    EFI_GRAPHICS_OUTPUT_MODE_INFORMATION *info;
    UINTN SizeOfInfo, numModes, nativeMode;

    uefi_call_wrapper(gop->QueryMode, 4, gop, gop->Mode==NULL?0:gop->Mode->Mode, &SizeOfInfo, &info);
    // this is needed to get the current video mode
    if (status == EFI_NOT_STARTED)
        status = uefi_call_wrapper(gop->SetMode, 2, gop, 0);
    if(EFI_ERROR(status)) {
        Print(L" [WARN] Unable to get native mode");
    } else {
        nativeMode = gop->Mode->Mode;
        numModes = gop->Mode->MaxMode;
    }

    framebuffer.baseAddress = (void*)gop->Mode->FrameBufferBase;
    framebuffer.bufferSize = gop->Mode->FrameBufferSize;
    framebuffer.width = gop->Mode->Info->HorizontalResolution;
    framebuffer.height = gop->Mode->Info->VerticalResolution;
    framebuffer.pixelsPerScanline = gop->Mode->Info->PixelsPerScanLine;

//    for (int i = 0; i < numModes; i++) {
//        status = uefi_call_wrapper(gop->QueryMode, 4, gop, i, &SizeOfInfo, &info);
//        Print(L"mode %03d width %d height %d format %x%s\n",
//              i,
//              info->HorizontalResolution,
//              info->VerticalResolution,
//              info->PixelFormat,
//              i == nativeMode ? "(current)" : ""
//        );
//    }

    Print(L" [OK]\n");
    //#endregion

    //#region Load Font
    Print(L"(Bootloader) Loading Font");

    PSF1Font* font = LoadPSF1Font(NULL, L"zap-ext-light16.psf", ImageHandle, SystemTable);
    if (font == NULL) {
        Print(L" [ERR]\n");
    } else {
        Print(L" [OK]\n");
    }
    //#endregion

    //#region Load Kernel
    Print(L"(Bootloader) Loading ModuletOS Kernel");

    EFI_FILE* Kernel = LoadFile(NULL, L"kernel.elf", ImageHandle, SystemTable);
    if (Kernel == NULL){
        Print(L" [ERR]\n");
        Print(L"(Bootloader) Could not load kernel\n");
        return EFI_LOAD_ERROR;
    }

    Elf64_Ehdr header;
    {
        UINTN FileInfoSize;
        EFI_FILE_INFO* FileInfo;
        uefi_call_wrapper(Kernel->GetInfo, 4, Kernel, &gEfiFileInfoGuid, &FileInfoSize, NULL);
        uefi_call_wrapper(SystemTable->BootServices->AllocatePool, 3, EfiLoaderData, FileInfoSize, (void**)&FileInfo);
        uefi_call_wrapper(Kernel->GetInfo, 4, Kernel, &gEfiFileInfoGuid, &FileInfoSize, (void**)&FileInfo);

        UINTN size = sizeof(header);
        uefi_call_wrapper(Kernel->Read, 3, Kernel, &size, &header);
    }

    if (
            memcmp(&header.e_ident[EI_MAG0], ELFMAG, SELFMAG) != 0 ||
            header.e_ident[EI_CLASS] != ELFCLASS64 ||
            header.e_ident[EI_DATA] != ELFDATA2LSB ||
            header.e_type != ET_EXEC ||
            header.e_machine != EM_X86_64 ||
            header.e_version != EV_CURRENT
            )
    {
        Print(L" [ERR]\n");
        Print(L"(Bootloader) Kernel format is bad\n");
        return EFI_LOAD_ERROR;
    }

    Elf64_Phdr* phdrs;
    {
        uefi_call_wrapper(Kernel->SetPosition, 2, Kernel, header.e_phoff);
        UINTN size = header.e_phnum * header.e_phentsize;
        uefi_call_wrapper(SystemTable->BootServices->AllocatePool, 2, EfiLoaderData, size, (void**)&phdrs);
        uefi_call_wrapper(Kernel->Read, 3, Kernel, &size, phdrs);
    }

    for (
            Elf64_Phdr* phdr = phdrs;
            (char*)phdr < (char*)phdrs + header.e_phnum * header.e_phentsize;
            phdr = (Elf64_Phdr*)((char*)phdr + header.e_phentsize))
    {
        switch (phdr->p_type){
            case PT_LOAD:
            {
                int pages = (phdr->p_memsz + 0x1000 - 1) / 0x1000;
                Elf64_Addr segment = phdr->p_paddr;
                EFI_STATUS s = uefi_call_wrapper(SystemTable->BootServices->AllocatePages, 4, AllocateAddress, EfiLoaderData, pages, &segment);
                if (EFI_ERROR(s)) {
                    Print(L" [FTL]\nError allocating memory for kernel at address from %f KiB to %f KiB\n",segment / (double) 1024, (segment + (pages * 4096)) / (double) 1024);
                    return EFI_LOAD_ERROR;
                }

                uefi_call_wrapper(Kernel->SetPosition, 2, Kernel, phdr->p_offset);
                UINTN size = phdr->p_filesz;
                uefi_call_wrapper(Kernel->Read, 3, Kernel, &size, (void*)segment);
                break;
            }
        }
    }

    Print(L" [OK]\n");

    //#endregion

    //#region Memory Map
    Print(L"(Bootloader) Loading memory map");
    EFI_MEMORY_DESCRIPTOR* Map = NULL;
    UINTN MapSize = 0, MapKey;
    UINTN DescriptorSize;
    UINT32 DescriptorVersion;

    status = uefi_call_wrapper(SystemTable->BootServices->GetMemoryMap, 5, &MapSize, Map, &MapKey, &DescriptorSize, &DescriptorVersion);
    MapSize += 512; // TODO Fix this
    status = uefi_call_wrapper(SystemTable->BootServices->AllocatePool, 3, EfiLoaderData, MapSize, (void**)&Map);
    if (EFI_ERROR(status)) {
        Print(L" [FTL]\n");
        Print(L"Cant allocate pool. Status: %d\n", status);
        return EFI_LOAD_ERROR;
    }
    status = uefi_call_wrapper(SystemTable->BootServices->GetMemoryMap, 5, &MapSize, Map, &MapKey, &DescriptorSize, &DescriptorVersion);
    if (EFI_ERROR(status)) {
        Print(L" [FTL]\n");
        Print(L"Status: %d\n", status);
        return EFI_LOAD_ERROR;
    }

    memoryMap.map = Map;
    memoryMap.mapSize = MapSize;
    memoryMap.descriptorSize = DescriptorSize;
    Print(L" [OK]\n");
    //#endregion

    //#region ACPI
    Print(L"(Bootloader) Searching for ACPI");
    EFI_CONFIGURATION_TABLE* configTable = SystemTable->ConfigurationTable;
    void* rsdp = NULL;
    EFI_GUID Acpi2TableGuid = ACPI_TABLE_GUID;

    for (UINTN index = 0; index < SystemTable->NumberOfTableEntries; index++){
        if (CompareGuid(&configTable->VendorGuid, &Acpi2TableGuid)){
            if (strcmp((CHAR8*)"RSD PTR ", (CHAR8*)configTable->VendorTable, 8)){
                rsdp = (void*)configTable->VendorTable;
                break;
            }
        }
        configTable++;
    }

    if (rsdp == NULL) {
        Print(L" [ERR]\n");
        Print(L"ACPI not found");
    } else {
        Print(L" [OK]\n");
    }
    //#endregion

    bootInfo.framebuffer = &framebuffer;
    bootInfo.psf1Font = font;
    bootInfo.memoryMap = &memoryMap;
    bootInfo.rsdp = rsdp;
    bootInfo.RT = RT;

    int (*KernelStart)(BootInfo *) = ((__attribute__((sysv_abi)) int (*)(BootInfo *) ) header.e_entry);

    uefi_call_wrapper(SystemTable->BootServices->ExitBootServices, 2, ImageHandle, MapKey);

    int kernelStatus = KernelStart(&bootInfo);

    Print(L"(Bootloader) Kernel status: %d\r\n", kernelStatus);

    if (kernelStatus != 1) return EFI_LOAD_ERROR;

    return EFI_SUCCESS;
}