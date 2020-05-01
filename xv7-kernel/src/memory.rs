use crate::config::*;
use crate::pretty::Pretty;
use bitvec::prelude::*;
use boot::PhysAddr;
use lazy_static::lazy_static;
use spin::Mutex;
pub use x86_64::structures::paging::{FrameAllocator, FrameDeallocator};
use x86_64::structures::paging::{PageSize, PhysFrame, Size4KiB};

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

    pub fn install_memory_region(&mut self, phys_start: PhysAddr, page_count: usize) {
        assert!(phys_start.is_aligned(Size4KiB::SIZE));
        let frame_start = (phys_start.as_u64() / Size4KiB::SIZE) as usize;
        for i in frame_start..frame_start + page_count {
            self.inner.set(i, true);
        }
    }

    #[allow(unused)]
    pub fn print_statistics(&mut self) {
        println!(
            "BitmapFrameAllocator: bitmap occupies {}, maximum supported pyhsical memory: {}",
            self.inner.len().pretty(),
            (self.inner.len() * Size4KiB::SIZE as usize).pretty(),
        );

        let frames = self.inner.count_ones();

        println!(
            "BitmapFrameAllocator: {} frames available, which is {} of memory",
            frames,
            (frames * Size4KiB::SIZE as usize).pretty(),
        );
    }
}

unsafe impl<'map> FrameAllocator<Size4KiB> for BitmapFrameAllocator<'map> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self
            .inner
            .iter()
            .enumerate()
            .filter_map(|(index, unused)| {
                if *unused {
                    Some((
                        index,
                        PhysFrame::containing_address(PhysAddr::new(index as u64 * Size4KiB::SIZE)),
                    ))
                } else {
                    None
                }
            })
            .next();

        if let Some((index, frame)) = frame {
            self.inner.set(index, false);
            Some(frame)
        } else {
            None
        }
    }
}

impl<'map> FrameDeallocator<Size4KiB> for BitmapFrameAllocator<'map> {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let index = frame.start_address().as_u64() / Size4KiB::SIZE;
        self.inner.set(index as usize, true);
    }
}

lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<BitmapFrameAllocator<'static>> = {
        static mut MAP: [u8; MAX_FRAMES_SUPPORTED / 8] = [0; MAX_FRAMES_SUPPORTED / 8];
        Mutex::new(BitmapFrameAllocator::new(unsafe { &mut MAP }))
    };
}
