use x86_64::VirtAddr;

use super::paging;
use crate::config::{KERNEL_HEAP_BASE, KERNEL_HEAP_SIZE};
use crate::memory::MapPerm;

pub fn init_heap() {
    paging::map_kernel_range(
        VirtAddr::new(KERNEL_HEAP_BASE),
        KERNEL_HEAP_SIZE,
        MapPerm::R | MapPerm::W,
    )
    .expect("Failed to map kernel heap");

    unsafe {
        crate::allocator::ALLOCATOR
            .lock()
            .init(KERNEL_HEAP_BASE as *mut u8, KERNEL_HEAP_SIZE);
    }
}
