use x86_64::VirtAddr;
use x86_64::registers::rflags::RFlags;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::idt::InterruptStackFrameValue;
use crate::gdt::segment_selectors;
use crate::interrupts::handlers::syscall_handler::SyscallContext;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CpuRegistriesContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,

    pub stack_frame: InterruptStackFrameValue,
}

impl CpuRegistriesContext {
    pub fn empty() -> Self {
        CpuRegistriesContext {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rsi: 0,
            rdi: 0,
            rbp: 0,
            rdx: 0,
            rcx: 0,
            rbx: 0,
            rax: 0,

            stack_frame: InterruptStackFrameValue::new(
                VirtAddr::new(0),
                SegmentSelector::NULL,
                RFlags::empty(),
                VirtAddr::new(0),
                SegmentSelector::NULL,
            ),
        }
    }

    pub fn from_syscall_context(syscall_context: &SyscallContext, stack_pointer: VirtAddr) -> Self {
        let segment_selectors = segment_selectors();

        unsafe {
            CpuRegistriesContext {
                r15: syscall_context.r15,
                r14: syscall_context.r14,
                r13: syscall_context.r13,
                r12: syscall_context.r12,
                r11: syscall_context.r11_rflags,
                r10: syscall_context.r10,
                r9: syscall_context.r9,
                r8: syscall_context.r8,
                rsi: syscall_context.rsi,
                rdi: syscall_context.rdi,
                rbp: syscall_context.rbp,
                rdx: syscall_context.rdx,
                rcx: syscall_context.rcx_rip,
                rbx: syscall_context.rbx,
                rax: syscall_context.rax,
                stack_frame: InterruptStackFrameValue::new(
                    VirtAddr::new_unsafe(syscall_context.rcx_rip),
                    segment_selectors.user_code_selector,
                    RFlags::from_bits_truncate(syscall_context.r11_rflags),
                    stack_pointer,
                    segment_selectors.user_data_selector,
                ),
            }
        }
    }
}
