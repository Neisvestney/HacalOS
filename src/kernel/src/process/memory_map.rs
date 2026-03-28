use alloc::string::String;
use alloc::sync::Arc;
use derivative::Derivative;
use x86_64::structures::paging::Page;
use x86_64::structures::paging::page::PageRangeInclusive;

#[derive(Debug)]
pub struct ProcessMemoryMapEntry {
    pub page_range: PageRangeInclusive,
    pub entry_type: ProcessMemoryMapEntryType,
}

impl ProcessMemoryMapEntry {
    pub fn new(page_range: PageRangeInclusive, entry_type: ProcessMemoryMapEntryType) -> Self {
        ProcessMemoryMapEntry {
            page_range,
            entry_type,
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub enum ProcessMemoryMapEntryType {
    File {
        path: Arc<String>,
        #[derivative(Debug(format_with="crate::utils::fmt_hex::fmt_hex"))]
        offset_in_segment: u64,
        #[derivative(Debug(format_with="crate::utils::fmt_hex::fmt_hex"))]
        offset_in_file: u64,
        #[derivative(Debug(format_with="crate::utils::human_bytes::u64_human_bytes_formatter"))]
        size: u64,
    },
    Stack {
        guard_page: Page,
    },
    Heap {},
}

impl ProcessMemoryMapEntryType {
    pub fn new_file(
        path: Arc<String>,
        offset_in_segment: u64,
        offset_in_file: u64,
        size: u64,
    ) -> Self {
        ProcessMemoryMapEntryType::File {
            path,
            offset_in_segment,
            offset_in_file,
            size,
        }
    }

    pub fn new_stack(guard_page: Page) -> Self {
        ProcessMemoryMapEntryType::Stack { guard_page }
    }
}
