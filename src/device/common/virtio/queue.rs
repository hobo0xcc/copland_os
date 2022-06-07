pub const DESC_NUM: usize = 8;

#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum VirtQueueDescFlag {
    VIRTQ_DESC_F_NEXT = 1,
    VIRTQ_DESC_F_WRITE = 2,
    VIRTQ_DESC_F_INDIRECT = 4,
}

#[repr(C)]
pub struct VirtQueueDesc {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

#[repr(C)]
pub struct VirtQueueAvail {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; DESC_NUM],
    pub unused: u16,
}

#[repr(C)]
pub struct VirtQueueUsedElem {
    pub id: u32,
    pub len: u32,
}

#[repr(C)]
pub struct VirtQueueUsed {
    pub flags: u16,
    pub idx: u16,
    pub ring: [VirtQueueUsedElem; DESC_NUM],
}
