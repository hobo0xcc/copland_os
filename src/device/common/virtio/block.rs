#![allow(invalid_value)]

use super::header::VirtIORegister;
use super::queue::*;
use core::mem;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref VIRTIO_BLOCK: Mutex<VirtIOBlock<'static>> = Mutex::new(VirtIOBlock::new());
}

#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum VirtIOBlockReqType {
    VIRTIO_BLK_T_IN = 0,
    VIRTIO_BLK_T_OUT = 1,
}

#[repr(C)]
pub struct VirtIOBlockReq {
    typ: u32,
    reserved: u32,
    sector: u64,
}

pub struct VirtIOBlock<'q> {
    header: &'q mut VirtIORegister,
    pages: *mut u8,
    desc: &'q mut VirtQueueDesc,
    avail: &'q mut VirtQueueAvail,
    used: &'q mut VirtQueueUsed,
    free: [bool; DESC_NUM],
    used_idx: u16,
    requests: [VirtIOBlockReq; DESC_NUM],
}

unsafe impl Send for VirtIOBlock<'_> {}
unsafe impl Sync for VirtIOBlock<'_> {}

impl VirtIOBlock<'_> {
    pub fn new() -> Self {
        unsafe { mem::MaybeUninit::zeroed().assume_init() }
    }

    pub unsafe fn init(&mut self, addr: usize) {
        self.header = (addr as *mut VirtIORegister).as_mut().unwrap();
        if self.header.magic_value.read() != 0x74726976
            || self.header.version.read() != 1
            || self.header.device_id.read() != 2
            || self.header.vendor_id.read() != 0x554d4551
        {
            panic!("Invalid VirtIO Block: {:#x}", addr);
        }
    }
}
