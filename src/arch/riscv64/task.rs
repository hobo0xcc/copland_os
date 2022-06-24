use crate::lazy::Lazy;
use crate::task::{ArchTaskManager, TaskId};
use core::arch::global_asm;
use hashbrown::HashMap;

pub static mut ARCH_TASK_MANAGER: Lazy<TaskManager> =
    Lazy::<TaskManager, fn() -> TaskManager>::new(|| TaskManager::new());

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
        self.tasks.get_mut(&id).unwrap().context.ra = start_address;
    }
}

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
#[repr(packed)]
pub struct Context {
    ra: usize,
    sp: usize,
    s0: usize,
    s1: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
}

impl Context {
    pub fn new() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
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
