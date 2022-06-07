use volatile::{ReadOnly, ReadWrite, WriteOnly};

#[repr(C)]
#[allow(invalid_value)]
pub struct VirtIORegister {
    pub magic_value: ReadOnly<u32>,
    pub version: ReadOnly<u32>,
    pub device_id: ReadOnly<u32>,
    pub vendor_id: ReadOnly<u32>,
    pub host_features: ReadOnly<u32>,
    pub host_features_sel: WriteOnly<u32>,
    pub _reserved_1: WriteOnly<u32>,
    pub _reserved_2: WriteOnly<u32>,
    pub guest_features: WriteOnly<u32>,
    pub guest_features_sel: WriteOnly<u32>,
    pub guest_page_size: WriteOnly<u32>,
    pub _reserved_3: WriteOnly<u32>,
    pub queue_sel: WriteOnly<u32>,
    pub queue_num_max: ReadOnly<u32>,
    pub queue_num: WriteOnly<u32>,
    pub queue_align: WriteOnly<u32>,
    pub queue_pfn: ReadWrite<u32>,
    pub _reserved_4: WriteOnly<u32>,
    pub _reserved_5: WriteOnly<u32>,
    pub _reserved_6: WriteOnly<u32>,
    pub queue_notify: WriteOnly<u32>,
    pub _reserved_7: WriteOnly<u32>,
    pub _reserved_8: WriteOnly<u32>,
    pub _reserved_9: WriteOnly<u32>,
    pub interrupt_status: ReadOnly<u32>,
    pub interrupt_ack: WriteOnly<u32>,
    pub _reserved_10: WriteOnly<u32>,
    pub _reserved_11: WriteOnly<u32>,
    pub status: ReadWrite<u32>,
}

// https://docs.oasis-open.org/virtio/virtio/v1.1/csprd01/virtio-v1.1-csprd01.html#x1-100001
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum VirtIODeviceStatus {
    ACKNOWLEDGE = 1,
    DRIVER = 2,
    FAILED = 128,
    FEATURES_OK = 8,
    DRIVER_OK = 4,
    DEVICE_NEEDS_RESET = 64,
}
