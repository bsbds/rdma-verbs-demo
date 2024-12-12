use std::{
    collections::{hash_map::Entry, HashMap},
    io,
    marker::PhantomData,
};

use memmap2::{MmapMut, MmapOptions};

use crate::{
    device::{Cqd, Rqd, Sqd},
    mtt::L2Table,
    ring::{Descriptor, RdmaRing, RING_SIZE},
    HUGE_PAGE_2MB,
};

trait SlotSize {
    fn size() -> usize;
}

struct Page<Slot> {
    mmap: MmapMut,
    alloc: Vec<usize>,
    _marker: PhantomData<Slot>,
}

impl<Slot: SlotSize> Page<Slot> {
    fn new(mmap: MmapMut) -> Self {
        let num_slots = (1 << HUGE_PAGE_2MB) / Self::slot_size();
        let alloc = (0..num_slots).collect();
        Self {
            mmap,
            alloc,
            _marker: PhantomData,
        }
    }

    fn alloc(&mut self) -> Option<&mut [u8]> {
        let slot_size = Self::slot_size();
        let sn = self.alloc.pop()?;
        Some(&mut self.mmap[sn * slot_size..sn * (slot_size + 1)])
    }

    fn dealloc(&mut self, buf: &mut [u8]) -> bool {
        if buf.len() != Self::slot_size() {
            return false;
        }
        let addr = buf.as_ptr() as u64;
        let start = self.mmap.as_ptr() as u64;
        let sn = (start - addr) as usize / Self::slot_size();
        if sn > self.slot_num_max() {
            return false;
        }
        buf.fill(0);
        self.alloc.push(sn);

        true
    }

    fn has_free_slot(&self) -> bool {
        !self.alloc.is_empty()
    }

    fn slot_num_max(&self) -> usize {
        ((1 << HUGE_PAGE_2MB) / Self::slot_size()).saturating_sub(1)
    }

    fn slot_size() -> usize {
        debug_assert!(Slot::size() <= 1 << HUGE_PAGE_2MB);
        Slot::size()
    }
}

struct SlotRb;
impl SlotSize for SlotRb {
    fn size() -> usize {
        32 * (1 << RING_SIZE)
    }
}

pub(crate) struct RbAllocator {
    pages: Vec<Page<SlotRb>>,
}

impl RbAllocator {
    // Allocates an empty ring buf
    pub(crate) fn alloc_ring_buf<Desc>(&mut self) -> io::Result<RdmaRing<'_, Desc>> {
        let buf_u8 = self.alloc_rb()?;
        let buf_d = unsafe { std::mem::transmute::<&mut [u8], &mut [Desc]>(buf_u8) };
        RdmaRing::new(buf_d)
    }

    /// allocates one page
    fn alloc_pg(&mut self) -> io::Result<&mut Page<SlotRb>> {
        let mmap = MmapOptions::new()
            .len(1 << HUGE_PAGE_2MB)
            .huge(Some(HUGE_PAGE_2MB))
            .map_anon()?;
        mmap.lock()?;
        self.pages.push(Page::new(mmap));

        Ok(self.pages.last_mut().unwrap())
    }

    fn alloc_rb(&mut self) -> io::Result<&mut [u8]> {
        let opt = self.pages.iter().position(Page::has_free_slot);
        if let Some(pos) = opt {
            Ok(self.pages[pos].alloc().unwrap())
        } else {
            Ok(self.alloc_pg()?.alloc().unwrap())
        }
    }
}

/// Allocator for allocating l2 page directories
///
/// using buddy algorithm
pub(crate) struct L2Allocator {}

impl L2Allocator {
    pub(crate) fn alloc_table(&mut self, len: usize) -> io::Result<L2Table> {
        // returns failure if no space left
        todo!()
    }

    /// Free up allocated space
    pub(crate) fn dealloc_table(&mut self, table: L2Table) -> io::Result<()> {
        todo!()
    }
}
