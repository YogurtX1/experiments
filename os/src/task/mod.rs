pub mod context;
mod switch;

use crate::config::{KERNEL_STACK_SIZE, PAGE_SIZE, PAGE_SIZE_BITS};
use crate::loader::get_num_app;
use crate::mm::{KERNEL_SPACE, init_kernel_memory};
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use alloc::vec::Vec;
use context::TaskContext;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

#[derive(Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}

pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub trap_cx_ppn: usize,  // TrapContext 物理页号
    pub kernel_stack: KernelStack,
    pub memory_set: Option<crate::mm::MemorySet>,
}

impl TaskControlBlock {
    /// 返回 TrapContext 的物理地址 (页对齐, offset=0)
    /// TrapContext 位于内核栈最高页的开始 (kstack_top - PAGE_SIZE)
    pub fn get_trap_cx_ptr(&self) -> usize {
        self.trap_cx_ppn << PAGE_SIZE_BITS
    }

    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        let ptr = self.get_trap_cx_ptr();
        unsafe { &mut *(ptr as *mut TrapContext) }
    }

    pub fn get_user_satp(&self) -> usize {
        match &self.memory_set {
            Some(ms) => ms.satp(),
            None => KERNEL_SPACE.exclusive_access().page_table.satp(),
        }
    }
}

#[allow(dead_code)]
pub struct TaskManager {
    num_app: usize,
    pub inner: TaskManagerInner,
}

pub struct TaskManagerInner {
    pub tasks: Vec<TaskControlBlock>,
    pub current_task: usize,
}

impl TaskManager {
    pub fn new() -> Self {
        let num_app = get_num_app();
        Self {
            num_app,
            inner: TaskManagerInner {
                tasks: Vec::with_capacity(num_app),
                current_task: 0,
            },
        }
    }

    fn run_first_task(&self) -> ! {
        let task0 = &self.inner.tasks[0];
        // sscratch 必须设为 TrapContext 在用户页表中的虚拟地址,
        // 因为 __alltraps 用 csrrw sp, sscratch, sp 时用户页表处于激活状态,
        // 内核物理地址在用户页表中可能映射到错误的物理帧
        let trap_cx_user = task0.get_trap_cx().trap_cx_user;

        unsafe {
            core::arch::asm!("csrw sscratch, {}", in(reg) trap_cx_user);
        }

        // sscratch 已设置，此时才能安全启用定时器中断
        crate::trap::enable_timer_interrupt();
        crate::timer::set_next_trigger();

        extern "C" {
            fn __switch(
                current_cx: *const TaskContext,
                next_cx: *const TaskContext,
            );
        }
        let next_cx = &task0.task_cx as *const TaskContext;
        unsafe {
            __switch(&TaskContext::zero_init() as *const TaskContext, next_cx);
        }
        panic!("unreachable in run_first_task!");
    }

    fn run_next_task(&mut self) {
        if let Some(next_id) = self.find_next_task() {
            let next_cx_ptr = &self.inner.tasks[next_id].task_cx as *const TaskContext;

            // sscratch 必须设为 TrapContext 在用户页表中的虚拟地址
            let trap_cx_user = self.inner.tasks[next_id].get_trap_cx().trap_cx_user;
            unsafe {
                core::arch::asm!("csrw sscratch, {}", in(reg) trap_cx_user);
            }

            extern "C" {
                fn __switch(
                    current_cx: *const TaskContext,
                    next_cx: *const TaskContext,
                );
            }
            let cur = self.inner.current_task;
            let current_cx_ptr = &self.inner.tasks[cur].task_cx as *const TaskContext;
            self.inner.current_task = next_id;
            unsafe {
                __switch(current_cx_ptr, next_cx_ptr);
            }
        } else {
            panic!("All applications completed!");
        }
    }

    fn find_next_task(&self) -> Option<usize> {
        let current = self.inner.current_task;
        for i in 1..=self.inner.tasks.len() {
            let id = (current + i) % self.inner.tasks.len();
            if self.inner.tasks[id].task_status == TaskStatus::Ready {
                return Some(id);
            }
        }
        None
    }
}

pub fn run_first_task() -> ! {
    TASK_MANAGER.exclusive_access().run_first_task()
}

pub fn init() {
    init_kernel_memory();
    println!("[kernel] Memory system initialized");

    // 激活内核页表
    let kernel_satp = KERNEL_SPACE.exclusive_access().page_table.satp();
    unsafe {
        riscv::register::satp::write(kernel_satp);
        core::arch::asm!("sfence.vma", options(nomem, nostack));
    }
    println!("[kernel] Kernel page table activated, satp={:#x}", kernel_satp);

    // 更新 stvec 为 trampoline 地址
    crate::trap::set_trampoline_stvec(crate::config::TRAMPOLINE);

    let num_app = get_num_app();
    println!("[kernel] num_app = {}", num_app);

    let mut tasks = Vec::with_capacity(num_app);

    for i in 0..num_app {
        let kstack = KernelStack::new(i);
        let kstack_top = kstack.top();

        // ★ TrapContext 放在 kstack_top - PAGE_SIZE 处(页对齐)，
        //    避免 trap_handler 堆栈帧溢出覆盖 TrapContext
        //    旧布局: TrapContext 在 kstack_top - 320 (紧贴栈顶)
        //    新布局: TrapContext 在 kstack_top - 4096 (页对齐，有 4KB 保护空间)
        let trap_cx_kernel = kstack_top - PAGE_SIZE;
        // TrapContext 的物理页号 (用于映射到用户地址空间)
        let trap_cx_ppn = trap_cx_kernel >> PAGE_SIZE_BITS;

        let trap_cx = unsafe { &mut *(trap_cx_kernel as *mut TrapContext) };
        let kernel_satp = KERNEL_SPACE.exclusive_access().page_table.satp();
        extern "C" {
            fn trap_handler();
        }
        *trap_cx = TrapContext::app_init_context(
            0,                   // entry (稍后设置)
            0,                   // user_sp (稍后设置)
            kernel_satp,
            kstack_top,
            trap_handler as *const () as usize,
            0,                   // user_satp (稍后设置)
            0,                   // trap_cx_user (稍后设置)
            crate::config::TRAP_CONTEXT_USER, // __alltraps csrrw 用 (offset=0, 页对齐)
        );

        tasks.push(TaskControlBlock {
            task_status: if i == 0 { TaskStatus::Running } else { TaskStatus::Ready },
            task_cx: TaskContext::goto_trap_return(kstack_top),
            trap_cx_ppn,
            kernel_stack: kstack,
            memory_set: None,
        });
    }

    TASK_MANAGER.exclusive_access().inner.tasks = tasks;
    TASK_MANAGER.exclusive_access().inner.current_task = 0;
    println!("[kernel] {} tasks initialized", num_app);
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    TASK_MANAGER.exclusive_access().run_next_task();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    TASK_MANAGER.exclusive_access().run_next_task();
}

fn mark_current_exited() {
    let mut tm = TASK_MANAGER.exclusive_access();
    let current = tm.inner.current_task;
    tm.inner.tasks[current].task_status = TaskStatus::Exited;
}

fn mark_current_suspended() {
    let mut tm = TASK_MANAGER.exclusive_access();
    let current = tm.inner.current_task;
    tm.inner.tasks[current].task_status = TaskStatus::Ready;
}

pub struct KernelStack {
    bottom_ppn: usize,
}

impl KernelStack {
    /// 分配 KERNEL_STACK_SIZE / PAGE_SIZE 个连续物理页作为内核栈
    pub fn new(_pid: usize) -> Self {
        let bottom = crate::mm::frame_alloc()
            .expect("KernelStack: alloc bottom frame failed");
        let num_pages = KERNEL_STACK_SIZE / PAGE_SIZE;
        // 帧分配器是顺序分配的，连续 alloc 获得连续页
        for _ in 1..num_pages {
            crate::mm::frame_alloc()
                .expect("KernelStack: alloc subsequent frame failed");
        }
        Self { bottom_ppn: bottom.0 }
    }

    pub fn top(&self) -> usize {
        (self.bottom_ppn << PAGE_SIZE_BITS) + KERNEL_STACK_SIZE
    }
}
