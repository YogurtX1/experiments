#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]

#[macro_use]
pub mod console;
pub mod syscall;
pub mod lang_items;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main());
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("Cannot find main!");
}

// 🎯 降维打击：双重符号绑定，彻底堵死所有链接器找不到符号的可能性！
fn clear_bss() {
    extern "C" {
        // 尝试绑定第一种可能的命名
        fn sbss();
        fn ebss();
        // 尝试绑定第二种可能的命名
        fn start_bss();
        fn end_bss();
    }
    
    // 动态获取真正有效的 BSS 边界
    let start = if (sbss as usize) != 0 { sbss as usize } else { start_bss as usize };
    let end = if (ebss as usize) != 0 { ebss as usize } else { end_bss as usize };

    if start != 0 && end != 0 && start < end {
        (start..end).for_each(|addr| unsafe {
            (addr as *mut u8).write_volatile(0);
        });
    }
}

pub fn write(fd: usize, buf: &[u8]) -> isize { 
    syscall::sys_write(fd, buf.as_ptr(), buf.len()) 
}

pub fn exit(exit_code: i32) -> ! { 
    syscall::sys_exit(exit_code) 
}

pub fn yield_() -> isize { 
    syscall::sys_yield() 
}
