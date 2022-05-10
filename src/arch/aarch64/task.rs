use crate::task::{ArchTaskManager, TaskId};
use hashbrown::HashMap;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref ARCH_TASK_MANAGER: Mutex<TaskManager> = Mutex::new(TaskManager::new());
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
        ARCH_TASK_MANAGER.force_unlock();
        unimplemented!()
    }

    fn create_arch_task(&mut self, id: TaskId) {
        self.tasks.insert(id, Task::new(id));
    }

    fn init_stack(&mut self, id: TaskId, stack_pointer: usize) {
        unimplemented!()
    }

    fn init_start(&mut self, id: TaskId, start_address: usize) {
        unimplemented!()
    }
}

pub struct Task {
    id: TaskId,
}

impl Task {
    pub fn new(id: TaskId) -> Self {
        Self { id }
    }
}
