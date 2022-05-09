use hashbrown::HashMap;
use lazy_static::lazy_static;
use log::info;
use spin::Mutex;

lazy_static! {
    pub static ref TASK_MANAGER: Mutex<TaskManager> = Mutex::new(TaskManager::new());
}

pub type TaskId = usize;

pub struct Task {
    id: TaskId,
}

impl Task {
    pub fn new(id: TaskId) -> Self {
        Self { id }
    }
}

pub struct TaskManager {
    tasks: HashMap<TaskId, Task>,
    running: TaskId,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            running: 0,
        }
    }

    pub fn init(&mut self) {
        info!("Initialize Task Manager");
        self.running = 0;
        self.tasks.insert(self.running, Task::new(self.running));
    }
}
