use crate::loader::get_num_app;
use core::cell::RefCell;

pub mod context;
mod switch;

pub use context::TaskContext;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}

#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
}

pub struct TaskManager {
    num_app: usize,
    inner: RefCell<TaskManagerInner>,
}

unsafe impl Sync for TaskManager {}

pub struct TaskManagerInner {
    tasks: [TaskControlBlock; 4],
    current_task: usize,
}

pub static mut TASK_MANAGER: TaskManager = TaskManager {
    num_app: 0,
    inner: RefCell::new(TaskManagerInner {
        tasks: [TaskControlBlock {
            task_status: TaskStatus::UnInit,
            task_cx: TaskContext::zero_init(),
        }; 4],
        current_task: 0,
    }),
};

pub fn init() {
    unsafe {
        let num_app = get_num_app();
        TASK_MANAGER.num_app = num_app;

        let mut inner = TASK_MANAGER.inner.borrow_mut();
        for i in 0..num_app {
            inner.tasks[i].task_status = TaskStatus::Ready;
            inner.tasks[i].task_cx = crate::loader::init_app_cx(i);
        }
    }
    println!("[kernel] TaskManager successfully initialized.");
}

impl TaskManager {
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.borrow_mut();
        inner.tasks[0].task_status = TaskStatus::Running;

        // 修复: 用局部变量做二级指针，让 __switch 的 ld t1,0(a1) 正确解引用
        let task_cx_ptr: *const TaskContext = &inner.tasks[0].task_cx;
        let next_task_cx_ptr2 = &task_cx_ptr as *const *const TaskContext as *const usize;
        let first_kstack_top = inner.tasks[0].task_cx.sp;

        let mut _unused_context_ptr: *const usize = core::ptr::null();
        let current_task_cx_ptr2 = &mut _unused_context_ptr as *mut *const usize as *const usize;

        drop(inner);

        unsafe {
            core::arch::asm!("csrw sscratch, {}", in(reg) first_kstack_top);
            switch::__switch(current_task_cx_ptr2, next_task_cx_ptr2);
        }
        panic!("unreachable in run_first_task!");
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.borrow();
        let current = inner.current_task;
        for i in 1..=self.num_app {
            let next = (current + i) % self.num_app;
            if inner.tasks[next].task_status == TaskStatus::Ready {
                return Some(next);
            }
        }
        None
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.borrow_mut();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;

            // 修复: 用局部变量做二级指针，让 __switch 正确解引用
            let current_ptr: *const TaskContext = &inner.tasks[current].task_cx;
            let next_ptr: *const TaskContext = &inner.tasks[next].task_cx;

            let current_task_cx_ptr2 = &current_ptr as *const *const TaskContext as *const usize;
            let next_task_cx_ptr2 = &next_ptr as *const *const TaskContext as *const usize;

            let next_kstack_top = inner.tasks[next].task_cx.sp;

            drop(inner);

            unsafe {
                core::arch::asm!("csrw sscratch, {}", in(reg) next_kstack_top);
                switch::__switch(current_task_cx_ptr2, next_task_cx_ptr2);
            }
        } else {
            panic!("All applications completed!");
        }
    }
}

pub fn run_first_task() {
    unsafe { TASK_MANAGER.run_first_task(); }
}

pub fn suspend_current_and_run_next() {
    unsafe {
        TASK_MANAGER.mark_current_suspended();
        TASK_MANAGER.run_next_task();
    }
}

pub fn exit_current_and_run_next() {
    unsafe {
        TASK_MANAGER.mark_current_exited();
        TASK_MANAGER.run_next_task();
    }
}

pub fn run_next_task() {
    unsafe { TASK_MANAGER.run_next_task(); }
}
