use crate::config::TRAP_CONTEXT_USER;
use crate::sync::UPSafeCell;
use super::{PidHandle, pid_alloc, KernelStack};
use super::TaskContext;
use super::TaskStatus;
use crate::mm::MemorySet;
use crate::mm::address::{PhysPageNum, VirtAddr};
use alloc::sync::{Weak, Arc};
use alloc::vec::Vec;
use crate::trap::TrapContext;

pub struct TaskControlBlock {
    // immutable
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    // mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }
}

impl TaskControlBlock {
    pub fn inner_exclusive_access(&self) -> crate::sync::ExclusiveRef<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }

    pub fn new(elf_data: &[u8]) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT_USER).into())
            .unwrap();
        let trap_cx_ppn_pa = trap_cx_ppn.0 * crate::config::PAGE_SIZE;
        let user_token = memory_set.token();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        // push a task context which goes to trap_return to the top of kernel stack
        let task_control_block = Self {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe { UPSafeCell::new(TaskControlBlockInner {
                trap_cx_ppn,
                base_size: user_sp,
                task_cx: TaskContext::goto_trap_return(kernel_stack_top, trap_cx_ppn),
                task_status: TaskStatus::Ready,
                memory_set,
                parent: None,
                children: Vec::new(),
                exit_code: 0,
            })},
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
            user_token,
            TRAP_CONTEXT_USER,
            TRAP_CONTEXT_USER,
            trap_cx_ppn_pa,
        );
        task_control_block
    }

    pub fn exec(&self, elf_data: &[u8]) {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT_USER).into())
            .unwrap();
        let trap_cx_ppn_pa = trap_cx_ppn.0 * crate::config::PAGE_SIZE;
        let user_token = memory_set.token();

        // **** access inner exclusively
        let mut inner = self.inner_exclusive_access();
        // substitute memory_set
        inner.memory_set = memory_set;
        // update trap_cx ppn
        inner.trap_cx_ppn = trap_cx_ppn;
        // update task_cx (s0 需要指向新的 TrapContext 物理页)
        inner.task_cx = TaskContext::goto_trap_return(self.kernel_stack.get_top(), trap_cx_ppn);
        // initialize trap_cx
        let trap_cx = inner.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize,
            user_token,
            TRAP_CONTEXT_USER,
            TRAP_CONTEXT_USER,
            trap_cx_ppn_pa,
        );
        // **** release inner automatically
    }

    pub fn fork(self: &Arc<TaskControlBlock>) -> Arc<TaskControlBlock> {
        // ---- access parent PCB exclusively
        let mut parent_inner = self.inner_exclusive_access();
        // copy user space(include trap context)
        let memory_set = MemorySet::from_existed_user(
            &parent_inner.memory_set
        );
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT_USER).into())
            .unwrap();
        let trap_cx_ppn_pa = trap_cx_ppn.0 * crate::config::PAGE_SIZE;
        // ★ 保存子进程的 user_token (memory_set 即将被 move)
        let child_user_token = memory_set.token();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        // fork: parent -> child
        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe { UPSafeCell::new(TaskControlBlockInner {
                trap_cx_ppn,
                base_size: parent_inner.base_size,
                task_cx: TaskContext::goto_trap_return(kernel_stack_top, trap_cx_ppn),
                task_status: TaskStatus::Ready,
                memory_set,
                parent: Some(Arc::downgrade(self)),
                children: Vec::new(),
                exit_code: 0,
            })},
        });
        // add child
        parent_inner.children.push(task_control_block.clone());
        // ★ 修复子进程 TrapContext: 更新内核栈指针、TrapContext 物理地址、用户页表
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;
        trap_cx.trap_cx_ppn_pa = trap_cx_ppn_pa;
        trap_cx.user_satp = child_user_token; // ← 关键修复: 子进程必须用自己的页表
        // return
        task_control_block
        // ---- release parent PCB automatically
        // **** release children PCB automatically
    }

    pub fn getpid(&self) -> usize {
        self.pid.0
    }
}

extern "C" {
    fn trap_handler();
}

use crate::mm::KERNEL_SPACE;
