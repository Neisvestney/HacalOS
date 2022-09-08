//#include <efi.h>
//#include <efilib.h>

#include <elf.h>
#include "../gnu-efi/inc/efi.h"
#include "../gnu-efi/inc/efilib.h"

typedef unsigned long long size_t;

typedef struct {
    void* BaseAddress;
    size_t BufferSize;
    unsigned int Width;
    unsigned int Height;
    unsigned int PixelsPerScanline;
} Framebuffer;
Framebuffer framebuffer;

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

int memcmp(const void* aptr, const void* bptr, size_t n){
    const unsigned char* a = aptr, *b = bptr;
    for (size_t i = 0; i < n; i++){
        if (a[i] < b[i]) return -1;
        else if (a[i] > b[i]) return 1;
    }
    return 0;
}

EFI_STATUS efi_main (EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE *SystemTable)
{
    InitializeLib(ImageHandle, SystemTable);
    Print(L"Loading ModuletOS Kernel");

    ST = SystemTable;

    // GOP Setup
    EFI_STATUS status;
    EFI_GUID gopGuid = EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID;
    EFI_GRAPHICS_OUTPUT_PROTOCOL *gop;

    status = uefi_call_wrapper(BS->LocateProtocol, 3, &gopGuid, NULL, (void**)&gop);
    if(EFI_ERROR(status))
        Print(L"Unable to locate GOP\n");

    EFI_GRAPHICS_OUTPUT_MODE_INFORMATION *info;
    UINTN SizeOfInfo, numModes, nativeMode;

    uefi_call_wrapper(gop->QueryMode, 4, gop, gop->Mode==NULL?0:gop->Mode->Mode, &SizeOfInfo, &info);
    // this is needed to get the current video mode
    if (status == EFI_NOT_STARTED)
        status = uefi_call_wrapper(gop->SetMode, 2, gop, 0);
    if(EFI_ERROR(status)) {
        Print(L"Unable to get native mode\n");
    } else {
        nativeMode = gop->Mode->Mode;
        numModes = gop->Mode->MaxMode;
    }

    framebuffer.BaseAddress = (void*)gop->Mode->FrameBufferBase;
    framebuffer.BufferSize = gop->Mode->FrameBufferSize;
    framebuffer.Width = gop->Mode->Info->HorizontalResolution;
    framebuffer.Height = gop->Mode->Info->VerticalResolution;
    framebuffer.PixelsPerScanline = gop->Mode->Info->PixelsPerScanLine;

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

    // Load Kernel
    EFI_FILE* Kernel = LoadFile(NULL, L"kernel.elf", ImageHandle, SystemTable);
    if (Kernel == NULL){
        Print(L" [ERR]\n");
        Print(L"Could not load kernel\n");
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
        Print(L"Kernel format is bad\n");
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
                SystemTable->BootServices->AllocatePages(AllocateAddress, EfiLoaderData, pages, &segment);

                uefi_call_wrapper(Kernel->SetPosition, 2, Kernel, phdr->p_offset);
                UINTN size = phdr->p_filesz;
                uefi_call_wrapper(Kernel->Read, 3, Kernel, &size, (void*)segment);
                break;
            }
        }
    }

    Print(L" [OK]\n");

    int (*KernelStart)(Framebuffer*) = ((__attribute__((sysv_abi)) int (*)(Framebuffer*) ) header.e_entry);

    int kernelStatus = KernelStart(&framebuffer);

    Print(L"Kernel status: %d\r\n", kernelStatus);

    if (kernelStatus != 1) return EFI_LOAD_ERROR;

    return EFI_SUCCESS;
}