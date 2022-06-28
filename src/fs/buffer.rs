use crate::fs::Size;
use crate::*;
use alloc::vec;
use alloc::vec::*;
use fatfs::{IoBase, Read, Seek, SeekFrom, Write};
use hashbrown::HashMap;

pub const CACHE_SIZE: usize = 60;
pub const SECTOR_SIZE: usize = crate::fs::fat32::BLOCK_SIZE;

pub struct Buffer<R> {
    inner: R,
    pos: usize,
    cache: Cache,
}

pub struct Cache {
    cache: Vec<[u8; SECTOR_SIZE]>,
    cache_from: [Option<usize>; CACHE_SIZE],
    cur: usize,
    map: HashMap<usize, usize>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            cache: vec![[0; SECTOR_SIZE]; CACHE_SIZE],
            cache_from: [None; CACHE_SIZE],
            cur: 0,
            map: HashMap::new(),
        }
    }

    pub fn next_sector_cache<R: Write + Seek + IoBase>(
        &mut self,
        inner: &mut R,
    ) -> Result<usize, R::Error> {
        let res = self.cur;
        self.cur = (self.cur + 1) % CACHE_SIZE;
        if let Some(sector) = self.cache_from[res] {
            inner.seek(SeekFrom::Start((sector * SECTOR_SIZE) as u64))?;
            inner.write(&self.cache[res])?;
            self.map.remove(&sector);
        }
        self.cache_from[res] = None;
        Ok(res)
    }

    pub fn set_cache<R: Write + Seek + IoBase>(
        &mut self,
        inner: &mut R,
        sector: usize,
        buf: &[u8],
    ) -> Result<(), R::Error> {
        assert!(buf.len() == SECTOR_SIZE);
        let cache_index = self.next_sector_cache(inner)?;
        self.map.insert(sector, cache_index);
        self.cache_from[cache_index] = Some(sector);
        self.cache[cache_index].copy_from_slice(&buf[0..SECTOR_SIZE]);
        Ok(())
    }

    pub fn copy_into(&self, sector: usize, begin: usize, buf: &mut [u8]) -> Result<(), ()> {
        let cache_index = *self.map.get(&sector).ok_or(())?;
        let end = begin + buf.len();
        assert!(end <= SECTOR_SIZE);
        buf.copy_from_slice(&self.cache[cache_index][begin..end]);
        Ok(())
    }

    pub fn copy_from(&mut self, sector: usize, begin: usize, buf: &[u8]) -> Result<(), ()> {
        let cache_index = *self.map.get(&sector).ok_or(())?;
        let end = begin + buf.len();
        assert!(end <= SECTOR_SIZE);
        self.cache[cache_index][begin..end].copy_from_slice(&buf);
        Ok(())
    }

    pub fn flush<R: Write + Seek + IoBase>(&mut self, inner: &mut R) -> Result<(), R::Error> {
        for (i, sector) in self.cache_from.iter().enumerate() {
            if let Some(sector) = sector.as_ref() {
                inner.seek(SeekFrom::Start((sector * SECTOR_SIZE) as u64))?;
                inner.write(&self.cache[i])?;
                // self.map.remove(&sector);
            }
        }

        Ok(())
    }
}

impl<R> Buffer<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            pos: 0,
            cache: Cache::new(),
        }
    }
}

impl<R: IoBase> IoBase for Buffer<R> {
    type Error = R::Error;
}

impl<R: Read + Write + Seek + IoBase> Read for Buffer<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut read_bytes = 0;
        while read_bytes < buf.len() {
            let read_amount = usize::min(SECTOR_SIZE, buf.len() - read_bytes);
            let sector = self.pos / SECTOR_SIZE;
            let begin = self.pos % SECTOR_SIZE;
            let read_range = read_bytes..(read_bytes + read_amount);
            match self
                .cache
                .copy_into(sector, begin, &mut buf[read_range.clone()])
            {
                Ok(_) => {}
                Err(_) => {
                    let mut tmp_buf = [0; SECTOR_SIZE];
                    let sector_bytes = sector * SECTOR_SIZE;
                    self.inner.seek(SeekFrom::Start(sector_bytes as u64))?;
                    self.inner.read(&mut tmp_buf)?;
                    self.cache.set_cache(&mut self.inner, sector, &tmp_buf)?;
                    self.cache
                        .copy_into(sector, begin, &mut buf[read_range.clone()])
                        .unwrap();
                    // buf[read_range].copy_from_slice(&tmp_buf[0..read_amount]);
                }
            }
            self.pos += read_amount;
            read_bytes += read_amount;
        }

        Ok(buf.len())
    }
}

impl<R: Read + Write + Seek + IoBase> Write for Buffer<R> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let mut write_bytes = 0;
        while write_bytes < buf.len() {
            let write_amount = usize::min(SECTOR_SIZE, buf.len() - write_bytes);
            let sector = self.pos / SECTOR_SIZE;
            let begin = self.pos % SECTOR_SIZE;
            let write_range = write_bytes..(write_bytes + write_amount);
            match self
                .cache
                .copy_from(sector, begin, &buf[write_range.clone()])
            {
                Ok(_) => {}
                Err(_) => {
                    let mut tmp_buf = [0; SECTOR_SIZE];
                    let sector_bytes = sector * SECTOR_SIZE;
                    self.inner.seek(SeekFrom::Start(sector_bytes as u64))?;
                    self.inner.read(&mut tmp_buf)?;
                    self.cache.set_cache(&mut self.inner, sector, &tmp_buf)?;
                    self.cache
                        .copy_from(sector, begin, &buf[write_range.clone()])
                        .unwrap();
                }
            }
            self.pos += write_amount;
            write_bytes += write_amount;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.cache.flush(&mut self.inner)?;
        Ok(())
    }
}

impl<R: Seek + IoBase + Size> Seek for Buffer<R> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        match pos {
            SeekFrom::Current(i) => {
                self.pos = (self.pos as i64 + i) as usize;
            }
            SeekFrom::End(i) => self.pos = (self.inner.size() as i64 + i) as usize,
            SeekFrom::Start(n) => {
                self.pos = n as usize;
            }
        }
        Ok(self.pos as u64)
    }
}
