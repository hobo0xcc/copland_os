pub mod buffer;
pub mod fat32;

pub trait Size {
    fn size(&self) -> usize;
}
