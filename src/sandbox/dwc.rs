use crate::device::raspi3b::base::MMIO_BASE;
use crate::*;
use volatile::{ReadWrite, Volatile};

// https://github.com/bztsrc/raspi3-tutorial
pub const USB_CONTROLLER: usize = MMIO_BASE + 0x980000;

#[repr(C)]
pub struct DWCRegister {
    otg_control: ReadWrite<u32>,                  // 0x0
    otg_interrupt: ReadWrite<u32>,                // 0x4
    ahb: ReadWrite<u32>,                          // 0x8
    usb: ReadWrite<u32>,                          // 0xc
    reset: ReadWrite<u32>,                        // 0x10
    interrupt: ReadWrite<u32>,                    // 0x14
    interrupt_mask: ReadWrite<u32>,               // 0x18
    receive_peek: ReadWrite<u32>,                 // 0x1c
    receive_pop: ReadWrite<u32>,                  // 0x20
    receive_size: ReadWrite<u32>,                 // 0x24
    non_periodic_fifo_size: ReadWrite<u32>,       // 0x28
    non_periodic_fifo_status: ReadWrite<u32>,     // 0x2c
    i2c_control: ReadWrite<u32>,                  // 0x30
    phy_vendor_control: ReadWrite<u32>,           // 0x34
    gpio: ReadWrite<u32>,                         // 0x38
    user_id: ReadWrite<u32>,                      // 0x3c
    vendor_id: ReadWrite<u32>,                    // 0x40
    hardware: ReadWrite<u32>,                     // 0x44
    low_power_mode_configuration: ReadWrite<u32>, // 0x48
    _reserved0: [u32; 13],                        // 0x4c ... 0x7c
    mdio_control: ReadWrite<u32>,                 // 0x80
    mdio_read_write: ReadWrite<u32>,              // 0x84
    misc_control: ReadWrite<u32>,                 // 0x88
    _reserved1: [u32; 29],                        // 0x8c ... 0xfc
    periodic_fifo_size: ReadWrite<u32>,           // 0x100
    periodic_fifo_base: ReadWrite<u32>,           // 0x104
    _reserved2: [u32; 830],                       // 0x108 ... 0xdfc
    power: ReadWrite<u32>,                        // 0xe00
}

impl DWCRegister {
    pub fn new<'a>(addr: usize) -> &'a mut Self {
        unsafe { (addr as *mut DWCRegister).as_mut().unwrap() }
    }
}

pub fn dwc() {
    let regs = DWCRegister::new(USB_CONTROLLER);
    assert_eq!(
        (&regs.mdio_control as *const Volatile<u32> as *const u8 as usize) & 0xff,
        0x80
    );
    assert_eq!(
        (&regs.periodic_fifo_size as *const Volatile<u32> as *const u8 as usize) & 0x1ff,
        0x100
    );
    assert_eq!(
        (&regs.power as *const Volatile<u32> as *const u8 as usize) & 0xfff,
        0xe00
    );
    println!("vendor_id: {:#x}", regs.vendor_id.read());
    println!("DWC OK!");
}
