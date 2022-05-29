pub const DESC_NUM: usize = 8;

#[repr(C)]
pub struct VirtQueueDesc {
    addr: u64,
    len: u32,
    flags: u16,
    nex: u16,
}

#[repr(C)]
pub struct VirtQueueAvail {
    flags: u16,
    idx: u16,
    ring: [u16; DESC_NUM],
    unused: u16,
}

#[repr(C)]
pub struct VirtQueueUsedElem {
    id: u32,
    len: u32,
}

#[repr(C)]
pub struct VirtQueueUsed {
    flags: u16,
    idx: u16,
    ring: [VirtQueueUsedElem; DESC_NUM],
}
