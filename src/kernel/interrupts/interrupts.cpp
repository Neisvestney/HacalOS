#include "interrupts.h"
#include "../BasicRenderer.h"
#include "../io.h"
#include "../cstr.h"

__attribute__((interrupt)) void PageFault_Handler(struct interrupt_frame* frame, uint64_t errorCode){
    basicRenderer.Print("Page Fault Detected: ");
    basicRenderer.Printl(toHexString(errorCode));
    while(true);
}

__attribute__((interrupt)) void DoubleFault_Handler(struct interrupt_frame* frame){
    basicRenderer.Printl("Double Fault Detected");
    while(true);
}

__attribute__((interrupt)) void GPFault_Handler(struct interrupt_frame* frame, uint64_t errorCode){
    basicRenderer.Print("General Protection Fault Detected: ");
    basicRenderer.Printl(toHexString(errorCode));
    basicRenderer.Printl(toHexString(frame->ss));
    basicRenderer.Printl(toHexString(frame->sp));
    basicRenderer.Printl(toHexString(frame->flags));
    basicRenderer.Printl(toHexString(frame->cs));
    basicRenderer.Printl(toHexString(frame->ip));
    while(true);
}

__attribute__((interrupt)) void KeyboardInt_Handler(struct interrupt_frame* frame){
    basicRenderer.Printl("Pressed");
    uint8_t scancode = inb(0x60);
    PIC_EndMaster();
}

__attribute__((interrupt)) void SysCall_Handler(struct interrupt_frame* frame){
    basicRenderer.Printl("SysCall");
    basicRenderer.Printl(toHexString(frame->ss));
    basicRenderer.Printl(toHexString(frame->sp));
    basicRenderer.Printl(toHexString(frame->flags));
    basicRenderer.Printl(toHexString(frame->cs));
    basicRenderer.Printl(toHexString(frame->ip));
}

void PIC_EndMaster(){
    outb(PIC1_COMMAND, PIC_EOI);
}

void PIC_EndSlave(){
    outb(PIC2_COMMAND, PIC_EOI);
    outb(PIC1_COMMAND, PIC_EOI);
}


void RemapPIC(){
    uint8_t a1, a2;

    a1 = inb(PIC1_DATA);
    io_wait();
    a2 = inb(PIC2_DATA);
    io_wait();

    outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
    io_wait();
    outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
    io_wait();

    outb(PIC1_DATA, 0x20);
    io_wait();
    outb(PIC2_DATA, 0x28);
    io_wait();

    outb(PIC1_DATA, 4);
    io_wait();
    outb(PIC2_DATA, 2);
    io_wait();

    outb(PIC1_DATA, ICW4_8086);
    io_wait();
    outb(PIC2_DATA, ICW4_8086);
    io_wait();

    outb(PIC1_DATA, a1);
    io_wait();
    outb(PIC2_DATA, a2);
}