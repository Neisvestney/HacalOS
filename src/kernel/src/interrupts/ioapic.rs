use crate::acpi::ApicInfo;
use crate::interrupts::idt::LAPIC_KEYBOARD_VECTOR;
use crate::memory::paging::map_mmio_single_page;
use crate::utils::relocate::relocate_addr;
use acpi::platform::interrupt::{InterruptSourceOverride, Polarity, TriggerMode};
use x2apic::ioapic::{IoApic, IrqFlags, IrqMode, RedirectionTableEntry};

pub const KEYBOARD_ISA_IRQ: u8 = 1;

pub const IO_APIC_BASE_OFFSET: u8 = 0x20;

pub fn resolve_irq(irq: u8, overrides: &[InterruptSourceOverride]) -> (u32, Polarity, TriggerMode) {
    if let Some(iso) = overrides.iter().find(|o| o.isa_source == irq) {
        let pol = match iso.polarity {
            Polarity::SameAsBus => Polarity::ActiveHigh,
            p => p,
        };
        let trig = match iso.trigger_mode {
            TriggerMode::SameAsBus => TriggerMode::Edge,
            t => t,
        };
        return (iso.global_system_interrupt, pol, trig);
    }

    (irq as u32, Polarity::ActiveHigh, TriggerMode::Edge)
}

pub unsafe fn map_irq(
    ioapic: &mut IoApic,
    gsi_base: u32,
    gsi: u32,
    vector: u8,
    polarity: Polarity,
    trigger: TriggerMode,
    dest_lapic_id: u8,
) {
    unsafe {
        let pin = (gsi - gsi_base) as u8;

        let mut flags = IrqFlags::empty();
        if polarity == Polarity::ActiveLow {
            flags |= IrqFlags::LOW_ACTIVE;
        }
        if trigger == TriggerMode::Level {
            flags |= IrqFlags::LEVEL_TRIGGERED;
        }

        let mut entry = RedirectionTableEntry::default();
        entry.set_mode(IrqMode::Fixed);
        entry.set_flags(flags);
        entry.set_dest(dest_lapic_id);
        entry.set_vector(vector);

        ioapic.set_table_entry(pin, entry);
        ioapic.enable_irq(pin);
    }
}

pub fn init_ioapic_interrupts(apic_info: &ApicInfo, lapic_id: u8) {
    unsafe {
        map_mmio_single_page(apic_info.io_apic_address).expect("Failed to map IO APIC MMIO region");

        let mut ioapic = IoApic::new(relocate_addr(apic_info.io_apic_address).as_u64());
        // IO Apic initialization
        ioapic.init(IO_APIC_BASE_OFFSET);

        // Keyboard IRQ initialization
        let (gsi, polarity, trigger) =
            resolve_irq(KEYBOARD_ISA_IRQ, &apic_info.interrupt_source_override);
        let pin = (gsi - apic_info.io_apic_global_system_interrupt_base) as u8;

        map_irq(
            &mut ioapic,
            apic_info.io_apic_global_system_interrupt_base,
            gsi,
            LAPIC_KEYBOARD_VECTOR,
            polarity,
            trigger,
            lapic_id,
        );

        log::info!(
            "Keyboard: IRQ{} to GSI{} to pin{} to vector {:#x} to LAPIC {}",
            KEYBOARD_ISA_IRQ,
            gsi,
            pin,
            LAPIC_KEYBOARD_VECTOR,
            lapic_id
        );
    }
}
