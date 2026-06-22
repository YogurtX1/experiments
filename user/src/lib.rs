#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::arch::global_asm;
use core::panic::PanicInfo;

use buddy_system_allocator::LockedHeap;

pub mod console;
mod syscall;

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

pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code)
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

use syscall::*;

pub fn read(fd: usize, buf: &mut [u8]) -> isize { sys_read(fd, buf) }
pub fn getpid() -> isize { sys_getpid() }
pub fn fork() -> isize { sys_fork() }
pub fn exec(path: &str) -> isize { sys_exec(path) }

pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => { yield_(); }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => { yield_(); }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}

pub fn sleep(period_ms: usize) {
    let start = sys_get_time();
    while sys_get_time() < start + period_ms as isize {
        sys_yield();
    }
}

pub fn yield_() -> isize { sys_yield() }

// ─── 用户堆分配器 ────────────────────────────────────────
const USER_HEAP_SIZE: usize = 16384;
static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start_rust() {
    unsafe {
        HEAP.lock()
            .init(HEAP_SPACE.as_ptr() as usize, USER_HEAP_SIZE);
    }
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
