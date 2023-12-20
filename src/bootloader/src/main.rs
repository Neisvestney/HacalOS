#![no_main]
#![no_std]

use log::{error, info};
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::Result;

#[entry]
fn main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    info!("Hello world!");
    error!("Hello world!");

    let bs = system_table.boot_services();

    let gop_handle = bs.get_handle_for_protocol::<GraphicsOutput>().unwrap();

    bs.stall(10_000_000);

    Status::SUCCESS
}
