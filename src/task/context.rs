#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct TaskContext {
    pub ra: usize,   // 偏移 0   字节
    pub sp: usize,   // 偏移 8   字节
    pub s0: usize,   // 偏移 16  字节
    pub s1: usize,   // 偏移 24  字节
    pub s2: usize,   // 偏移 32  字节
    pub s3: usize,   // 偏移 40  字节
    pub s4: usize,   // 偏移 48  字节
    pub s5: usize,   // 偏移 56  字节
    pub s6: usize,   // 偏移 64  字节
    pub s7: usize,   // 偏移 72  字节
    pub s8: usize,   // 偏移 80  字节
    pub s9: usize,   // 偏移 88  字节
    pub s10: usize,  // 偏移 96  字节
    pub s11: usize,  // 偏移 104 字节
}

impl TaskContext {
    // 🎯 核心修复：加上 const 关键字，允许它在全局静态变量初始化时被调用
    pub const fn zero_init() -> Self {
        Self {
            ra: 0, sp: 0,
            s0: 0, s1: 0, s2: 0, s3: 0, s4: 0, s5: 0, s6: 0, s7: 0, s8: 0, s9: 0, s10: 0, s11: 0,
        }
    }

    pub fn goto_trap_return(kstack_ptr: usize) -> Self {
        extern "C" {
            fn __restore();
        }
        Self {
            ra: __restore as usize,
            sp: kstack_ptr,
            s0: 0, s1: 0, s2: 0, s3: 0, s4: 0, s5: 0, s6: 0, s7: 0, s8: 0, s9: 0, s10: 0, s11: 0,
        }
    }
}
