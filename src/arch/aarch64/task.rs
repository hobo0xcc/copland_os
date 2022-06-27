use crate::lazy::Lazy;
use crate::task::{ArchTaskManager, TaskId};
use alloc::alloc::{alloc_zeroed, Layout};
use alloc::string::*;
use core::arch::global_asm;
use hashbrown::HashMap;

pub const KERNEL_STACK_SIZE: usize = 0x8000;

pub static mut ARCH_TASK_MANAGER: Lazy<TaskManager> = Lazy::new(|| TaskManager::new());

global_asm!(include_str!("switch.S"));

extern "C" {
    pub fn switch(from: usize, to: usize);
}

pub struct TaskManager {
    tasks: HashMap<TaskId, Task>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }
}

impl ArchTaskManager for TaskManager {
    unsafe fn context_switch(&mut self, from: TaskId, to: TaskId) {
        assert!(self.tasks.contains_key(&from));
        assert!(self.tasks.contains_key(&to));
        let task_from = self.tasks.get(&from).unwrap();
        let task_to = self.tasks.get(&to).unwrap();
        let context_from = &task_from.context as *const Context as usize;
        let context_to = &task_to.context as *const Context as usize;
        switch(context_from, context_to);
    }

    unsafe fn user_switch(&mut self, current: TaskId) -> ! {
        assert!(self.tasks.contains_key(&current));
        unimplemented!();
        loop {}
        // let task = self.tasks.get(&current).unwrap();
    }

    fn create_arch_task(&mut self, id: TaskId, name: String) {
        let kernel_stack = unsafe {
            let layout = Layout::from_size_align(KERNEL_STACK_SIZE, 0x1000).unwrap();
            alloc_zeroed(layout)
        };
        self.tasks.insert(id, Task::new(id, name, kernel_stack));
    }

    fn init_start(&mut self, id: TaskId, start_address: usize) {
        if !self.tasks.contains_key(&id) {
            panic!("Unknown Task ID: {}", id);
        }
        self.tasks.get_mut(&id).unwrap().context.x30 = start_address;
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[allow(dead_code)]
#[repr(packed)]
pub struct Context {
    sp: usize,
    x18: usize,
    x19: usize,
    x20: usize,
    x21: usize,
    x22: usize,
    x23: usize,
    x24: usize,
    x25: usize,
    x26: usize,
    x27: usize,
    x28: usize,
    x29: usize,
    x30: usize,
}

#[allow(dead_code)]
pub struct Task {
    id: TaskId,
    name: String,
    kernel_stack: *mut u8,
    pub context: Context,
}

impl Task {
    pub fn new(id: TaskId, name: String, kernel_stack: *mut u8) -> Self {
        Self {
            id,
            name,
            kernel_stack,
            context: Context {
                sp: kernel_stack as usize + KERNEL_STACK_SIZE,
                ..Default::default()
            },
        }
    }
}
