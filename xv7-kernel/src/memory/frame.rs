use core::fmt;
use core::mem::ManuallyDrop;

use boot::PhysAddr;

use super::PAGE_SIZE;
use super::bitmap::FRAME_ALLOCATOR;
use crate::config::PAGE_OFFSET_BASE;

/// An exclusively owned 4 KiB physical memory frame.
///
/// Automatically returns the underlying physical page to `FRAME_ALLOCATOR` on `Drop`
/// unless ownership is explicitly transferred into a hardware PTE via [`OwnedFrame::into_raw`].
pub struct OwnedFrame(PhysAddr);

impl OwnedFrame {
    /// Allocates an uninitialized 4 KiB physical frame.
    #[inline]
    pub fn alloc() -> Option<Self> {
        FRAME_ALLOCATOR.lock().alloc().map(Self)
    }

    /// Allocates a 4 KiB physical frame and fills it with zeroes.
    #[inline]
    pub fn alloc_zeroed() -> Option<Self> {
        let mut frame = Self::alloc()?;
        frame.as_mut_slice().fill(0);
        Some(frame)
    }

    #[inline]
    pub const fn paddr(&self) -> PhysAddr {
        self.0
    }

    /// Views this frame's contents through the kernel direct physical mapping.
    #[inline]
    pub fn as_slice(&self) -> &[u8; PAGE_SIZE] {
        unsafe { &*((PAGE_OFFSET_BASE + self.0.as_u64()) as *const [u8; PAGE_SIZE]) }
    }

    /// Mutably views this frame's contents through the kernel direct physical mapping.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8; PAGE_SIZE] {
        unsafe { &mut *((PAGE_OFFSET_BASE + self.0.as_u64()) as *mut [u8; PAGE_SIZE]) }
    }

    /// Consumes the `OwnedFrame` without freeing it, returning its raw `PhysAddr`
    /// (e.g. when installing ownership into a page-table entry).
    #[inline]
    pub fn into_raw(self) -> PhysAddr {
        ManuallyDrop::new(self).0
    }

    /// Reconstructs an `OwnedFrame` from a raw `PhysAddr` previously produced by [`Self::into_raw`].
    ///
    /// # Safety
    /// `paddr` must be a valid page-aligned physical frame whose exclusive ownership is being reclaimed.
    #[inline]
    pub unsafe fn from_raw(paddr: PhysAddr) -> Self {
        debug_assert!(paddr.as_u64() != 0 && paddr.as_u64().is_multiple_of(PAGE_SIZE as u64));
        Self(paddr)
    }
}

impl Drop for OwnedFrame {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            FRAME_ALLOCATOR.lock().dealloc(self.0);
        }
    }
}

impl fmt::Debug for OwnedFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("OwnedFrame").field(&self.0).finish()
    }
}
