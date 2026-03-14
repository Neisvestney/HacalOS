use crate::acpi::ApicInfo;
use crate::interrupts::idt::{LAPIC_ERROR_VECTOR, LAPIC_SPURIOUS_VECTOR, LAPIC_TIMER_VECTOR};
use crate::memory::paging::map_mmio_single_page;
use crate::utils::relocate::relocate_addr;
use log::info;
use spin::{Mutex, Once};
use x2apic::lapic::{LocalApic, LocalApicBuilder};

pub fn init_lapic(apic_info: &ApicInfo) -> LocalApic {
    unsafe {
        let lapic_virt_addr = relocate_addr(apic_info.local_apic_address).as_u64();

        map_mmio_single_page(apic_info.local_apic_address).expect("Failed to map lapic page");

        let mut lapic = LocalApicBuilder::new()
            .timer_vector(LAPIC_TIMER_VECTOR as usize)
            .error_vector(LAPIC_ERROR_VECTOR as usize)
            .spurious_vector(LAPIC_SPURIOUS_VECTOR as usize)
            .set_xapic_base(lapic_virt_addr)
            .build()
            .expect("Failed to build LocalApic");

        lapic.enable();
        info!("Local APIC enabled, id={}", lapic.id());

        lapic
    }
}
