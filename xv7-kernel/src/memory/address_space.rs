use alloc::collections::BTreeMap;

use abi::error::*;
use bitflags::bitflags;
use boot::VirtAddr;

use super::{OwnedFrame, PAGE_SIZE};
use crate::arch::paging::{ArchPageTable, MapError};
use crate::config::PAGE_OFFSET_BASE;

/// Keep virtual page `0` unmapped so user null-pointer dereferences always fault.
pub const USER_SPACE_MIN: u64 = PAGE_SIZE as u64;
/// Exclusive upper bound of the 48-bit canonical lower-half user address space (`128 TiB`).
pub const USER_SPACE_MAX: u64 = 0x0000_8000_0000_0000;

bitflags! {
    /// Architecture-independent page mapping permissions.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct MapPerm: u8 {
        const R = 1 << 0;
        const W = 1 << 1;
        const X = 1 << 2;
        const U = 1 << 3;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VmAreaKind {
    Elf,
    Stack,
    Heap,
    Anonymous,
}

/// A contiguous, page-aligned mapped region in a user [`AddressSpace`].
#[derive(Clone, Debug)]
pub struct VmArea {
    pub start: u64,
    pub end: u64,
    pub perm: MapPerm,
    pub kind: VmAreaKind,
}

/// Per-process virtual address space owning its hardware page table and VMA table.
///
/// Dropping an `AddressSpace` automatically frees all user physical data frames
/// and all user page-table frames (`L1`..=`L4`).
pub struct AddressSpace {
    arch: ArchPageTable,
    areas: BTreeMap<u64, VmArea>,
}

#[inline]
const fn align_down(addr: u64) -> u64 {
    addr & !(PAGE_SIZE as u64 - 1)
}

#[inline]
const fn align_up(addr: u64) -> Option<u64> {
    let mask = PAGE_SIZE as u64 - 1;
    match addr.checked_add(mask) {
        Some(v) => Some(v & !mask),
        None => None,
    }
}

#[inline]
fn check_user_bounds(start: u64, len: usize) -> Result<u64> {
    let end = start
        .checked_add(len as u64)
        .ok_or_else(|| Error::new(EFAULT))?;
    if start < USER_SPACE_MIN || end > USER_SPACE_MAX {
        return Err(Error::new(EFAULT));
    }
    Ok(end)
}

impl AddressSpace {
    pub fn new() -> Result<Self> {
        let arch = ArchPageTable::new_user().ok_or_else(|| Error::new(ENOMEM))?;
        Ok(Self {
            arch,
            areas: BTreeMap::new(),
        })
    }

    #[inline]
    pub fn activate(&self) {
        self.arch.activate();
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.arch.is_active()
    }

    #[allow(unused)]
    #[inline]
    pub fn areas(&self) -> impl Iterator<Item = &VmArea> {
        self.areas.values()
    }

    /// Allocates zeroed physical frames and maps `[vaddr, vaddr + len)` (page-aligned outward)
    /// into this user address space with `perm`.
    ///
    /// Provides strong exception safety: if frame allocation fails partway through,
    /// any pages already mapped by this call are unmapped and freed before returning `Err(ENOMEM)`.
    pub fn map_region(
        &mut self,
        vaddr: u64,
        len: usize,
        perm: MapPerm,
        kind: VmAreaKind,
    ) -> Result<()> {
        if len == 0 || !perm.contains(MapPerm::U) {
            return Err(Error::new(EINVAL));
        }

        let raw_end = vaddr
            .checked_add(len as u64)
            .ok_or_else(|| Error::new(EINVAL))?;
        let start = align_down(vaddr);
        let end = align_up(raw_end).ok_or_else(|| Error::new(EINVAL))?;

        if start < USER_SPACE_MIN || end > USER_SPACE_MAX {
            return Err(Error::new(EFAULT));
        }

        if let Some((_, prev)) = self.areas.range(..end).next_back()
            && prev.end > start
        {
            return Err(Error::new(EEXIST));
        }

        let mut mapped_until = start;
        while mapped_until < end {
            let step = (|| -> core::result::Result<(), MapError> {
                let frame = OwnedFrame::alloc_zeroed().ok_or(MapError::OutOfMemory)?;
                self.arch.map_4k(VirtAddr::new(mapped_until), frame, perm)
            })();

            if let Err(err) = step {
                let mut rollback = start;
                while rollback < mapped_until {
                    let _ = self.arch.unmap_4k(VirtAddr::new(rollback));
                    rollback += PAGE_SIZE as u64;
                }
                return Err(match err {
                    MapError::OutOfMemory => Error::new(ENOMEM),
                    MapError::AlreadyMapped => Error::new(EEXIST),
                });
            }
            mapped_until += PAGE_SIZE as u64;
        }

        self.areas.insert(
            start,
            VmArea {
                start,
                end,
                perm,
                kind,
            },
        );
        Ok(())
    }

    /// Validates that every page covering `[uaddr, uaddr + len)` lies in user space
    /// and satisfies `required` permissions.
    pub fn validate_user_range(&self, uaddr: usize, len: usize, required: MapPerm) -> Result<()> {
        if len == 0 {
            return Ok(());
        }
        let start = uaddr as u64;
        let end = check_user_bounds(start, len)?;
        let required = required | MapPerm::U;

        let mut page = align_down(start);
        let last_page = align_down(end - 1);
        loop {
            let (_, perm) = self
                .arch
                .query(VirtAddr::new(page))
                .ok_or_else(|| Error::new(EFAULT))?;
            if !perm.contains(required) {
                return Err(Error::new(EFAULT));
            }
            if page == last_page {
                break;
            }
            page += PAGE_SIZE as u64;
        }
        Ok(())
    }

    /// Safely copies `dst.len()` bytes from user virtual address `src_uaddr` into kernel `dst`,
    /// verifying `MapPerm::U | MapPerm::R` on every touched page.
    pub fn copy_from_user(&self, src_uaddr: usize, dst: &mut [u8]) -> Result<()> {
        if dst.is_empty() {
            return Ok(());
        }
        let start = src_uaddr as u64;
        let _ = check_user_bounds(start, dst.len())?;

        let mut offset = 0usize;
        while offset < dst.len() {
            let cur_uaddr = start + offset as u64;
            let (paddr, perm) = self
                .arch
                .query(VirtAddr::new(cur_uaddr))
                .ok_or_else(|| Error::new(EFAULT))?;
            if !perm.contains(MapPerm::U | MapPerm::R) {
                return Err(Error::new(EFAULT));
            }
            let page_offset = (cur_uaddr as usize) & (PAGE_SIZE - 1);
            let chunk = (PAGE_SIZE - page_offset).min(dst.len() - offset);
            let src_ptr = (PAGE_OFFSET_BASE + paddr.as_u64()) as *const u8;
            unsafe {
                core::ptr::copy_nonoverlapping(src_ptr, dst.as_mut_ptr().add(offset), chunk);
            }
            offset += chunk;
        }
        Ok(())
    }

    /// Safely copies `src` from kernel memory into user virtual address `dst_uaddr`,
    /// verifying `MapPerm::U | MapPerm::W` on every touched page.
    pub fn copy_to_user(&mut self, dst_uaddr: usize, src: &[u8]) -> Result<()> {
        self.write_checked(dst_uaddr as u64, src, MapPerm::U | MapPerm::W)
    }

    /// Populates initial user segment contents (such as read-only `.text` / `.rodata` during `exec`)
    /// via the kernel direct physical mapping while keeping user PTEs non-writable (`W^X`).
    pub(crate) fn init_bytes(&mut self, dst_uaddr: u64, src: &[u8]) -> Result<()> {
        self.write_checked(dst_uaddr, src, MapPerm::U)
    }

    fn write_checked(&mut self, start: u64, src: &[u8], required: MapPerm) -> Result<()> {
        if src.is_empty() {
            return Ok(());
        }
        let _ = check_user_bounds(start, src.len())?;

        let mut offset = 0usize;
        while offset < src.len() {
            let cur_uaddr = start + offset as u64;
            let (paddr, perm) = self
                .arch
                .query(VirtAddr::new(cur_uaddr))
                .ok_or_else(|| Error::new(EFAULT))?;
            if !perm.contains(required) {
                return Err(Error::new(EFAULT));
            }
            let page_offset = (cur_uaddr as usize) & (PAGE_SIZE - 1);
            let chunk = (PAGE_SIZE - page_offset).min(src.len() - offset);
            let dst_ptr = (PAGE_OFFSET_BASE + paddr.as_u64()) as *mut u8;
            unsafe {
                core::ptr::copy_nonoverlapping(src.as_ptr().add(offset), dst_ptr, chunk);
            }
            offset += chunk;
        }
        Ok(())
    }
}
