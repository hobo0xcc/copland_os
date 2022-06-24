use super::mailbox::*;
use crate::lazy::Lazy;

pub static mut FRAMEBUFFER: Lazy<FrameBuffer> = Lazy::new(|| {
    let mut fb = FrameBuffer {
        width: 0,
        height: 0,
        pitch: 0,
        isrgb: 0,
        // uninitialized
        fb: 0x12341234_usize as *mut u32,
    };
    fb.init();
    fb
});

pub struct FrameBuffer {
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub isrgb: u32,
    pub fb: *mut u32,
}

impl FrameBuffer {
    pub fn init(&mut self) {
        unsafe {
            MBOX.mbox[0] = 35 * 4;
            MBOX.mbox[1] = MBOX_REQUEST;

            // tag
            MBOX.mbox[2] = MailBoxTag::SetPhysicalSize as u32;
            // request size
            MBOX.mbox[3] = 8;
            // response size
            MBOX.mbox[4] = 8;
            // data
            MBOX.mbox[5] = 1024;
            MBOX.mbox[6] = 768;
            // data end

            MBOX.mbox[7] = MailBoxTag::SetVirtualSize as u32;
            MBOX.mbox[8] = 8;
            MBOX.mbox[9] = 8;
            MBOX.mbox[10] = 1024;
            MBOX.mbox[11] = 768;

            MBOX.mbox[12] = MailBoxTag::SetVirtualOffset as u32;
            MBOX.mbox[13] = 8;
            MBOX.mbox[14] = 8;
            MBOX.mbox[15] = 0;
            MBOX.mbox[16] = 0;

            MBOX.mbox[17] = MailBoxTag::SetDepth as u32;
            MBOX.mbox[18] = 4;
            MBOX.mbox[19] = 4;
            MBOX.mbox[20] = 32;

            MBOX.mbox[21] = MailBoxTag::SetPixelOrder as u32;
            MBOX.mbox[22] = 4;
            MBOX.mbox[23] = 4;
            MBOX.mbox[24] = 1;

            MBOX.mbox[25] = MailBoxTag::AllocateBuffer as u32;
            MBOX.mbox[26] = 8;
            MBOX.mbox[27] = 8;
            MBOX.mbox[28] = 4096;
            MBOX.mbox[29] = 0;

            MBOX.mbox[30] = MailBoxTag::GetPitch as u32;
            MBOX.mbox[31] = 4;
            MBOX.mbox[32] = 4;
            MBOX.mbox[33] = 0;

            MBOX.mbox[34] = MailBoxTag::MBoxTagLast as u32;

            if MBOX.mbox_call(MBOX_CH_PROP) && MBOX.mbox[20] == 32 && MBOX.mbox[28] != 0 {
                MBOX.mbox[28] &= 0x3FFFFFFF; // gpu addr -> cpu addr
                self.width = MBOX.mbox[5];
                self.height = MBOX.mbox[6];
                self.pitch = MBOX.mbox[33];
                self.isrgb = MBOX.mbox[24];
                self.fb = (MBOX.mbox[28] as usize) as *mut u32;
            } else {
                panic!("failed to init framebuffer");
            }
        }
    }
}
