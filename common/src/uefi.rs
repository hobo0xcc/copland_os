pub const PAGE_SIZE: usize = 4096;

#[repr(C)]
pub struct MemoryRegion {
    pub start: usize,
    pub end: usize,
}

#[repr(C)]
pub struct MemoryMap {
    pub descriptors: *const MemoryRegion,
    pub len: usize,
}

#[repr(usize)]
pub enum FrameBufferFormat {
    RGB,
    BGR,
}

#[repr(C)]
pub struct FrameBuffer {
    pub ptr: *mut u32,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub format: FrameBufferFormat,
}
