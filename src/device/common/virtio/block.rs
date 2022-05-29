#![allow(invalid_value)]

use super::header::*;
use super::queue::*;
use alloc::alloc::{alloc_zeroed, Layout};
use core::mem;
use lazy_static::lazy_static;
use log::info;
use spin::Mutex;

lazy_static! {
    pub static ref VIRTIO_BLOCK: Mutex<VirtIOBlock<'static>> = Mutex::new(VirtIOBlock::new());
}

#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum VirtIOBlockReqType {
    VIRTIO_BLK_T_IN = 0,
    VIRTIO_BLK_T_OUT = 1,
}

#[allow(non_camel_case_types)]
#[repr(usize)]
enum VirtIODeviceFeature {
    VIRTIO_BLK_F_RO = 5,
    VIRTIO_BLK_F_SCSI = 7,
    VIRTIO_BLK_F_CONFIG_WCE = 11,
    VIRTIO_BLK_F_MQ = 12,
    VIRTIO_F_ANY_LAYOUT = 27,
    VIRTIO_RING_F_INDIRECT_DESC = 28,
    VIRTIO_RING_F_EVENT_IDX = 29,
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
    desc: &'q mut [VirtQueueDesc; DESC_NUM],
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
        info!("VirtIO init: Start");
        self.header = (addr as *mut VirtIORegister).as_mut().unwrap();
        if self.header.magic_value.read() != 0x74726976
            || self.header.version.read() != 1
            || self.header.device_id.read() != 2
            || self.header.vendor_id.read() != 0x554d4551
        {
            panic!("Invalid VirtIO Block: {:#x}", addr);
        }

        info!("Found valid VirtIO Block device");

        let mut status = VirtIODeviceStatus::ACKNOWLEDGE as u32;
        self.header.status.write(status);
        info!("VirtIO Acknowledge");

        status |= VirtIODeviceStatus::DRIVER as u32;
        self.header.status.write(status);
        {
            use VirtIODeviceFeature::*;
            let mut features = self.header.host_features.read();
            features &= !(1 << VIRTIO_BLK_F_RO as usize);
            features &= !(1 << VIRTIO_BLK_F_SCSI as usize);
            features &= !(1 << VIRTIO_BLK_F_CONFIG_WCE as usize);
            features &= !(1 << VIRTIO_BLK_F_MQ as usize);
            features &= !(1 << VIRTIO_F_ANY_LAYOUT as usize);
            features &= !(1 << VIRTIO_RING_F_EVENT_IDX as usize);
            features &= !(1 << VIRTIO_RING_F_INDIRECT_DESC as usize);
            self.header.guest_features.write(features);
        }

        status |= VirtIODeviceStatus::FEATURES_OK as u32;
        self.header.status.write(status);
        info!("VirtIO features OK");

        status |= VirtIODeviceStatus::DRIVER_OK as u32;
        self.header.status.write(status);
        info!("VirtIO driver OK");

        self.header
            .guest_page_size
            .write(crate::arch::PAGE_SIZE as u32);

        self.header.queue_sel.write(0);
        let max = self.header.queue_num_max.read();
        if max == 0 {
            panic!("VirtIO Disk has no queue 0");
        }
        if max < DESC_NUM as u32 {
            panic!("VirtIO Disk max queue too short");
        }
        self.header.queue_num.write(DESC_NUM as u32);

        // Allocate queue pages
        let mut pages = {
            let mut layout =
                Layout::from_size_align(crate::arch::PAGE_SIZE * 2, crate::arch::PAGE_SIZE)
                    .unwrap();
            alloc_zeroed(layout)
        };
        self.pages = pages;

        self.header
            .queue_pfn
            .write((self.pages as usize >> crate::arch::PAGE_SHIFT) as u32);
        self.desc = (self.pages as *mut [VirtQueueDesc; DESC_NUM])
            .as_mut()
            .unwrap();
        self.avail = (self.pages.add(DESC_NUM * mem::size_of::<VirtQueueDesc>())
            as *mut VirtQueueAvail)
            .as_mut()
            .unwrap();
        self.used = (self.pages.add(crate::arch::PAGE_SIZE) as *mut VirtQueueUsed)
            .as_mut()
            .unwrap();

        for free in self.free.iter_mut() {
            *free = true;
        }

        info!("VirtIO init: Succeeded");
    }
}
