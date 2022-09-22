#ifndef MODULETOS_IDT_H
#define MODULETOS_IDT_H

#include <stdint.h>

#define IDT_TA_InterruptGate    0b10001110
#define IDT_SysCall             0b11101110
#define IDT_TA_CallGate         0b10001100
#define IDT_TA_TrapGate         0b10001111

struct IDTDescEntry {
    uint16_t offset0;
    uint16_t selector;
    uint8_t ist;
    uint8_t type_attr;
    uint16_t offset1;
    uint32_t offset2;
    uint32_t ignore;
    void SetOffset(uint64_t offset);
    uint64_t GetOffset();
};

struct IDTR {
    uint16_t limit;
    uint64_t offset;
} __attribute__((packed));

void setIDTGate(IDTR* idtr, void* handler, uint8_t entryOffset, uint8_t typeAttr, uint8_t selector);

#endif //MODULETOS_IDT_H
