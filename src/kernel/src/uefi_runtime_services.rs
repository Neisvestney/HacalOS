use crate::utils::relocate::relocate_addr;
use alloc::vec::Vec;
use uefi::prelude::SystemTable;
use uefi::table::Runtime;
use uefi::table::boot::{MemoryMap, MemoryMapIter};
use x86_64::PhysAddr;

pub fn relocate_uefi_runtime_services(
    runtime_system_table: SystemTable<Runtime>,
    memory_map: &MemoryMap,
) -> SystemTable<Runtime> {
    let mut relocated_memory_map = Vec::new();
    for memory_map_entry in memory_map.entries() {
        let mut memory_map_entry = *memory_map_entry;
        memory_map_entry.virt_start =
            relocate_addr(PhysAddr::new(memory_map_entry.phys_start)).as_u64();
        relocated_memory_map.push(memory_map_entry);
    }

    let ptr = runtime_system_table.as_ptr() as u64;

    unsafe {
        runtime_system_table
            .set_virtual_address_map(
                &mut *relocated_memory_map,
                relocate_addr(PhysAddr::new(ptr)).as_u64(),
            )
            .expect("Failed to relocate runtime system table")
    }
}
