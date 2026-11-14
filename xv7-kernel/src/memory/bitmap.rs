use core::mem::size_of_val;

use boot::PhysAddr;
use spin::Mutex;
pub use x86_64::structures::paging::{FrameAllocator, FrameDeallocator};
use x86_64::structures::paging::{PageSize, PhysFrame, Size4KiB};

use crate::config::*;
use crate::pretty::Pretty;

const WORD_BITS: usize = u64::BITS as usize;
const BITMAP_WORDS: usize = MAX_FRAMES_SUPPORTED / WORD_BITS;

pub static FRAME_ALLOCATOR: Mutex<BitmapFrameAllocator> = Mutex::new(BitmapFrameAllocator::new());

pub struct BitmapFrameAllocator {
    /// `1` bit = free physical frame, `0` bit = allocated / reserved.
    words: [u64; BITMAP_WORDS],
    next_word: usize,
}

impl BitmapFrameAllocator {
    pub const fn new() -> Self {
        Self {
            words: [0; BITMAP_WORDS],
            next_word: 0,
        }
    }

    pub fn install_memory_region(&mut self, phys_start: PhysAddr, page_count: usize) {
        assert!(phys_start.is_aligned(Size4KiB::SIZE));
        let start = (phys_start.as_u64() / Size4KiB::SIZE) as usize;
        let end = start.saturating_add(page_count).min(MAX_FRAMES_SUPPORTED);

        // Keep frame 0 reserved as a null-frame guard.
        let first = start.max(1);
        for frame in first..end {
            self.words[frame / WORD_BITS] |= 1u64 << (frame % WORD_BITS);
        }
        if first < end {
            let first_word = first / WORD_BITS;
            if first_word < self.next_word || self.words[self.next_word] == 0 {
                self.next_word = first_word;
            }
        }
    }

    #[allow(unused)]
    pub fn print_statistics(&self) {
        let free_frames: usize = self.words.iter().map(|w| w.count_ones() as usize).sum();
        println!(
            "BitmapFrameAllocator: bitmap occupies {}, maximum supported physical memory: {}",
            size_of_val(&self.words).pretty(),
            (MAX_FRAMES_SUPPORTED * Size4KiB::SIZE as usize).pretty(),
        );
        println!(
            "BitmapFrameAllocator: {} frames available, which is {} of memory",
            free_frames,
            (free_frames * Size4KiB::SIZE as usize).pretty(),
        );
    }
}

unsafe impl FrameAllocator<Size4KiB> for BitmapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        for offset in 0..BITMAP_WORDS {
            let word_idx = (self.next_word + offset) % BITMAP_WORDS;
            let word = self.words[word_idx];
            if word != 0 {
                let bit = word.trailing_zeros() as usize;
                self.words[word_idx] &= !(1u64 << bit);
                self.next_word = word_idx;
                let paddr = ((word_idx * WORD_BITS + bit) as u64) * Size4KiB::SIZE;
                return Some(PhysFrame::containing_address(PhysAddr::new(paddr)));
            }
        }
        None
    }
}

impl FrameDeallocator<Size4KiB> for BitmapFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let idx = (frame.start_address().as_u64() / Size4KiB::SIZE) as usize;
        assert!(idx > 0 && idx < MAX_FRAMES_SUPPORTED);
        let word_idx = idx / WORD_BITS;
        let mask = 1u64 << (idx % WORD_BITS);
        debug_assert_eq!(
            self.words[word_idx] & mask,
            0,
            "double free of frame {idx:#x}"
        );
        self.words[word_idx] |= mask;
        if word_idx < self.next_word {
            self.next_word = word_idx;
        }
    }
}
