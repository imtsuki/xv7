use crate::config::*;
use bitvec::prelude::*;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::paging::Size4KiB;
pub use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, UnusedPhysFrame};

pub struct BitmapFrameAllocator<'map> {
    #[allow(unused)]
    inner: &'map mut BitSlice<Lsb0, u8>,
}

impl<'map> BitmapFrameAllocator<'map> {
    pub fn new(map: &'map mut [u8]) -> Self {
        Self {
            inner: BitSlice::from_slice_mut(map),
        }
    }

    #[allow(unused)]
    pub fn init(&mut self) {
        todo!();
    }

    pub fn hello(&mut self) {
        println!(
            "BitmapFrameAllocator: pages count: {}, bitmap occupies {}KiB",
            self.inner.len(),
            self.inner.len() / 1024,
        );
        println!(
            "BitmapFrameAllocator: supported pyhsical memory: {}GiB",
            self.inner.len() * 4096 / 1024 / 1024 / 1024,
        );
    }
}

unsafe impl<'map> FrameAllocator<Size4KiB> for BitmapFrameAllocator<'map> {
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame<Size4KiB>> {
        todo!()
    }
}

impl<'map> FrameDeallocator<Size4KiB> for BitmapFrameAllocator<'map> {
    fn deallocate_frame(&mut self, _frame: UnusedPhysFrame<Size4KiB>) {
        todo!()
    }
}

lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<BitmapFrameAllocator<'static>> = {
        static mut MAP: [u8; MAX_FRAMES_SUPPORTED / 8] = [0; MAX_FRAMES_SUPPORTED / 8];
        Mutex::new(BitmapFrameAllocator::new(unsafe { &mut MAP }))
    };
}
