use log::info;
use x86_64::structures::paging::{OffsetPageTable, PageTable, PageTableFlags};
use x86_64::VirtAddr;
use crate::VIRTUAL_TO_PHYSICAL_OFFSET;

fn walk_page_table(
    table: &PageTable,
    level: u8,
    base: u64,
    phys_offset: VirtAddr,
) {
    for (i, entry) in table.iter().enumerate() {
        if entry.is_unused() {
            continue;
        }

        let addr = base | ((i as u64) << (12 + 9 * (level - 1)));

        if level == 1 || entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            info!(
                "Virt: {:#018x} -> Phys: {:#018x} ({:?})",
                addr,
                entry.addr().as_u64(),
                entry.flags()
            );
        } else {
            let next_table_virt =
                phys_offset + entry.addr().as_u64();

            let next_table: &PageTable =
                unsafe { &*(next_table_virt.as_ptr()) };

            walk_page_table(next_table, level - 1, addr, phys_offset);
        }
    }
}

pub fn dump_lower_half(mapper: &OffsetPageTable) {
    let level_4_table = mapper.level_4_table();
    let phys_offset = VIRTUAL_TO_PHYSICAL_OFFSET;

    // Lower half = entries 0..256
    for (i, entry) in level_4_table.iter().enumerate().take(256) {
        if entry.is_unused() {
            continue;
        }

        let base = (i as u64) << 39;

        let next_table_virt =
            phys_offset + entry.addr().as_u64();

        let next_table: &PageTable =
            unsafe { &*(next_table_virt.as_ptr()) };

        walk_page_table(next_table, 3, base, phys_offset);
    }
}