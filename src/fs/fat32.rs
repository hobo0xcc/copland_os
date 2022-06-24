use crate::device::common::virtio::block::{BlockOpType, VirtIOBlock, BLOCK_SIZE, VIRTIO_BLOCK};
use crate::sync::lazy::Lazy;
use core::ops::DerefMut;
use fatfs::{IoBase, IoError, Read, Seek, Write};

pub static mut FILE_SYSTEM: Lazy<
    fatfs::FileSystem<Disk, fatfs::NullTimeProvider, fatfs::LossyOemCpConverter>,
> = Lazy::<
    fatfs::FileSystem<Disk, fatfs::NullTimeProvider, fatfs::LossyOemCpConverter>,
    fn() -> fatfs::FileSystem<Disk<'static>, fatfs::NullTimeProvider, fatfs::LossyOemCpConverter>,
>::new(|| unsafe {
    fatfs::FileSystem::new(Disk::new(VIRTIO_BLOCK.deref_mut()), fatfs::FsOptions::new()).unwrap()
});

#[derive(Debug)]
pub enum DiskError {
    Dummy,
}

impl IoError for DiskError {
    fn is_interrupted(&self) -> bool {
        false
    }

    fn new_unexpected_eof_error() -> Self {
        Self::Dummy
    }

    fn new_write_zero_error() -> Self {
        Self::Dummy
    }
}

pub struct Disk<'r> {
    raw: &'r mut VirtIOBlock<'r>,
    pos: usize,
}

impl<'r> Disk<'r> {
    pub fn new(raw: &'r mut VirtIOBlock<'r>) -> Self {
        Self { raw, pos: 0 }
    }
}

impl<'r> IoBase for Disk<'r> {
    type Error = DiskError;
}

impl<'r> Read for Disk<'r> {
    // TODO: refine
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let len = buf.len() / BLOCK_SIZE + (if buf.len() % BLOCK_SIZE != 0 { 1 } else { 0 });
        let sector = self.pos / BLOCK_SIZE;
        let max_size = self.raw.size() * BLOCK_SIZE;
        let mut read_count: usize = 0;
        let mut tmp_buf: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
        for i in sector..(sector + len) {
            if self.pos >= max_size {
                break;
            }
            self.raw
                .block_op(tmp_buf.as_mut_ptr(), i as u64, BlockOpType::Read);
            let start = self.pos % BLOCK_SIZE;
            let copy_amount = usize::min(buf.len() - read_count, BLOCK_SIZE - start);
            unsafe {
                core::ptr::copy_nonoverlapping(
                    tmp_buf.as_ptr().add(start),
                    buf.as_mut_ptr().add(read_count),
                    copy_amount,
                );
            }
            self.pos += copy_amount;
            read_count += copy_amount;
        }

        Ok(read_count)
    }
}

impl<'r> Write for Disk<'r> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let len = buf.len() / BLOCK_SIZE + (if buf.len() % BLOCK_SIZE != 0 { 1 } else { 0 });
        let sector = self.pos / BLOCK_SIZE;
        let max_size = self.raw.size() * BLOCK_SIZE;
        let mut write_count = 0;
        for i in sector..(sector + len) {
            if self.pos >= max_size {
                break;
            }
            let start = self.pos % BLOCK_SIZE;
            let copy_amount = usize::min(buf.len() - write_count, BLOCK_SIZE - start);
            let mut tmp_buf: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
            self.raw
                .block_op(tmp_buf.as_mut_ptr(), i as u64, BlockOpType::Read);
            unsafe {
                core::ptr::copy_nonoverlapping(
                    buf.as_ptr().add(write_count),
                    tmp_buf.as_mut_ptr().add(start),
                    copy_amount,
                );
            }
            self.raw
                .block_op(tmp_buf.as_mut_ptr(), i as u64, BlockOpType::Write);
            self.pos += copy_amount;
            write_count += copy_amount;
        }

        Ok(write_count)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'r> Seek for Disk<'r> {
    fn seek(&mut self, pos: fatfs::SeekFrom) -> Result<u64, Self::Error> {
        match pos {
            fatfs::SeekFrom::Current(i) => {
                self.pos = (self.pos as i64 + i) as usize;
            }
            fatfs::SeekFrom::End(i) => {
                self.pos = ((self.raw.size() * BLOCK_SIZE) as i64 + i) as usize
            }
            fatfs::SeekFrom::Start(n) => {
                self.pos = n as usize;
            }
        }

        Ok(self.pos as u64)
    }
}
