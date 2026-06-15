use crate::config::TRAMPOLINE;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct TaskContext {
    pub ra: usize,
    pub sp: usize,
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
}

impl TaskContext {
    pub const fn zero_init() -> Self {
        Self {
            ra: 0, sp: 0,
            s0: 0, s1: 0, s2: 0, s3: 0, s4: 0, s5: 0,
            s6: 0, s7: 0, s8: 0, s9: 0, s10: 0, s11: 0,
        }
    }

    pub fn goto_trap_return(kstack_top: usize) -> Self {
        extern "C" {
            fn __alltraps();
            fn __goto_restore();
        }
        // 计算 __goto_restore 在 trampoline 页面内的偏移
        let offset = __goto_restore as *const () as usize - __alltraps as *const () as usize;
        // ra 必须指向 trampoline 映射地址，因为 __restore 会切换页表
        let trampoline_ra = TRAMPOLINE + offset;

        Self {
            ra: trampoline_ra,
            sp: kstack_top,
            s0: 0, s1: 0, s2: 0, s3: 0, s4: 0, s5: 0,
            s6: 0, s7: 0, s8: 0, s9: 0, s10: 0, s11: 0,
        }
    }
}
