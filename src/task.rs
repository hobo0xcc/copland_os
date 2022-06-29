use crate::arch::PAGE_SIZE;
use crate::error::TaskError;
use crate::fs::fat32;
use crate::lazy::Lazy;
use crate::*;
use alloc::alloc::{alloc_zeroed, Layout};
use alloc::collections::VecDeque;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::*;
use fatfs::{Read, Seek, SeekFrom};
use goblin::elf;
use hashbrown::HashMap;
use log::info;

#[cfg(target_arch = "riscv64")]
use crate::arch::riscv64;

#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64;

macro_rules! arch_task_manager {
    () => {
        loop {
            #[cfg(target_arch = "riscv64")]
            break &mut riscv64::task::ARCH_TASK_MANAGER;

            #[cfg(target_arch = "aarch64")]
            break &mut aarch64::task::ARCH_TASK_MANAGER;
        }
    };
}

pub static mut TASK_MANAGER: Lazy<TaskManager> =
    Lazy::<TaskManager, fn() -> TaskManager>::new(|| TaskManager::new());

pub trait ArchTaskManager {
    unsafe fn context_switch(&mut self, from: TaskId, to: TaskId);
    unsafe fn user_switch(&mut self, current: TaskId) -> !;
    fn map(
        &mut self,
        id: TaskId,
        paddr: usize,
        vaddr: usize,
        r: bool,
        w: bool,
        x: bool,
    ) -> Result<(), TaskError>;
    fn create_arch_task(&mut self, id: TaskId, name: String);
    fn init_start(&mut self, id: TaskId, start_address: usize) -> Result<(), TaskError>;
    fn init_user_entry(&mut self, id: TaskId, entry: usize) -> Result<(), TaskError>;
}

pub type TaskId = usize;

pub struct MemoryRegion {
    paddr: usize,
    vaddr: Option<usize>,
    size: usize,
    r: bool,
    w: bool,
    x: bool,
}

pub enum TaskState {
    Running,
    Ready,
    Stop,
}

#[allow(dead_code)]
pub struct Task {
    id: TaskId,
    name: String,
    state: TaskState,
    memory: Vec<MemoryRegion>,
}

impl Task {
    pub fn new(name: &str, id: TaskId) -> Self {
        Self {
            id,
            name: name.to_string(),
            state: TaskState::Stop,
            memory: Vec::new(),
        }
    }

    pub fn update_state(&mut self, state: TaskState) {
        self.state = state;
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

    pub fn current(&self) -> TaskId {
        self.running
    }

    pub fn next_task_id(&mut self) -> TaskId {
        let result = self.task_id;
        self.task_id += 1;
        result
    }

    pub fn init(&mut self) -> Result<(), TaskError> {
        info!("Initialize Task Manager");
        self.task_id = 0;
        self.running = self.create_task("kernel", 0)?;
        self.tasks
            .get_mut(&self.running)
            .ok_or(TaskError::TaskNotFound(self.running))?
            .update_state(TaskState::Running);
        Ok(())
    }

    // Round robin scheduling
    pub unsafe fn schedule(&mut self) {
        if self.ready_queue.len() == 0 {
            return;
        }
        assert!(1 <= self.ready_queue.len());
        let next_running = self.ready_queue.pop_front().unwrap();
        let current_running = self.running;
        assert!(self.tasks.contains_key(&next_running));
        assert!(self.tasks.contains_key(&current_running));

        self.tasks
            .get_mut(&next_running)
            .unwrap()
            .update_state(TaskState::Running);
        self.tasks
            .get_mut(&current_running)
            .unwrap()
            .update_state(TaskState::Ready);

        self.ready_queue.push_back(current_running);
        self.running = next_running;
        // Do context switch
        #[cfg(target_arch = "riscv64")]
        riscv64::task::ARCH_TASK_MANAGER.context_switch(current_running, next_running);

        #[cfg(target_arch = "aarch64")]
        aarch64::task::ARCH_TASK_MANAGER.context_switch(current_running, next_running);
    }

    pub fn ready_task(&mut self, id: TaskId) {
        if !self.tasks.contains_key(&id) {
            panic!("Unknown Task ID: {}", id);
        }
        assert!(self.tasks.contains_key(&id));
        self.tasks
            .get_mut(&id)
            .unwrap()
            .update_state(TaskState::Ready);
        self.ready_queue.push_back(id);
    }

    pub fn create_task(&mut self, name: &str, func: usize) -> Result<TaskId, TaskError> {
        let task_id = self.next_task_id();
        let task = Task::new(name, task_id);
        self.tasks.insert(task_id, task);
        assert!(self.tasks.contains_key(&task_id));

        unsafe {
            let arch_tm = arch_task_manager!();

            arch_tm.create_arch_task(task_id, name.to_string());
            arch_tm.init_start(task_id, func)?;
        }

        Ok(task_id)
    }

    pub fn exec(&mut self, id: TaskId, path: &str) -> Result<(), TaskError> {
        let task = self.tasks.get_mut(&id).ok_or(TaskError::TaskNotFound(id))?;
        let root_dir = unsafe { fat32::FILE_SYSTEM.root_dir() };
        let mut file = root_dir
            .open_file(path)
            .map_err(|e| TaskError::DiskError(e))?;
        let file_size = file
            .seek(SeekFrom::End(0))
            .map_err(|e| TaskError::DiskError(e))? as usize;
        file.seek(SeekFrom::Start(0))
            .map_err(|e| TaskError::DiskError(e))?;
        let mut buf: Vec<u8> = vec![0; file_size];
        file.read(&mut buf).map_err(|e| TaskError::DiskError(e))?;
        let elf_exe = elf::Elf::parse(&buf).map_err(|e| TaskError::ExecParseError(e))?;
        for ph in elf_exe.program_headers.iter() {
            let page_offset = ph.vm_range().start % PAGE_SIZE;
            let mut size = page_offset + ph.p_memsz as usize;
            size = size + (PAGE_SIZE - size % PAGE_SIZE); // Round up
            assert!(size % PAGE_SIZE == 0);
            let program: *mut u8 = unsafe {
                let layout = Layout::from_size_align(size, PAGE_SIZE).unwrap();
                alloc_zeroed(layout)
            };
            unsafe {
                core::ptr::copy_nonoverlapping(
                    (&buf).as_slice().as_ptr().add(ph.p_offset as usize),
                    program.add(page_offset),
                    ph.p_memsz as usize,
                );
            }
            let mem_region = MemoryRegion {
                paddr: program as usize,
                vaddr: Some(ph.vm_range().start - page_offset),
                size,
                r: ph.is_read(),
                w: ph.is_write(),
                x: ph.is_executable(),
            };
            task.memory.push(mem_region);
        }
        let arch_tm = unsafe { arch_task_manager!() };
        for region in task.memory.iter() {
            if let Some(vaddr) = region.vaddr {
                for i in (0..region.size).step_by(PAGE_SIZE) {
                    arch_tm.map(
                        id,
                        region.paddr + i,
                        vaddr + i,
                        region.r,
                        region.w,
                        region.x,
                    )?;
                }
            }
        }
        arch_tm.init_user_entry(id, elf_exe.entry as usize)?;
        Ok(())
    }
}

pub unsafe fn user_entry() -> ! {
    let task = TASK_MANAGER.tasks.get(&TASK_MANAGER.running).unwrap();
    let arch_tm = arch_task_manager!();
    arch_tm.user_switch(task.id);
}
