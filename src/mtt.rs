use std::io;

const L1_SIZE: u8 = 20;

pub(crate) struct Mtt {
    l1_table: [L1Entry; 1 << L1_SIZE],
}

impl Mtt {
    pub(crate) fn entries(&self) -> &[L1Entry] {
        &self.l1_table
    }

    pub(crate) fn entries_mut(&mut self) -> &mut [L1Entry] {
        &mut self.l1_table
    }
}

#[repr(C)] // 24bytes
pub(crate) struct L1Entry {
    key: u32,
    offset: u32,
    size: u32,
    base_va: u64,
}

impl L1Entry {
    pub(crate) fn new(key: u32, offset: u32, size: u32, base_va: u64) -> Self {
        Self {
            key,
            offset,
            size,
            base_va,
        }
    }

    pub(crate) fn key(&self) -> u32 {
        self.key
    }

    pub(crate) fn offset(&self) -> u32 {
        self.offset
    }

    pub(crate) fn size(&self) -> u32 {
        self.size
    }

    pub(crate) fn base_va(&self) -> u64 {
        self.base_va
    }
}

pub(crate) struct L2Table {
    inner: Vec<L2Entry>,
}

impl L2Table {
    pub(crate) fn entries(&self) -> &[L2Entry] {
        &self.inner
    }

    pub(crate) fn entries_mut(&mut self) -> &mut [L2Entry] {
        &mut self.inner
    }
}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct L2Entry {
    pa: u64,
}

impl L2Entry {
    pub(crate) fn pa(&self) -> u64 {
        self.pa
    }

    pub(crate) fn set_pa(&mut self, pa: u64) {
        self.pa = pa
    }
}
