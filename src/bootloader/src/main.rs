#![no_main]
#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use bootloader_structs::BootInfo;
use log::{error, info};
use uefi::prelude::*;
use uefi_services::println;

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    info!("Hello world!");
    error!("Hello world!");

    let bs = system_table.boot_services();

    let a = Box::new(BootInfo {
        i: 2
    });

    info!("{}", a.i);

    bs.stall(10_000_000);

    Status::SUCCESS
}
