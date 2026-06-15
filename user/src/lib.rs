#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;

// 纯汇编入口和系统调用蹦床（完全绕过 Rust 编译器对寄存器的干扰）
global_asm!(include_str!("entry.S"));

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    sys_exit(-1);
}

extern "C" {
    fn _user_syscall(id: usize, arg0: usize, arg1: usize, arg2: usize) -> isize;
}

pub fn sys_exit(xstate: i32) -> ! {
    unsafe { _user_syscall(93, xstate as usize, 0, 0); }
    panic!("Unreachable in sys_exit!");
}

pub fn sys_write(fd: usize, buf: &[u8]) -> isize {
    unsafe { _user_syscall(64, fd, buf.as_ptr() as usize, buf.len()) }
}

pub fn sys_yield() -> isize {
    unsafe { _user_syscall(124, 0, 0, 0) }
}

pub fn sys_get_time() -> isize {
    unsafe { _user_syscall(169, 0, 0, 0) }
}

use core::fmt::{self, Write};

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        sys_write(1, s.as_bytes());
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n"); };
    ($($arg:tt)*) => {
        $crate::print(format_args!("{}\n", format_args!($($arg)*)));
    };
}
