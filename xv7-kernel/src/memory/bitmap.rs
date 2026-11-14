use core::mem::size_of_val;

use boot::PhysAddr;
use spin::Mutex;

use super::PAGE_SIZE;
use crate::config::MAX_FRAMES_SUPPORTED;
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

    pub fn install_region(&mut self, phys_start: PhysAddr, page_count: usize) {
        assert_eq!(phys_start.as_u64() % PAGE_SIZE as u64, 0);
        let start = (phys_start.as_u64() as usize) / PAGE_SIZE;
        let end = start.saturating_add(page_count).min(MAX_FRAMES_SUPPORTED);

        // Keep physical frame 0 reserved as a null-frame sentinel.
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

    pub fn alloc(&mut self) -> Option<PhysAddr> {
        for offset in 0..BITMAP_WORDS {
            let word_idx = (self.next_word + offset) % BITMAP_WORDS;
            let word = self.words[word_idx];
            if word != 0 {
                let bit = word.trailing_zeros() as usize;
                self.words[word_idx] &= !(1u64 << bit);
                self.next_word = word_idx;
                let paddr = ((word_idx * WORD_BITS + bit) * PAGE_SIZE) as u64;
                return Some(PhysAddr::new(paddr));
            }
        }
        None
    }

    /// # Safety
    /// `paddr` must be a page-aligned physical frame previously allocated from this allocator
    /// and no longer referenced anywhere.
    pub unsafe fn dealloc(&mut self, paddr: PhysAddr) {
        debug_assert_eq!(paddr.as_u64() % PAGE_SIZE as u64, 0);
        let idx = (paddr.as_u64() as usize) / PAGE_SIZE;
        assert!(idx > 0 && idx < MAX_FRAMES_SUPPORTED);
        let word_idx = idx / WORD_BITS;
        let mask = 1u64 << (idx % WORD_BITS);
        debug_assert_eq!(
            self.words[word_idx] & mask,
            0,
            "double free of physical frame {idx:#x}"
        );
        self.words[word_idx] |= mask;
        if word_idx < self.next_word {
            self.next_word = word_idx;
        }
    }

    #[allow(unused)]
    pub fn free_frames(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }

    #[allow(unused)]
    pub fn print_statistics(&self) {
        let free_frames = self.free_frames();
        println!(
            "BitmapFrameAllocator: bitmap occupies {}, maximum supported physical memory: {}",
            size_of_val(&self.words).pretty(),
            (MAX_FRAMES_SUPPORTED * PAGE_SIZE).pretty(),
        );
        println!(
            "BitmapFrameAllocator: {} frames available, which is {} of memory",
            free_frames,
            (free_frames * PAGE_SIZE).pretty(),
        );
    }
}
