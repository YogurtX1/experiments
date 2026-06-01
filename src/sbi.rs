#![allow(dead_code)]

use core::arch::asm;

const SBI_CONSOLE_PUTCHAR: usize = 1;

#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let ret;
    unsafe {
        asm!(
            "ecall",
            in("x10") arg0,
            in("x11") arg1,
            in("x12") arg2,
            in("x17") which,
            lateout("x10") ret,
        );
    }
    ret
}

pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

/// 🎯 让 OpenSBI 帮我们关闭整个 QEMU 虚拟机
pub fn shutdown() -> ! {
    // 经典的老版本 SBI 关机调用：
    // 功能号 (which) = 9 (SBI_SHUTDOWN)
    // 参数全部填 0
    sbi_call(9, 0, 0, 0);
    
    // 如果 SBI 关机失败，用死循环兜底满足 -> ! 返回类型
    loop {}
}
