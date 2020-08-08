use boot::MemoryMap;
pub use x86_64::structures::paging::{FrameAllocator, FrameDeallocator};
use x86_64::structures::paging::{PageSize, Size4KiB};

#[cfg(feature = "frame-allocator-bitmap")]
mod bitmap;
#[cfg(feature = "frame-allocator-bitmap")]
pub use bitmap::FRAME_ALLOCATOR;

#[cfg(feature = "frame-allocator-buddy")]
mod buddy;
#[cfg(feature = "frame-allocator-buddy")]
pub use buddy::FRAME_ALLOCATOR;

pub fn print_memory_map(mmap: &MemoryMap) {
    println!("Mem phys map:");
    for descriptor in mmap.clone().iter {
        println!(
            "[mem {:#016x}-{:#016x} {:>8}] {:?}",
            descriptor.phys_start,
            descriptor.phys_start + descriptor.page_count * Size4KiB::SIZE - 1,
            descriptor.page_count,
            descriptor.ty
        );
    }
}
