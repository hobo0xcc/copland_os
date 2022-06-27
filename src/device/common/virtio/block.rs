#![allow(invalid_value)]

use super::header::*;
use super::queue::*;
use crate::lazy::Lazy;
use crate::KERNEL_LOCK;
use alloc::alloc::{alloc_zeroed, Layout};
use alloc::vec::Vec;
use core::mem;
use core::sync::atomic::{fence, Ordering};
use log::info;
use volatile::ReadWrite;

// https://github.com/mit-pdos/xv6-riscv/blob/riscv/kernel/virtio_disk.c

pub const BLOCK_SIZE: usize = 512;

pub static mut VIRTIO_BLOCK: Lazy<VirtIOBlock<'static>> =
    Lazy::<VirtIOBlock, fn() -> VirtIOBlock<'static>>::new(|| VirtIOBlock::new());

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

#[repr(C)]
pub struct Config {
    capacity: u64,
    // ...
}

pub struct VirtIOBlock<'q> {
    header: &'q mut VirtIORegister,
    config: &'q mut Config,
    pages: *mut u8,
    desc: &'q mut [VirtQueueDesc; DESC_NUM],
    avail: &'q mut VirtQueueAvail,
    used: &'q mut VirtQueueUsed,
    free: [bool; DESC_NUM],
    used_idx: u16,
    requests: [VirtIOBlockReq; DESC_NUM],
    status: [u8; DESC_NUM],
    complete: [ReadWrite<bool>; DESC_NUM],
}

unsafe impl Send for VirtIOBlock<'_> {}
unsafe impl Sync for VirtIOBlock<'_> {}

pub enum BlockOpType {
    Write,
    Read,
}

impl VirtIOBlock<'_> {
    pub fn new() -> Self {
        unsafe { mem::MaybeUninit::zeroed().assume_init() }
    }

    pub unsafe fn init(&mut self, addr: usize) {
        info!("VirtIO init: Start");
        assert_eq!(core::mem::size_of::<VirtIORegister>(), 0x74);
        self.header = (addr as *mut VirtIORegister).as_mut().unwrap();
        self.config = ((addr + 0x100) as *mut Config).as_mut().unwrap();
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
        self.pages = {
            let layout =
                Layout::from_size_align(crate::arch::PAGE_SIZE * 2, crate::arch::PAGE_SIZE)
                    .unwrap();
            alloc_zeroed(layout)
        };

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
        for complete in self.complete.iter_mut() {
            complete.write(false);
        }

        info!("VirtIO init: Succeeded");
    }

    fn alloc_desc(&mut self, num: usize) -> Vec<u16> {
        let mut result = Vec::new();
        for i in 0..DESC_NUM {
            if self.free[i] {
                self.free[i] = false;
                result.push(i as u16);
                if result.len() == num {
                    break;
                }
            }
        }

        if result.len() != num {
            panic!("VirtIO Descriptor allocaiton failed");
        }

        result
    }

    fn free_desc(&mut self, index: u16) {
        self.free[index as usize] = true;
        let chained =
            (self.desc[index as usize].flags & VirtQueueDescFlag::VIRTQ_DESC_F_NEXT as u16) != 0;
        if chained {
            self.free_desc(self.desc[index as usize].next);
        }
    }

    pub fn block_op(&mut self, buf: *mut u8, sector: u64, op: BlockOpType) {
        let indexes = self.alloc_desc(3);
        assert!(indexes.iter().all(|i| *i < DESC_NUM as u16));
        let mut request = self.requests.get_mut(indexes[0] as usize).unwrap();
        match op {
            BlockOpType::Read => request.typ = VirtIOBlockReqType::VIRTIO_BLK_T_IN as u32,
            BlockOpType::Write => request.typ = VirtIOBlockReqType::VIRTIO_BLK_T_OUT as u32,
        }
        request.reserved = 0;
        request.sector = sector;

        self.desc[indexes[0] as usize] = VirtQueueDesc {
            addr: request as *mut VirtIOBlockReq as u64,
            len: mem::size_of::<VirtIOBlockReq>() as u32,
            flags: VirtQueueDescFlag::VIRTQ_DESC_F_NEXT as u16,
            next: indexes[1],
        };

        self.desc[indexes[1] as usize] = VirtQueueDesc {
            addr: buf as u64,
            len: BLOCK_SIZE as u32,
            flags: {
                // TODO: refine
                let mut flags = match op {
                    BlockOpType::Read => VirtQueueDescFlag::VIRTQ_DESC_F_WRITE as u16,
                    BlockOpType::Write => 0,
                };
                flags |= VirtQueueDescFlag::VIRTQ_DESC_F_NEXT as u16;
                flags
            },
            next: indexes[2],
        };

        self.status[indexes[0] as usize] = 0xff;
        self.desc[indexes[2] as usize] = VirtQueueDesc {
            addr: (&self.status[indexes[0] as usize]) as *const u8 as u64,
            len: 1,
            flags: VirtQueueDescFlag::VIRTQ_DESC_F_WRITE as u16,
            next: 0,
        };

        self.complete[indexes[0] as usize].write(false);
        self.avail.ring[self.avail.idx as usize % DESC_NUM] = indexes[0];

        fence(Ordering::SeqCst);
        self.avail.idx += 1;
        fence(Ordering::SeqCst);

        self.header.queue_notify.write(0);
        while !self.complete[indexes[0] as usize].read() {
            unsafe {
                KERNEL_LOCK.wait_intr();
            }
        }

        self.free_desc(indexes[0]);
    }

    pub fn size(&self) -> usize {
        self.config.capacity as usize
    }

    pub fn interrupt(&mut self) {
        self.header
            .interrupt_ack
            .write(self.header.interrupt_status.read() & 0x3);
        fence(Ordering::SeqCst);

        while self.used_idx != self.used.idx {
            fence(Ordering::SeqCst);
            let id = self.used.ring[self.used_idx as usize % DESC_NUM].id as usize;
            if self.status[id] != 0 {
                panic!("VirtIO status");
            }

            self.complete[id].write(true);
            self.used_idx += 1;
        }
    }
}
