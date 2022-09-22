#include "idt.h"

void IDTDescEntry::SetOffset(uint64_t offset){
    offset0 = (uint16_t)(offset & 0x000000000000ffff);
    offset1 = (uint16_t)((offset & 0x00000000ffff0000) >> 16);
    offset2 = (uint32_t)((offset & 0xffffffff00000000) >> 32);
}

uint64_t IDTDescEntry::GetOffset(){
    uint64_t offset = 0;
    offset |= (uint64_t)offset0;
    offset |= (uint64_t)offset1 << 16;
    offset |= (uint64_t)offset2 << 32;
    return offset;
}

void setIDTGate(IDTR* idtr, void* handler, uint8_t entryOffset, uint8_t typeAttr, uint8_t selector) {
    IDTDescEntry* interrupt = (IDTDescEntry*)(idtr->offset + entryOffset * sizeof(IDTDescEntry));
    interrupt->SetOffset((uint64_t)handler);
    interrupt->type_attr = typeAttr;
    interrupt->selector = selector;
}