use alloc::collections::VecDeque;
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
    ready_queue: VecDeque<TaskId>,
    running: TaskId,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            ready_queue: VecDeque::new(),
            running: 0,
        }
    }

    pub fn init(&mut self) {
        info!("Initialize Task Manager");
        self.running = 0;
        self.tasks.insert(self.running, Task::new(self.running));
    }

    pub fn schedule(&mut self) {
        if self.ready_queue.len() == 0 {
            return;
        }
        assert!(1 <= self.ready_queue.len());
        let next_task_id = self.ready_queue.pop_front().unwrap();
        let current_task_id = self.running;
        // Do context switch
        unimplemented!();
    }
}
