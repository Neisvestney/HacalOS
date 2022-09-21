[bits 64]
goToUserMode:
; IRETQ Method
    cli
    mov rcx, rax
    mov ax, (4 * 8) | 3 ; ring 3 data with bottom 2 bits set for ring 3
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax ; SS is handled by iret
    ; set up the stack frame iret expects
    mov rax, rsp
    push (4 * 8) | 3 ; data selector
    push rax ; current esp
    pushf ; eflags
    pop rbx ; Get EFLAGS back into EAX. The only way to read EFLAGS is to pushf then pop.
    or rbx, 0x200 ; Set the IF flag.
    push rbx ; Push the new EFLAGS value back onto the stack.
    push (3 * 8) | 3 ; code selector (ring 3 code with bottom 2 bits set for ring 3)
    push rcx ; instruction address to return to
    o64 iret

; SYSEXIT Method
;    mov ax, (4 * 8) | 3 ; user data segment with RPL 3
;    mov ds, ax
;    mov es, ax
;    mov fs, ax
;    mov gs, ax ; sysexit sets SS
;
;    ; setup wrmsr inputs
;    xor rdx, rdx ; not necessary; set to 0
;    mov rax, 0x100008 ; SS=0x10+0x10=0x20, CS=0x8+0x10=0x18
;    mov rcx, 0x174 ; MSR specifier: IA32_SYSENTER_CS
;    wrmsr ; set sysexit segments
;
;    ; setup sysexit inputs
;    mov rdx, inUserMode ; to be loaded into EIP
;    mov rcx, rsp ; to be loaded into ESP
;    sysexit

; SYSCALL Method
; enable system call extensions that enables sysret and syscall
;	mov rcx, 0xc0000082
;	wrmsr
;	mov rcx, 0xc0000080
;	rdmsr
;	or eax, 1
;	wrmsr
;	mov rcx, 0xc0000081
;	rdmsr
;	mov edx, 0x00180008
;	wrmsr
;
;	mov ecx, inUserMode ; to be loaded into RIP
;	mov r11, 0x202 ; to be loaded into EFLAGS
;	o64 sysret ;use "o64 sysret" if you assemble with NASM

GLOBAL goToUserMode