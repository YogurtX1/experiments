use super::{TaskContext, TaskControlBlock};
use alloc::sync::Arc;
use lazy_static::*;
use super::{fetch_task, TaskStatus};
use super::__switch;
use crate::trap::TrapContext;
use crate::sync::UPSafeCell;

pub struct Processor {
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx: TaskContext,
}

impl Processor {
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(|task| Arc::clone(task))
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe {
        UPSafeCell::new(Processor::new())
    };
}

pub fn run_tasks() {
    crate::println!("[run_tasks] entering scheduler loop");
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        let task_opt = fetch_task();
        if let Some(task) = task_opt {
            let pid = task.getpid();
            crate::println!("[run_tasks] switching to task pid={}", pid);
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            let trap_cx_ppn = task_inner.trap_cx_ppn;
            drop(task_inner);
            processor.current = Some(task);
            drop(processor);
            // ★ 预先设置 sscratch 为 TrapContext 物理地址
            let trap_cx_pa = trap_cx_ppn.0 * crate::config::PAGE_SIZE;
            crate::println!(
                "[run_tasks] trap_cx_ppn={:?}, trap_cx_pa={:#x}, task_cx={:?}",
                trap_cx_ppn, trap_cx_pa, unsafe { &*next_task_cx_ptr }
            );
            // ★ 诊断: 在 __switch 之前 dump TrapContext 关键字段
            {
                let trap_cx_bytes = trap_cx_ppn.get_bytes_array();
                // sepc 在 offset 33 (264 bytes), user_satp 在 offset 37 (296 bytes)
                let read_field = |offset: usize| -> usize {
                    let bytes: [u8; 8] = trap_cx_bytes[offset..offset+8].try_into().unwrap();
                    usize::from_le_bytes(bytes)
                };
                let tcx_sepc = read_field(33 * 8);
                let tcx_user_satp = read_field(37 * 8);
                let tcx_sstatus = read_field(32 * 8);
                let tcx_kernel_satp = read_field(34 * 8);
                crate::println!(
                    "[run_tasks] TrapContext: sepc={:#x}, user_satp={:#x}, sstatus={:#x}, kernel_satp={:#x}",
                    tcx_sepc, tcx_user_satp, tcx_sstatus, tcx_kernel_satp
                );
                // ★ 验证用户页表能否翻译 entry_point
                {
                    use crate::mm::page_table::PageTable;
                    use crate::mm::VirtAddr;
                    let pt = PageTable::from_token(tcx_user_satp);
                    let entry_pa = pt.translate_va(VirtAddr::from(0x80400000usize));
                    let tramp_pa = pt.translate_va(VirtAddr::from(crate::config::TRAMPOLINE));
                    let tcx_pa_user = pt.translate_va(VirtAddr::from(crate::config::TRAP_CONTEXT_USER));
                    crate::println!(
                        "[run_tasks] User PT: entry={:?}, trampoline={:?}, trap_cx={:?}",
                        entry_pa, tramp_pa, tcx_pa_user
                    );
                }
            }
            unsafe {
                core::arch::asm!("csrw sscratch, {}", in(reg) trap_cx_pa);
            }
            crate::println!("[run_tasks] calling __switch...");
            // ★ 定时器在 trap_handler 首次返回用户态时启用
            //    避免提前启用在 __restore→sret 窗口期引入竞态
            unsafe {
                __switch(
                    idle_task_cx_ptr,
                    next_task_cx_ptr,
                );
            }
            crate::println!("[run_tasks] returned from __switch (task pid={} scheduled out)", pid);
            // __switch 返回到这里，意味着任务被调度出去了
        }
    }
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.inner_exclusive_access().get_user_token();
    token
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task().unwrap().inner_exclusive_access().get_trap_cx()
}

pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(
            switched_task_cx_ptr,
            idle_task_cx_ptr,
        );
    }
}
