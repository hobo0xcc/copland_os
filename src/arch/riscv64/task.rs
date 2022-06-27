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
        let context_from = &task_from.kcontext as *const KernelContext as usize;
        let context_to = &task_to.kcontext as *const KernelContext as usize;
        switch(context_from, context_to);
    }

    unsafe fn user_switch(&mut self, current: TaskId) -> ! {
        assert!(self.tasks.contains_key(&current));
        loop {}
        // let task = self.tasks.get(&current).unwrap();
    }

    fn create_arch_task(&mut self, id: TaskId) {
        self.tasks.insert(id, Task::new(id));
    }

    fn init_stack(&mut self, id: TaskId, stack_pointer: usize) {
        if !self.tasks.contains_key(&id) {
            panic!("Unknown Task ID: {}", id);
        }
        self.tasks.get_mut(&id).unwrap().kcontext.sp = stack_pointer;
    }

    fn init_start(&mut self, id: TaskId, start_address: usize) {
        if !self.tasks.contains_key(&id) {
            panic!("Unknown Task ID: {}", id);
        }
        self.tasks.get_mut(&id).unwrap().kcontext.ra = start_address;
    }
}

// For context switch in kernel
// We only need to save and restore the callee-saved registers because
// the caller-saved registers (i.e. non callee-saved registers) may be overwritten after
// the call of context switch.
#[derive(Copy, Clone, Debug, Default)]
#[allow(dead_code)]
#[repr(packed)]
pub struct KernelContext {
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

// For switching between kernel and user
#[derive(Copy, Clone, Debug, Default)]
#[allow(dead_code)]
#[repr(packed)]
pub struct UserContext {
    kernel_satp: usize,   // 0
    kernel_sp: usize,     // 8
    kernel_trap: usize,   // 16
    epc: usize,           // 24
    kernel_hartid: usize, // 32
    ra: usize,            // 40
    sp: usize,            // 48
    gp: usize,            // 56
    tp: usize,            // 64
    t0: usize,            // 72
    t1: usize,            // 80
    t2: usize,            // 88
    s0: usize,            // 96
    s1: usize,            // 104
    a0: usize,            // 112
    a1: usize,            // 120
    a2: usize,            // 128
    a3: usize,            // 136
    a4: usize,            // 144
    a5: usize,            // 152
    a6: usize,            // 160
    a7: usize,            // 168
    s2: usize,            // 176
    s3: usize,            // 184
    s4: usize,            // 192
    s5: usize,            // 200
    s6: usize,            // 208
    s7: usize,            // 216
    s8: usize,            // 224
    s9: usize,            // 232
    s11: usize,           // 240
    s12: usize,           // 248
    t3: usize,            // 256
    t4: usize,            // 264
    t5: usize,            // 272
    t6: usize,            // 280
}

impl KernelContext {
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
    pub kcontext: KernelContext,
    pub ucontext: UserContext,
}

impl Task {
    pub fn new(id: TaskId) -> Self {
        Self {
            id,
            kcontext: KernelContext::default(),
            ucontext: UserContext::default(),
        }
    }
}
