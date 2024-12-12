use std::io;

use crate::{
    allocator::{L2Allocator, RbAllocator},
    mtt::{L1Entry, L2Entry, L2Table, Mtt},
    ring::RdmaRing,
    v2p::va_to_pa,
    verbs::{IbvAccess, IbvBuf, IbvMr},
    HUGE_PAGE_2MB,
};

/// 'bufs are allocated by `RbAllocator`
pub(crate) struct RdmaDevice<'buf> {
    fd: Fd,
    bar: Bar,
    l2_allocator: L2Allocator,
    /// Queue pairs
    qps: Vec<QueuePair<'buf>>,
    cqs: Vec<CompletionQueue<'buf>>,
    /// The Memory translate table
    mtt: Mtt,
}

struct Fd;

struct Bar;

/// Send Queue Descriptor
pub(crate) struct Sqd {
    f: [u64; 4],
}

/// Receive Queue Descriptor
pub(crate) struct Rqd {
    f: [u64; 4],
}

/// Completion Queue Descriptor
pub(crate) struct Cqd {
    f: [u64; 4],
}

struct QueuePair<'buf> {
    send: RdmaRing<'buf, Sqd>,
    recv: RdmaRing<'buf, Rqd>,
}

struct CompletionQueue<'buf> {
    inner: RdmaRing<'buf, Cqd>,
}

impl RdmaDevice<'_> {
    /// Opens the device
    fn open() -> Self {
        // opens fd
        // mmap bar space
        todo!()
    }

    fn create_qp(&mut self) -> &mut QueuePair {
        // use allocator to create queue
        todo!()
    }

    fn create_cq(&mut self) -> &mut CompletionQueue {
        // use allocator to create queue
        todo!()
    }
}

struct MergedPages {
    /// start virtual address
    va: u64,
    /// start physical address
    pa: u64,
    /// num pages
    num_pages: u32,
}

const MTT_BASE: u64 = 0;

impl RdmaDevice<'_> {
    /// Registers a memory region
    pub(crate) fn reg_mr(&mut self, buf: IbvBuf, access: u32) -> io::Result<IbvMr> {
        let IbvBuf { addr, length } = buf;
        let num_pages = length >> HUGE_PAGE_2MB;
        let vas: Vec<_> = (addr..addr + length as u64)
            .step_by(1 << HUGE_PAGE_2MB)
            .collect();
        let pas: Vec<_> = vas
            .iter()
            .copied()
            .map(va_to_pa)
            .collect::<Result<_, _>>()?;
        let lkey = self.gen_key();
        let rkey = self.gen_key();
        let l1_index = (lkey >> 8) as usize;
        let num_pages = vas.len();
        let mut l2_table = self.l2_allocator.alloc_table(num_pages)?;
        for (entry, pa) in l2_table.entries_mut().iter_mut().zip(pas) {
            entry.set_pa(pa);
        }
        let l2_table_addr = (std::ptr::addr_of!(l2_table)) as u64;
        let offset = (l2_table_addr - MTT_BASE) / 8;
        let entry = L1Entry::new(lkey, offset as u32, num_pages as u32, addr);
        self.mtt.entries_mut()[l1_index] = entry;

        Ok(IbvMr::new(addr, length as u32, access, lkey, rkey))
    }

    pub(crate) fn dereg_mr(&mut self, mr: IbvMr) {
        let vas: Vec<_> = (mr.addr()..mr.addr() + mr.length() as u64)
            .step_by(1 << HUGE_PAGE_2MB)
            .collect();

        self.walk(mr.addr(), mr.lkey());
        // free the table
        todo!()
    }

    /// Gets the
    fn walk(&self, va: u64, key: u32) -> Option<u64> {
        let l1_index = (key >> 8) as usize;
        let entry = self.mtt.entries().get(l1_index)?;
        let page_index = (va - entry.base_va()) >> HUGE_PAGE_2MB;
        if !Self::validates_entry(entry, page_index) {
            return None;
        }
        let l2_table = Self::get_l2_table(MTT_BASE + entry.offset() as u64 * 8);
        l2_table.entries().get(page_index as usize).map(L2Entry::pa)
    }

    fn get_l2_table(addr: u64) -> &'static mut L2Table {
        todo!()
    }

    fn validates_entry(entry: &L1Entry, page_offset: u64) -> bool {
        // size used as validation page present
        entry.size() != 0 && page_offset < entry.size() as u64
    }

    /// Generates a lkey/rkey
    fn gen_key(&self) -> u32 {
        todo!()
    }
}
