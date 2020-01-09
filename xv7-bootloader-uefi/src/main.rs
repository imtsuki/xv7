#![no_std]
#![no_main]
#![feature(abi_efiapi)]

#[macro_use]
extern crate log;

use uefi::prelude::*;

#[no_mangle]
extern "C" fn __rust_probestack() {}

#[entry]
fn efi_main(_image: Handle, st: SystemTable<Boot>) -> Status {
    uefi_services::init(&st).expect_success("Failed to initialize utilities");
    st.stdout().clear();
    let bs = st.boot_services();
    let current_time = st.runtime_services().get_time().unwrap().unwrap();
    info!("Hello, UEFI");
    info!(
        "Now is {}/{}/{} {}:{}:{} {:?}",
        current_time.year(),
        current_time.month(),
        current_time.day(),
        current_time.hour(),
        current_time.minute(),
        current_time.second(),
        current_time.time_zone()
    );
    let map_size = bs.memory_map_size();
    info!("map_size: {}", map_size);

    loop {}
}
