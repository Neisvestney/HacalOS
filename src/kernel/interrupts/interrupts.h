#ifndef MODULETOS_INTERRUPTS_H
#define MODULETOS_INTERRUPTS_H

#include <stdint.h>

#define PIC1_COMMAND 0x20
#define PIC1_DATA 0x21
#define PIC2_COMMAND 0xA0
#define PIC2_DATA 0xA1
#define PIC_EOI 0x20

#define ICW1_INIT 0x10
#define ICW1_ICW4 0x01
#define ICW4_8086 0x01

struct interrupt_frame
{
    uint64_t ip;
    uint64_t cs;
    uint64_t flags;
    uint64_t sp;
    uint64_t ss;
};
__attribute__((interrupt)) void PageFault_Handler(struct interrupt_frame* frame);
__attribute__((interrupt)) void DoubleFault_Handler(struct interrupt_frame* frame);
__attribute__((interrupt)) void GPFault_Handler(struct interrupt_frame* frame);
__attribute__((interrupt)) void KeyboardInt_Handler(struct interrupt_frame* frame);
__attribute__((interrupt)) void SysCall_Handler(struct interrupt_frame* frame);

void RemapPIC();
void PIC_EndMaster();
void PIC_EndSlave();

#endif //MODULETOS_INTERRUPTS_H
