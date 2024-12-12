#![allow(unused)]

mod allocator;

mod device;

mod verbs;

mod ring;

mod mtt;

mod v2p;

use std::io;

pub(crate) const HUGE_PAGE_2MB: u8 = 21;
