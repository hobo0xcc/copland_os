use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref TASK_MANAGER: Mutex<TaskManager> = Mutex::new(TaskManager::new());
}

pub struct TaskManager {}

impl TaskManager {
    pub fn new() -> Self {
        Self {}
    }
}
