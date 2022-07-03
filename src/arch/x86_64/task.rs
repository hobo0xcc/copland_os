#![allow(unused_variables)]

use crate::error::*;
use crate::lazy::Lazy;
use crate::task::{ArchTaskManager, TaskId};
use alloc::string::String;

pub static mut ARCH_TASK_MANAGER: Lazy<TaskManager> =
    Lazy::<TaskManager, fn() -> TaskManager>::new(|| TaskManager::new());

pub struct TaskManager;

impl TaskManager {
    pub fn new() -> Self {
        Self
    }
}

impl ArchTaskManager for TaskManager {
    unsafe fn context_switch(&mut self, from: TaskId, to: TaskId) {}

    unsafe fn user_switch(&mut self, current: TaskId) -> ! {
        loop {}
    }

    fn map(
        &mut self,
        id: TaskId,
        paddr: usize,
        vaddr: usize,
        r: bool,
        w: bool,
        x: bool,
    ) -> Result<(), TaskError> {
        Ok(())
    }

    fn create_arch_task(&mut self, id: TaskId, name: String) {}

    fn init_start(&mut self, id: TaskId, start_address: usize) -> Result<(), TaskError> {
        Ok(())
    }

    fn init_user_entry(&mut self, id: TaskId, entry: usize) -> Result<(), TaskError> {
        Ok(())
    }
}
