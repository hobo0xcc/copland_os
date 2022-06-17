use super::mailbox::*;

pub struct FrameBuffer {
    width: u32,
    height: u32,
    pitch: u32,
    isrgb: u32,
    fb: *mut u8,
}

impl FrameBuffer {
    pub fn init(&mut self) {
        unsafe {
            MBOX.mbox[0] = 35 * 4;
        }
    }
}
