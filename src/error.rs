use crate::*;
use alloc::string::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VMError {
    Misaligned,
    NotFound,
}

#[derive(Debug)]
pub enum TaskError {
    FileNotFound(String),
    DiskError(fatfs::Error<DiskError>),
    ExecParseError(goblin::error::Error),
    TaskNotFound(task::TaskId),
    MapError(VMError),
}

#[derive(Debug)]
pub enum DiskError {
    Dummy,
}
