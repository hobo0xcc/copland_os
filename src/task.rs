use alloc::collections::VecDeque;
use alloc::string::{String, ToString};
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
    name: String,
}

impl Task {
    pub fn new(name: &str, id: TaskId) -> Self {
        Self {
            id,
            name: name.to_string(),
        }
    }
}

pub struct TaskManager {
    tasks: HashMap<TaskId, Task>,
    ready_queue: VecDeque<TaskId>,
    task_id: TaskId,
    running: TaskId,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            ready_queue: VecDeque::new(),
            task_id: 0,
            running: 0,
        }
    }

    pub fn next_task_id(&mut self) -> TaskId {
        let result = self.task_id;
        self.task_id += 1;
        result
    }

    pub fn init(&mut self) {
        info!("Initialize Task Manager");
        self.task_id = 0;
        self.running = self.next_task_id();
        assert_eq!(self.running, 0);
        self.tasks
            .insert(self.running, Task::new("kernel", self.running));
    }

    pub unsafe fn schedule(&mut self) {
        if self.ready_queue.len() == 0 {
            return;
        }
        assert!(1 <= self.ready_queue.len());
        let next_running = self.ready_queue.pop_front().unwrap();
        let current_running = self.running;

        self.ready_queue.push_back(current_running);
        self.running = next_running;
        // Do context switch
        unimplemented!();
    }

    pub fn create_task(&mut self, name: &str) -> TaskId {
        let task_id = self.next_task_id();
        let task = Task::new(name, task_id);
        self.tasks.insert(task_id, task);
        assert!(self.tasks.contains_key(&task_id));

        task_id
    }
}
