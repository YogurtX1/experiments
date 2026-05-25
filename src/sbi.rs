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
