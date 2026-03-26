pub fn align_down(addr: u64, align: u64) -> u64 {
    addr & !(align - 1)
}

pub fn align_up(addr: u64, align: u64) -> u64 {
    (addr + align - 1) & !(align - 1)
}
