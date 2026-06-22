#![no_std]
#![no_main]
#![feature(panic_info_message)]

extern crate alloc;

use core::arch::global_asm;

#[macro_use]
pub mod console;
pub mod config;
pub mod lang_items;
pub mod loader;
pub mod mm;
pub mod sbi;
pub mod sync;
pub mod syscall;
pub mod task;
pub mod timer;
pub mod trap;

global_asm!(include_str!("entry.asm"));

global_asm!(
    ".section .data\n",
    include_str!("linkage.S"),
    ".section .text\n"
);

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    clear_bss();
    println!("[kernel] Hello, world!");
    mm::init();
    println!("[kernel] back to world!");
    mm::remap_test();
    task::add_initproc();
    trap::init();
    // 注意: 不在此处启用定时器中断!
    // 定时器在 run_tasks() 内部、第一次 __switch 进入用户态之前启用。
    loader::list_apps();
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as *const () as usize..ebss as *const () as usize).for_each(|a| unsafe {
        (a as *mut u8).write_volatile(0);
    });
}
