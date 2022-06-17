// #define VIDEOCORE_MBOX  (MMIO_BASE+0x0000B880)
// #define MBOX_READ       ((volatile unsigned int*)(VIDEOCORE_MBOX+0x0))
// #define MBOX_POLL       ((volatile unsigned int*)(VIDEOCORE_MBOX+0x10))
// #define MBOX_SENDER     ((volatile unsigned int*)(VIDEOCORE_MBOX+0x14))
// #define MBOX_STATUS     ((volatile unsigned int*)(VIDEOCORE_MBOX+0x18))
// #define MBOX_CONFIG     ((volatile unsigned int*)(VIDEOCORE_MBOX+0x1C))
// #define MBOX_WRITE      ((volatile unsigned int*)(VIDEOCORE_MBOX+0x20))
// #define MBOX_RESPONSE   0x80000000
// #define MBOX_FULL       0x80000000
// #define MBOX_EMPTY      0x40000000

use super::base::*;
use core::arch::asm;

pub const VIDEOCORE_MBOX: usize = MMIO_BASE + 0x0000B880;
pub const MBOX_READ: usize = VIDEOCORE_MBOX + 0x0;
pub const MBOX_POLL: usize = VIDEOCORE_MBOX + 0x10;
pub const MBOX_SENDER: usize = VIDEOCORE_MBOX + 0x14;
pub const MBOX_STATUS: usize = VIDEOCORE_MBOX + 0x18;
pub const MBOX_CONFIG: usize = VIDEOCORE_MBOX + 0x1c;
pub const MBOX_WRITE: usize = VIDEOCORE_MBOX + 0x20;
pub const MBOX_RESPONSE: u32 = 0x80000000;
pub const MBOX_FULL: u32 = 0x80000000;
pub const MBOX_EMPTY: u32 = 0x40000000;

// extern volatile unsigned int mbox[36];

// #define MBOX_REQUEST    0
pub const MBOX_REQUEST: u32 = 0;

// /* channels */
// #define MBOX_CH_POWER   0
pub const MBOX_CH_POWER: u8 = 0;
// #define MBOX_CH_FB      1
pub const MBOX_CH_FB: u8 = 1;
// #define MBOX_CH_VUART   2
pub const MBOX_CH_VUART: u8 = 2;
// #define MBOX_CH_VCHIQ   3
pub const MBOX_CH_VCHIQ: u8 = 3;
// #define MBOX_CH_LEDS    4
pub const MBOX_CH_LEDS: u8 = 4;
// #define MBOX_CH_BTNS    5
pub const MBOX_CH_BTNS: u8 = 5;
// #define MBOX_CH_TOUCH   6
pub const MBOX_CH_TOUCH: u8 = 6;
// #define MBOX_CH_COUNT   7
pub const MBOX_CH_COUNT: u8 = 7;
// #define MBOX_CH_PROP    8
pub const MBOX_CH_PROP: u8 = 8;

// /* tags */
// #define MBOX_TAG_GETSERIAL      0x10004
pub const MBOX_TAG_GETSERIAL: u32 = 0x10004;
// #define MBOX_TAG_LAST           0
pub const MBOX_TAG_LAST: u32 = 0;

pub static mut MBOX: MBox = MBox { mbox: [0; 32] };

#[repr(align(16))]
#[repr(C)]
pub struct MBox {
    pub mbox: [u32; 32],
}

impl MBox {
    fn read_reg(addr: usize) -> u32 {
        unsafe { *(addr as *const u32) }
    }

    fn write_reg(addr: usize, data: u32) {
        unsafe {
            *(addr as *mut u32) = data;
        }
    }

    pub fn mbox_call(&mut self, channel: u8) -> bool {
        let mbox_addr: u64 = (&self.mbox[0]) as *const u32 as u64;
        let r: u32 = (mbox_addr | ((channel & 0xf) as u64)) as u32;
        while Self::read_reg(MBOX_STATUS) & MBOX_FULL != 0 {
            unsafe {
                asm!("nop");
            }
        }

        Self::write_reg(MBOX_WRITE, r);
        loop {
            // loop until arrival of response
            while Self::read_reg(MBOX_STATUS) & MBOX_EMPTY != 0 {
                unsafe {
                    asm!("nop");
                }
            }
            if r == Self::read_reg(MBOX_READ) {
                return self.mbox[1] == MBOX_RESPONSE;
            }
        }
    }
}
