use core::arch::asm;

const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
const SBI_SET_TIMER: usize = 0;
const SBI_SHUTDOWN: usize = 8;

pub fn console_putchar(c: usize) {
    // ★ black_box 防止编译器 DCE 消除 ecall 指令
    core::hint::black_box(sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0));
}

/// 底层逐字符输出，完全绕过 format_args!/println! 宏机制
/// 内联 asm! 直接在函数中，杜绝编译器优化消除
#[inline(never)]
pub fn debug_puts(s: &str) {
    for byte in s.bytes() {
        unsafe {
            core::arch::asm!(
                "ecall",
                in("a7") 1usize,
                in("a0") byte as usize,
                in("a1") 0usize,
                in("a2") 0usize,
            );
        }
    }
}

pub fn console_getchar() -> usize {
    sbi_call(SBI_CONSOLE_GETCHAR, 0, 0, 0)
}

#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            in("a7") which,
            in("a0") arg0,
            in("a1") arg1,
            in("a2") arg2,
            lateout("a0") ret,
        );
    }
    ret
}

pub fn shutdown() -> ! {
    sbi_call(SBI_SHUTDOWN, 0, 0, 0);
    panic!("It should shutdown!");
}

pub fn set_timer(stime_value: u64) {
    #[cfg(target_pointer_width = "32")]
    sbi_call(
        SBI_SET_TIMER,
        stime_value as usize,
        (stime_value >> 32) as usize,
        0,
    );
    #[cfg(target_pointer_width = "64")]
    sbi_call(SBI_SET_TIMER, stime_value as usize, 0, 0);
}
