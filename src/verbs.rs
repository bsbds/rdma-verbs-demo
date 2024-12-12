use crate::device::RdmaDevice;

pub trait IbVerbs {
    /// Register the buf region
    fn ibv_reg_mr(&self, pd: &mut IbvPd, buf: IbvBuf, access: IbvAccess) -> IbvMr;
}

pub struct IbvMr {
    addr: u64,
    length: u32,
    access: u32,
    lkey: u32,
    rkey: u32,
}

impl IbvMr {
    pub fn new(addr: u64, length: u32, access: u32, lkey: u32, rkey: u32) -> Self {
        Self {
            addr,
            length,
            access,
            lkey,
            rkey,
        }
    }

    pub fn addr(&self) -> u64 {
        self.addr
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn access(&self) -> u32 {
        self.access
    }

    pub fn lkey(&self) -> u32 {
        self.lkey
    }

    pub fn rkey(&self) -> u32 {
        self.rkey
    }
}

pub struct IbvPd<'buf> {
    dev: RdmaDevice<'buf>,
}

pub struct IbvBuf {
    pub addr: u64,
    pub length: usize,
}

#[repr(u8)]
pub enum IbvAccess {
    LocalWrite,
    RemoteWrite,
    RemoteRead,
    RemoteAtomic,
    // ...
}
