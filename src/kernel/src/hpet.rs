use crate::acpi::acpi_handler::AcpiHandlerImpl;
use crate::memory::paging::map_mmio_single_page;
use crate::utils::relocate::relocate_addr;
use acpi::{AcpiTables, HpetInfo};
use core::num::NonZero;
use ez_hpet::Hpet;
use log::info;
use spin::{Once, RwLock};
use x86_64::PhysAddr;

pub static HPET: Once<RwLock<HpetImpl<'static>>> = Once::new();

pub struct HpetImpl<'a>(Hpet<'a>);

impl<'a> HpetImpl<'a> {
    pub fn new(hpet: Hpet<'a>) -> Self {
        HpetImpl(hpet)
    }

    #[inline(always)]
    fn wait(&self, fs_per_unit: u64, unit: u64) {
        let h = &self.0;
        let period_fs = h.main_counter_tick_period() as u64;
        let ticks_per_unit = fs_per_unit / period_fs;

        let target = h.main_counter_value() + ticks_per_unit * unit;
        while h.main_counter_value() < target {
            core::hint::spin_loop();
        }
    }

    pub fn wait_ms(&self, ms: u64) {
        let fs_per_ms = 1_000_000_000_000u64;
        self.wait(fs_per_ms, ms);
    }

    pub fn wait_us(&self, us: u64) {
        let fs_per_us = 1_000_000_000u64;
        self.wait(fs_per_us, us);
    }

    #[inline(always)]
    fn read_current(&self, fs_per_unit: u64) -> u64 {
        let h = &self.0;
        let period_fs = h.main_counter_tick_period() as u64;
        let ticks = h.main_counter_value();

        (ticks * period_fs) / fs_per_unit
    }

    pub fn read_current_us(&self) -> u64 {
        let fs_per_us = 1_000_000_000u64;
        self.read_current(fs_per_us)
    }

    pub fn read_current_ms(&self) -> u64 {
        let fs_per_ms = 1_000_000_000_000u64;
        self.read_current(fs_per_ms)
    }

    pub fn get_hpet(&self) -> &Hpet<'_> {
        &self.0
    }

    pub fn get_hpet_mut(&mut self) -> &'a mut Hpet<'_> {
        &mut self.0
    }
}

pub fn init_hpet(acpi_tables: &AcpiTables<AcpiHandlerImpl>) {
    unsafe {
        let hpet_info = HpetInfo::new(acpi_tables).expect("No HPET table in ACPI");
        info!("HPET info: {:#x?}", hpet_info);
        let hpet_phys_addr = PhysAddr::new(hpet_info.base_address as u64);
        map_mmio_single_page(hpet_phys_addr);
        let mut hpet =
            Hpet::new(NonZero::new(relocate_addr(hpet_phys_addr).as_u64() as usize).unwrap());

        hpet.set_enable(true);

        let period_fs = hpet.main_counter_tick_period();
        let freq_hz = 1_000_000_000_000_000u64 / period_fs as u64;
        info!(
            "HPET period: {} fs, freq: {} Hz, supports_64_bit_mode: {}",
            period_fs,
            freq_hz,
            hpet.supports_64_bit_mode()
        );
        HPET.call_once(|| RwLock::new(HpetImpl::new(hpet)));
    }
}
