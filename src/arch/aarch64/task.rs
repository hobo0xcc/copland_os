use crate::lazy::Lazy;
use crate::task::{ArchTaskManager, TaskId};
use core::arch::global_asm;
use hashbrown::HashMap;

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

    fn create_arch_task(&mut self, id: TaskId) {
        self.tasks.insert(id, Task::new(id));
    }

    fn init_stack(&mut self, id: TaskId, stack_pointer: usize) {
        if !self.tasks.contains_key(&id) {
            panic!("Unknown Task ID: {}", id);
        }
        self.tasks.get_mut(&id).unwrap().context.sp = stack_pointer;
    }

    fn init_start(&mut self, id: TaskId, start_address: usize) {
        if !self.tasks.contains_key(&id) {
            panic!("Unknown Task ID: {}", id);
        }
        self.tasks.get_mut(&id).unwrap().context.x30 = start_address;
    }
}

#[derive(Copy, Clone, Debug)]
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

impl Context {
    pub fn new() -> Self {
        Self {
            sp: 0,
            x18: 0,
            x19: 0,
            x20: 0,
            x21: 0,
            x22: 0,
            x23: 0,
            x24: 0,
            x25: 0,
            x26: 0,
            x27: 0,
            x28: 0,
            x29: 0,
            x30: 0,
        }
    }
}

#[allow(dead_code)]
pub struct Task {
    id: TaskId,
    pub context: Context,
}

impl Task {
    pub fn new(id: TaskId) -> Self {
        Self {
            id,
            context: Context::new(),
        }
    }
}
