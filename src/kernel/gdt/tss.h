#ifndef MODULETOS_TSS_H
#define MODULETOS_TSS_H

#include <stdint.h>

struct TSS {
    uint32_t reserved0;
    uint64_t rsp0;
    uint64_t rsp1;
    uint64_t rsp2;
    uint64_t reserved1;
    uint64_t ist1;
    uint64_t ist2;
    uint64_t ist3;
    uint64_t ist4;
    uint64_t ist5;
    uint64_t ist6;
    uint64_t ist7;
    uint64_t reserved2;
    uint16_t reserved3;
    uint16_t iopb;
} __attribute__((packed));

extern "C" void flushTSS();

#define UPDATE_TSS(tss, var) \
                        uint64_t (var) = 0; \
                        asm volatile ("mov %0, %%rsp": "=r" (var)); \
                        (tss)->rsp0 = (var);

#endif //MODULETOS_TSS_H
