mod address_space;
mod bitmap;
mod frame;

use boot::{BootArgs, MemoryMap, MemoryType, PhysAddr};

pub use address_space::{
    AddressSpace, MapPerm, USER_SPACE_MAX, USER_SPACE_MIN, VmArea, VmAreaKind,
};
pub use frame::OwnedFrame;

pub const PAGE_SIZE: usize = 4096;

/// Registers all usable physical RAM regions from the bootloader memory map.
pub fn init(args: &BootArgs) {
    let mut allocator = bitmap::FRAME_ALLOCATOR.lock();
    for descriptor in args.memory_map.clone().iter {
        if descriptor.ty == MemoryType::CONVENTIONAL {
            allocator.install_region(
                PhysAddr::new(descriptor.phys_start),
                descriptor.page_count as usize,
            );
        }
    }
}

#[allow(unused)]
pub fn free_frame_count() -> usize {
    bitmap::FRAME_ALLOCATOR.lock().free_frames()
}

pub fn print_memory_map(mmap: &MemoryMap) {
    println!("Mem phys map:");
    for descriptor in mmap.clone().iter {
        println!(
            "[mem {:#016x}-{:#016x} {:>8}] {:?}",
            descriptor.phys_start,
            descriptor.phys_start + descriptor.page_count * PAGE_SIZE as u64 - 1,
            descriptor.page_count,
            descriptor.ty
        );
    }
}
