use linked_list_allocator::LockedHeap;

pub fn init_heap() {
    crate::arch::allocator::init_heap();
}

#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();
