pub mod acpi_handler;

use crate::acpi::acpi_handler::AcpiHandlerImpl;
use acpi::platform::interrupt::InterruptSourceOverride;
use acpi::{AcpiTables, InterruptModel};
use alloc::vec::Vec;
use x86_64::PhysAddr;

pub fn get_acpi_tables(rsbp_address: &Option<usize>) -> AcpiTables<AcpiHandlerImpl> {
    unsafe {
        AcpiTables::from_rsdp(
            AcpiHandlerImpl,
            rsbp_address.expect("Failed to parse ACPI tables: rsdp is not present"),
        )
        .expect("Failed to parse ACPI tables")
    }
}

#[derive(Debug, Clone)]
pub struct ApicInfo {
    pub local_apic_address: PhysAddr,
    pub io_apic_address: PhysAddr,
    pub io_apic_global_system_interrupt_base: u32,
    pub interrupt_source_override: Vec<InterruptSourceOverride>,
}

pub fn get_apic_info(acpi_tables: &AcpiTables<AcpiHandlerImpl>) -> ApicInfo {
    let platform = acpi_tables.platform_info().unwrap();

    match platform.interrupt_model {
        InterruptModel::Apic(apic) => {
            let io = &apic.io_apics[0];
            ApicInfo {
                local_apic_address: PhysAddr::new(apic.local_apic_address),
                io_apic_address: PhysAddr::new(io.address as u64),
                io_apic_global_system_interrupt_base: io.global_system_interrupt_base,
                interrupt_source_override: apic.interrupt_source_overrides.to_vec(),
            }
        }
        _ => panic!("No APIC in system"),
    }
}
