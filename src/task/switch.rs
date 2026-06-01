use core::arch::global_asm;

global_asm!(include_str!("switch.S"));

extern "C" {
    // 传入二级指针的物理数值地址
    pub fn __switch(current_task_cx_ptr2: *const usize, next_task_cx_ptr2: *const usize);
}
