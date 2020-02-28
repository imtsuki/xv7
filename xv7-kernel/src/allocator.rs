use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        todo!()
    }
}

#[allow(unused)]
pub fn init_heap() {}
