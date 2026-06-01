#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::arch::global_asm;

#[macro_use]
pub mod console;
pub mod config;
pub mod lang_items;
pub mod loader;
pub mod sbi;
pub mod syscall;
pub mod task;
pub mod trap;

global_asm!(include_str!("entry.asm"));

// 🎯 核心修复：在末尾加上 ".section .text\n"！
// 这样能确保 linkage.S 待在数据段的同时，后面内核的 rust_main 依然安全地待在代码段！
global_asm!(
    ".section .data\n",
    include_str!(concat!(env!("OUT_DIR"), "/linkage.S")),
    ".section .text\n"
);

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    clear_bss();
    println!("[kernel] Hello, world!");
    trap::init();
    loader::load_apps();
    
    task::init();
    task::run_first_task();
    panic!("Unreachable in rust_main!");
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe {
        (a as *mut u8).write_volatile(0);
    });
}
