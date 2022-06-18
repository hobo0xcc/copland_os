use super::base::*;
use core::arch::asm;

// https://github.com/BrianSidebotham/arm-tutorial-rpi/blob/master/part-5/armc-017/rpi-mailbox-interface.h

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

pub const MBOX_REQUEST: u32 = 0;

pub const MBOX_CH_POWER: u8 = 0;
pub const MBOX_CH_FB: u8 = 1;
pub const MBOX_CH_VUART: u8 = 2;
pub const MBOX_CH_VCHIQ: u8 = 3;
pub const MBOX_CH_LEDS: u8 = 4;
pub const MBOX_CH_BTNS: u8 = 5;
pub const MBOX_CH_TOUCH: u8 = 6;
pub const MBOX_CH_COUNT: u8 = 7;
pub const MBOX_CH_PROP: u8 = 8;

pub const MBOX_TAG_GETSERIAL: u32 = 0x10004;
pub const MBOX_TAG_LAST: u32 = 0;

// https://github.com/BrianSidebotham/arm-tutorial-rpi/blob/master/part-5/armc-017/rpi-mailbox-interface.h
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MailBoxTag {
    MBoxTagLast = 0x0,
    GetFirmwareVersion = 0x1,

    // Hardware
    GetBoardModel = 0x10001,
    GetBoardRevision = 0x10002,
    GetBoardMacAddress = 0x10003,
    GetBoardSerial = 0x10004,
    GetArmMemory = 0x10005,
    GetVCMemory = 0x10006,
    GetClocks = 0x10007,

    // Config
    GetCommandLine = 0x50001,

    // Shared resource management
    GetDMAChannels = 0x60001,

    // Power
    GetPowerState = 0x20001,
    GetTiming = 0x20002,
    SetPowerState = 0x28001,

    // Clocks
    GetClockState = 0x30001,
    SetClockState = 0x38001,
    GetClockRate = 0x30002,
    SetClockRate = 0x38002,
    GetMaxClockRate = 0x30004,
    GetMinClockRate = 0x30007,
    GetTurbo = 0x30009,
    SetTurbo = 0x38009,

    // Voltage
    GetVoltage = 0x30003,
    SetVoltage = 0x38003,
    GetMaxVoltage = 0x30005,
    GetMinVoltage = 0x30008,
    GetTemperature = 0x30006,
    GetMaxTemperature = 0x3000A,
    AllocateMemory = 0x3000C,
    LockMemory = 0x3000D,
    UnlockMemory = 0x3000E,
    ReleaseMemory = 0x3000F,
    ExecuteCode = 0x30010,
    GetDispmanxMemHandle = 0x30014,
    GetEDIDBlock = 0x30020,

    // Framebuffer
    AllocateBuffer = 0x40001,
    ReleaseBuffer = 0x48001,
    BlankScreen = 0x40002,
    GetPhysicalSize = 0x40003,
    TestPhysicalSize = 0x44003,
    SetPhysicalSize = 0x48003,
    GetVirtualSize = 0x40004,
    TestVirtualSie = 0x44004,
    SetVirtualSize = 0x48004,
    GetDepth = 0x40005,
    TestDepth = 0x44005,
    SetDepth = 0x48005,
    GetPixelOrder = 0x40006,
    TestPixelOrder = 0x44006,
    SetPixelOrder = 0x48006,
    GetAlphaMode = 0x40007,
    TestAlphaMode = 0x44007,
    SetAlphaMode = 0x48007,
    GetPitch = 0x40008,
    GetVirtualOffset = 0x40009,
    TestVirtualOffset = 0x44009,
    SetVirtualOffset = 0x48009,
    GerOverscan = 0x4000A,
    TestOverscan = 0x4400A,
    SetOverscan = 0x4800A,
    GetPalette = 0x4000B,
    TestPalette = 0x4400B,
    SetPalette = 0x4800B,
    SetCursorInfo = 0x8011,
    SetCursorState = 0x8010,
}

pub static mut MBOX: MBox = MBox { mbox: [0; 1024] };

#[repr(align(16))]
#[repr(C)]
pub struct MBox {
    pub mbox: [u32; 1024],
}

impl MBox {
    fn read_reg(addr: usize) -> u32 {
        unsafe { (addr as *const u32).read_volatile() }
    }

    fn write_reg(addr: usize, data: u32) {
        unsafe {
            (addr as *mut u32).write_volatile(data);
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
