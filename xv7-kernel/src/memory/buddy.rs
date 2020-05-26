use crate::config::*;
use crate::pretty::Pretty;
use bitvec::prelude::*;
use boot::PhysAddr;
use core::mem;
use core::ptr;
use lazy_static::lazy_static;
use spin::Mutex;
pub use x86_64::structures::paging::{FrameAllocator, FrameDeallocator};
use x86_64::structures::paging::{PageSize, PhysFrame, Size4KiB};

/// Buddy system allocator
///
/// another frame allocator implemented with buddy system
pub struct BuddyFrameAllocator<'zone> {
    zone: &'zone mut BuddyZone,
    frames: &'zone mut [BuddyFrame; MAX_FRAMES_SUPPORTED],
}

/// The max value of order. Consecutive (1<<order) elements are a block.
const MAX_ORDER: u8 = 11;

/// Macro to Round up $val by $align
macro_rules! align_to_upper {
    ($val: expr, $align:expr) => {
        (($val + $align - 1) / $align * $align)
    };
}

/// Macro to Round down $val by $align
macro_rules! align_to_lower {
    ($val: expr, $align:expr) => {
        ($val / $align * $align)
    };
}

/// Macro to get the buddy of a block
macro_rules! get_buddy {
    ($frame_index: expr, $order:expr) => {
        if ($frame_index) & (1 << ($order + 1) - 1) == 0 {
            $frame_index + (1 << $order)
        } else {
            $frame_index - (1 << $order)
        }
    };
}

/// The status of each block
#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
enum BuddyFrameStatus {
    UNCHECKED = 0, // default value, which means the block is not used and cannot be used
    USED = 1,      // the block was used
    NOTUSED = 2,   // the block was not used, so it should be stored in a BuddyFreeArea
}

/// A block of frames in buddy system, represents the physical memory of (1 << order) size
#[derive(Copy, Clone)]
struct BuddyFrame {
    next: *mut BuddyFrame, // Simply embed a pointers in it
    use_status: BuddyFrameStatus,
    order: u8,
}

/// Represens a group of blocks with same order.
#[derive(Copy, Clone)]
struct BuddyFreeArea {
    head: *mut BuddyFrame,
    length: usize,
}
/// Memory zone, BuddyFrameAllocator may allocate memory in different areas according to demand().
/// Only one zone is implemented now.
#[derive(Copy, Clone)]
struct BuddyZone {
    head: *mut BuddyFrame,
    free_area: [BuddyFreeArea; MAX_ORDER as usize],
}

impl BuddyFreeArea {
    unsafe fn drop_frame(&mut self, frame: *mut BuddyFrame) -> *mut BuddyFrame {
        let mut next = self.head;
        let mut pre = ptr::null_mut();
        while !next.is_null() && next != frame {
            pre = next;
            next = (*next).next;
        }
        if next.is_null() {
            return ptr::null_mut();
        }
        self.length -= 1;
        let next = (*frame).next;

        if !pre.is_null() {
            (*pre).next = next;
        } else {
            // head
            self.head = next;
        }
        (*frame).next = ptr::null_mut();
        (*frame).use_status = BuddyFrameStatus::NOTUSED;
        return frame;
    }

    unsafe fn push_frame(&mut self, frame: *mut BuddyFrame) {
        self.length += 1;
        (*frame).next = self.head;
        self.head = frame;
        (*frame).use_status = BuddyFrameStatus::NOTUSED;
    }

    unsafe fn pop_frame(&mut self) -> *mut BuddyFrame {
        if self.length == 0 {
            return ptr::null_mut();
        }
        self.length -= 1;
        let head = self.head;
        self.head = (*head).next;
        (*head).next = ptr::null_mut();
        (*head).use_status = BuddyFrameStatus::USED;
        return head;
    }
}

impl BuddyZone {
    fn count_free_mem(&self) -> usize {
        let mut mem_count = 0usize;
        for i in 0..MAX_ORDER {
            mem_count += self.free_area[i as usize].length * (1 << i);
        }
        return mem_count;
    }
}

unsafe impl<'zone> Send for BuddyFrameAllocator<'zone> {}

impl<'zone> BuddyFrameAllocator<'zone> {
    fn new(
        zone: &'zone mut BuddyZone,
        frames: &'zone mut [BuddyFrame; MAX_FRAMES_SUPPORTED],
    ) -> Self {
        Self {
            zone: zone,
            frames: frames,
        }
    }

    fn index_of_frame(&mut self, frame: *mut BuddyFrame) -> usize {
        (frame as usize - (&mut self.frames[0] as *mut BuddyFrame) as usize)
            / mem::size_of::<BuddyFrame>()
    }
    pub fn install_memory_region(&mut self, phys_start: PhysAddr, page_count: usize) {
        assert!(phys_start.is_aligned(Size4KiB::SIZE));
        let frame_start = (phys_start.as_u64() / Size4KiB::SIZE) as usize;
        unsafe {
            self.free_frame_range(frame_start, frame_start + page_count);
        }
    }

    unsafe fn free_frame_range(&mut self, index_l: usize, index_r: usize) {
        self.free_frame_range_top_down(index_l, index_r, MAX_ORDER - 1)
    }

    unsafe fn free_frame_range_top_down(&mut self, index_l: usize, index_r: usize, order: u8) {
        if index_l >= index_r || order >= MAX_ORDER {
            return;
        }

        let block_size: usize = 1 << order;

        let align_index_l: usize = align_to_upper!(index_l, block_size);
        let align_index_r: usize = align_to_lower!(index_r, block_size);
        if align_index_l <= align_index_r {
            self.free_frame_range_top_down(index_l, align_index_l, order.wrapping_sub(1));
            for frame_index in (align_index_l..align_index_r).step_by(1 << order) {
                self.free_frame_specific_order(frame_index, order);
            }
            self.free_frame_range_top_down(align_index_r, index_r, order.wrapping_sub(1));
        } else {
            self.free_frame_range_top_down(index_l, index_r, order.wrapping_sub(1));
        }
    }

    unsafe fn free_frame_specific_order(&mut self, mut frame_index: usize, mut order: u8) {
        if order >= MAX_ORDER {
            return;
        }

        if self.frames[frame_index].use_status == BuddyFrameStatus::NOTUSED {
            println!(
                "BuddyFrameAllocator: free twice on frame({}) detected",
                frame_index
            );
            return;
        }
        // there are one zome only in this implementation
        while order < MAX_ORDER {
            if order == MAX_ORDER - 1 {
                break;
            }
            let area = &mut self.zone.free_area[order as usize];
            let buddy_index = get_buddy!(frame_index, order);
            let buddy_frame = area.drop_frame(&mut self.frames[buddy_index]);
            if !buddy_frame.is_null() {
                frame_index = if frame_index < buddy_index {
                    frame_index
                } else {
                    buddy_index
                };
                order += 1;
            } else {
                break;
            }
        }
        assert_eq!(
            frame_index,
            align_to_lower!(frame_index, (1 << order) as usize),
            "frame_index {} cannot match order {}",
            frame_index,
            order
        );

        self.frames[frame_index].order = order;
        self.zone.free_area[order as usize].push_frame(&mut self.frames[frame_index]);
    }

    unsafe fn alloc_frame_specific_order(&mut self, order: u8) -> *mut BuddyFrame {
        let mut upper_order = order;
        while upper_order < MAX_ORDER && self.zone.free_area[upper_order as usize].length <= 0 {
            upper_order += 1; // search down to top
        }
        if upper_order >= MAX_ORDER {
            return ptr::null_mut(); // alloc failed, not enough space
        }
        let large_frame = self.zone.free_area[upper_order as usize].pop_frame();
        while upper_order > order {
            let offset = (1 << (upper_order - 1)) + self.index_of_frame(large_frame);
            self.frames[offset].order = upper_order - 1;
            self.zone.free_area[(upper_order - 1) as usize]
                .push_frame(&mut self.frames[offset] as *mut BuddyFrame);
            upper_order -= 1;
        }
        (*large_frame).use_status = BuddyFrameStatus::USED;
        (*large_frame).order = order;
        return large_frame;
    }

    pub unsafe fn check_bugs(&mut self) {
        for i in 0..MAX_ORDER {
            let area = self.zone.free_area[i as usize];

            let mut j = 0;
            let mut cur = area.head;
            while !cur.is_null() {
                let next = (*cur).next;
                let offset = self.index_of_frame(cur);
                assert_eq!(
                    offset,
                    align_to_lower!(offset, (1 << i) as usize),
                    "area({})'s frame at index({}) has offset({}), cannot match order",
                    i,
                    j,
                    offset
                );
                j += 1;
                cur = next;
            }
            assert_eq!(
                j, area.length,
                "area({})'s length was not equals to it's link length",
                i
            );
        }
    }

    #[allow(unused)]
    pub unsafe fn print_statistics(&mut self) {
        self.check_bugs();
        let free_mem_count = self.zone.count_free_mem();
        println!(
            "BuddyFrameAllocator: {} frames available, which is {} of memory",
            free_mem_count,
            (free_mem_count * Size4KiB::SIZE as usize).pretty(),
        );

        print!("default zone:\t");
        for i in 0..MAX_ORDER {
            print!("{:>8}", self.zone.free_area[i as usize].length);
        }
        println!();
    }
}

unsafe impl<'zone> FrameAllocator<Size4KiB> for BuddyFrameAllocator<'zone> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = unsafe { self.alloc_frame_specific_order(0) };
        if !frame.is_null() {
            Some(PhysFrame::containing_address(PhysAddr::new(
                self.index_of_frame(frame) as u64 * Size4KiB::SIZE,
            )))
        } else {
            None
        }
    }
}

impl<'zone> FrameDeallocator<Size4KiB> for BuddyFrameAllocator<'zone> {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let index = frame.start_address().as_u64() / Size4KiB::SIZE;
        self.free_frame_specific_order(index as usize, 0);
    }
}

lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<BuddyFrameAllocator<'static>> = {
        unsafe {
            static mut FRAMES: [BuddyFrame; MAX_FRAMES_SUPPORTED] = [BuddyFrame {
                next: ptr::null_mut(),
                use_status: BuddyFrameStatus::UNCHECKED,
                order: 0,
            };
                MAX_FRAMES_SUPPORTED];
            static mut DEFAULT_ZONE: BuddyZone = BuddyZone {
                head: ptr::null_mut(),
                free_area: [BuddyFreeArea {
                    head: ptr::null_mut(),
                    length: 0,
                }; MAX_ORDER as usize],
            };
            DEFAULT_ZONE.head = &mut FRAMES[0];
            Mutex::new(BuddyFrameAllocator::new(&mut DEFAULT_ZONE, &mut FRAMES))
        }
    };
}
