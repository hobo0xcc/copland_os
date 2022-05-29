use volatile::{ReadOnly, ReadWrite, WriteOnly};

#[repr(packed)]
#[allow(invalid_value)]
pub struct VirtIORegister {
    pub magic_value: ReadOnly<u32>,
    pub version: ReadOnly<u32>,
    pub device_id: ReadOnly<u32>,
    pub vendor_id: ReadOnly<u32>,
    pub host_features: ReadOnly<u32>,
    pub host_features_sel: WriteOnly<u32>,
    pub _reserved_1: WriteOnly<u64>,
    pub guest_features: WriteOnly<u32>,
    pub guest_features_sel: WriteOnly<u32>,
    pub guest_page_size: WriteOnly<u32>,
    pub _reserved_2: WriteOnly<u32>,
    pub queue_sel: WriteOnly<u32>,
    pub queue_num_max: ReadOnly<u32>,
    pub queue_num: WriteOnly<u32>,
    pub queue_align: WriteOnly<u32>,
    pub queue_pfn: ReadWrite<u32>,
    pub _reserved_3: WriteOnly<u64>,
    pub _reserved_4: WriteOnly<u32>,
    pub queue_notify: WriteOnly<u32>,
    pub _reserved_5: WriteOnly<u64>,
    pub _reserved_6: WriteOnly<u32>,
    pub interrupt_status: ReadOnly<u32>,
    pub interrupt_ack: WriteOnly<u32>,
    pub _reserved_7: WriteOnly<u64>,
    pub status: ReadWrite<u32>,
}
