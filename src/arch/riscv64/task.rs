use crate::arch::riscv64::csr::*;
use crate::arch::riscv64::trap;
use crate::arch::riscv64::vm;
use crate::arch::riscv64::vm::PageTable;
use crate::arch::PAGE_SIZE;
use crate::error::TaskError;
use crate::lazy::Lazy;
use crate::task::{ArchTaskManager, TaskId};
use alloc::alloc::{alloc_zeroed, Layout};
use alloc::format;
use alloc::string::*;
use core::arch::{asm, global_asm};
use core::mem::size_of;
use hashbrown::HashMap;

pub const USER_CONTEXT: usize = 0x3f_ffff_e000;
// virtual address of trampoline
pub const TRAMPOLINE: usize = 0x3f_ffff_f000;
// max kernel stack size
pub const KERNEL_STACK_SIZE: usize = 0x8000;

pub static mut ARCH_TASK_MANAGER: Lazy<TaskManager> =
    Lazy::<TaskManager, fn() -> TaskManager>::new(|| TaskManager::new());

global_asm!(include_str!("switch.S"));

extern "C" {
    pub fn switch(from: usize, to: usize);
}

global_asm!(include_str!("trampoline.S"));

extern "C" {
    pub fn trampoline();
    pub fn uservec();
    pub fn userret(context: usize, satp: usize);
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
        // write virtual address of uservec to stvec
        Csr::Stvec.write(TRAMPOLINE + ((uservec as usize) - (trampoline as usize)));
        let task = self.tasks.get_mut(&current).unwrap();
        let user_satp = vm::VM_MANAGER.make_satp(task.page_table_name.as_str());
        let mut tp: usize;
        asm!("mv {}, tp", out(reg)tp);

        (*task.ucontext).kernel_satp = Csr::Satp.read();
        (*task.ucontext).kernel_sp = (task.kernel_stack as usize) + KERNEL_STACK_SIZE;
        (*task.ucontext).kernel_hartid = tp;
        (*task.ucontext).kernel_trap = trap::user_trap as usize;

        let mut sstatus = Csr::Sstatus.read();
        sstatus &= !Sstatus::SPP.mask();
        sstatus |= Sstatus::SPIE.mask();
        Csr::Sstatus.write(sstatus);

        Csr::Sepc.write((*task.ucontext).epc);

        let fn_ret = TRAMPOLINE + ((userret as usize) - (trampoline as usize));
        (core::mem::transmute::<*mut u8, fn(usize, usize) -> !>(fn_ret as *mut u8))(
            USER_CONTEXT,
            user_satp,
        );
    }

    fn map(
        &mut self,
        id: TaskId,
        paddr: usize,
        vaddr: usize,
        r: bool,
        w: bool,
        x: bool,
    ) -> Result<(), TaskError> {
        let task = self.tasks.get_mut(&id).ok_or(TaskError::TaskNotFound(id))?;
        task.map(paddr, vaddr, r, w, x)?;
        Ok(())
    }

    fn create_arch_task(&mut self, id: TaskId, name: String) {
        let page_table_name = format!("{}.{}", name, id);
        let page_table = unsafe {
            let ptr = vm::VM_MANAGER.create_table();
            vm::VM_MANAGER.set_table(page_table_name.clone(), ptr);
            ptr
        };
        self.tasks
            .insert(id, Task::new(id, name, page_table_name, page_table));
    }

    fn init_start(&mut self, id: TaskId, start_address: usize) -> Result<(), TaskError> {
        self.tasks
            .get_mut(&id)
            .ok_or(TaskError::TaskNotFound(id))?
            .kcontext
            .ra = start_address;
        Ok(())
    }

    fn init_user_entry(&mut self, id: TaskId, entry: usize) -> Result<(), TaskError> {
        let ucontext = self
            .tasks
            .get_mut(&id)
            .ok_or(TaskError::TaskNotFound(id))?
            .ucontext;
        unsafe {
            (*ucontext).epc = entry;
        }
        Ok(())
    }
}

// For the context switch in the kernel
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
    name: String,
    page_table_name: String,
    page_table: *mut PageTable,
    kernel_stack: *mut u8,
    pub kcontext: KernelContext,
    pub ucontext: *mut UserContext,
}

impl Task {
    pub fn new(
        id: TaskId,
        name: String,
        page_table_name: String,
        page_table: *mut PageTable,
    ) -> Self {
        assert!(size_of::<UserContext>() <= PAGE_SIZE);
        let ucontext = unsafe {
            let layout = Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap();
            alloc_zeroed(layout)
        } as *mut UserContext;
        unsafe {
            vm::VM_MANAGER
                .map(
                    &page_table_name,
                    trampoline as usize,
                    TRAMPOLINE,
                    true,
                    true,
                    true,
                    false,
                )
                .unwrap();
            vm::VM_MANAGER
                .map(
                    &page_table_name,
                    ucontext as usize,
                    USER_CONTEXT,
                    true,
                    true,
                    false,
                    false,
                )
                .unwrap();
        }
        let kernel_stack = unsafe {
            let layout = Layout::from_size_align(KERNEL_STACK_SIZE, 0x1000).unwrap();
            alloc_zeroed(layout)
        };
        Self {
            id,
            name,
            page_table_name,
            page_table,
            kernel_stack,
            kcontext: KernelContext {
                sp: kernel_stack as usize + KERNEL_STACK_SIZE,
                ..Default::default()
            },
            ucontext,
        }
    }

    pub fn map(
        &mut self,
        paddr: usize,
        vaddr: usize,
        r: bool,
        w: bool,
        x: bool,
    ) -> Result<(), TaskError> {
        unsafe { vm::VM_MANAGER.map(self.page_table_name.as_str(), paddr, vaddr, r, w, x, true) }
            .map_err(|e| TaskError::MapError(e))?;
        Ok(())
    }
}
