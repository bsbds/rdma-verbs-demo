use std::io;

use crate::v2p::va_to_pa;

pub(crate) const RING_SIZE: u8 = 10;
pub(crate) const DESC_SIZE: u8 = 5;

const SIZE: u32 = 1 << (DESC_SIZE + RING_SIZE);
const SIZE_MASK: u32 = SIZE - 1;

pub(crate) trait Descriptor {
    fn f_valid(&self) -> bool;
}

pub(crate) struct RingCtx {
    head: u32,
    tail: u32,
    pa: u64,
}

impl RingCtx {
    pub(crate) fn new(pa: u64) -> Self {
        Self {
            head: 0,
            tail: 0,
            pa,
        }
    }

    pub(crate) fn head(&self) -> u32 {
        self.head
    }

    pub(crate) fn tail(&self) -> u32 {
        self.tail
    }

    pub(crate) fn head_idx(&self) -> usize {
        self.head as usize >> DESC_SIZE
    }

    pub(crate) fn tail_idx(&self) -> usize {
        self.tail as usize >> DESC_SIZE
    }

    pub(crate) fn inc_head(&mut self) {
        Self::inc_wrap(&mut self.head)
    }

    pub(crate) fn inc_tail(&mut self) {
        Self::inc_wrap(&mut self.tail)
    }

    fn inc_wrap(x: &mut u32) {
        *x = (*x + (1 << DESC_SIZE)) & SIZE_MASK;
    }
}

// approximately 24KB
pub(crate) struct RdmaRing<'buf, Desc> {
    ctx: RingCtx,
    buf: &'buf mut [Desc],
}

impl<'buf, Desc> RdmaRing<'buf, Desc> {
    pub(crate) fn new(buf: &'buf mut [Desc]) -> io::Result<Self> {
        debug_assert_eq!(buf.len(), 1 << RING_SIZE);
        let pa = va_to_pa(buf.as_ptr() as u64)?;

        Ok(Self {
            ctx: RingCtx::new(pa),
            buf,
        })
    }
}

impl<Desc: Descriptor> RdmaRing<'_, Desc> {
    /// Appends some descriptors to the ring buffer
    pub(crate) fn produce(&mut self, descs: Vec<Desc>) -> io::Result<()> {
        if self.num_free() < descs.len() {
            return Err(io::Error::from(io::ErrorKind::WouldBlock));
        }
        let ptr = self.buf.as_mut_ptr();
        for entry in descs {
            self.buf[self.ctx.tail_idx()] = entry;
            self.ctx.inc_tail();
        }

        Ok(())
    }

    /// Tries to poll next valid entry from the queue
    pub(crate) fn consume(&mut self) -> Option<&Desc> {
        let head = self.ctx.head_idx();
        let ready = self.buf[head].f_valid();
        ready.then(|| {
            self.ctx.inc_head();
            &self.buf[head]
        })
    }

    /// Gets number of free slots
    fn num_free(&self) -> usize {
        let dlt = self.ctx.head().wrapping_sub(self.ctx.tail());
        let bytes = SIZE - (dlt & SIZE_MASK);
        (bytes >> DESC_SIZE) as usize
    }
}

// 24 Bytes
#[repr(C)]
struct QueueEntry {
    /// Operations code
    opcode: u32,
    /// Stores something like valid bit
    flags: u32,
    /// Stores memory segment for rdma operations
    sg_list: *mut IbvSge,
    /// Num segments
    sg_len: u32,
}

/// Scatter/Gather Element
struct IbvSge {
    addr: u64,
    length: u64,
    lkey: u32,
}
