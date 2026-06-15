#![no_std]
#![no_main]

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
    trap::init();
    task::init();
    init_app_memory();
    println!("[kernel] Starting first task...");
    task::run_first_task();
}

fn init_app_memory() {
    use crate::config::{APP_BASE_ADDRESS, APP_SIZE_LIMIT, PAGE_SIZE, TRAMPOLINE};
    use crate::mm::address::VirtAddr;
    use crate::mm::{MapArea, MapType, MemorySet};
    use crate::mm::page_table::PTEFlags;

    let num_app = loader::get_num_app();
    let mut tm = crate::task::TASK_MANAGER.exclusive_access();

    // 获取 trampoline 物理页号 (从内核页表查询)
    let trampoline_frame = crate::mm::KERNEL_SPACE
        .exclusive_access()
        .translate(TRAMPOLINE.into())
        .expect("trampoline not found in kernel page table");

    for i in 0..num_app {
        let app_data = loader::get_app_data(i);
        let app_end = APP_BASE_ADDRESS + app_data.len();
        let app_pages = (app_end - APP_BASE_ADDRESS + PAGE_SIZE - 1) / PAGE_SIZE;
        let app_size = app_pages * PAGE_SIZE;

        // 先获取任务相关的数据（trap_cx_ppn）
        let trap_cx_ppn = {
            let task = &tm.inner.tasks[i];
            task.trap_cx_ppn
        };

        let mut memory_set = MemorySet::new();
        let user_satp = memory_set.satp();

        // 1. 映射用户代码区域
        let code_end: crate::mm::address::VirtAddr = (APP_BASE_ADDRESS + app_size).into();
        memory_set.push(
            MapArea::new(
                APP_BASE_ADDRESS.into(),
                code_end,
                MapType::Framed,
                PTEFlags::R | PTEFlags::W | PTEFlags::X | PTEFlags::U,
            ),
            Some(app_data),
        );

        // 2. 映射用户栈
        let stack_bottom: crate::mm::address::VirtAddr =
            (APP_BASE_ADDRESS + APP_SIZE_LIMIT).into();
        let stack_top: crate::mm::address::VirtAddr =
            (APP_BASE_ADDRESS + APP_SIZE_LIMIT + PAGE_SIZE * 16).into();
        memory_set.push(
            MapArea::new(
                stack_bottom,
                stack_top,
                MapType::Framed,
                PTEFlags::R | PTEFlags::W | PTEFlags::U,
            ),
            None,
        );

        // 3. 映射 TrapContext (与内核相同的物理帧)
        let trap_cx_user: usize = crate::config::TRAP_CONTEXT_USER;
        let trap_cx_frame = crate::mm::address::PhysPageNum(trap_cx_ppn);
        memory_set.push(
            MapArea::new(
                trap_cx_user.into(),
                (trap_cx_user + PAGE_SIZE).into(),
                MapType::Fixed(trap_cx_frame),
                PTEFlags::R | PTEFlags::W,
            ),
            None,
        );

        // 4. 映射 trampoline
        memory_set.map_trampoline(trampoline_frame, TRAMPOLINE);

        // 更新任务
        let task = &mut tm.inner.tasks[i];
        let trap_cx = task.get_trap_cx();
        let user_sp: usize = stack_top.into();

        // TrapContext 页对齐 (kstack_top - PAGE_SIZE), offset = 0
        let trap_cx_vaddr = crate::config::TRAP_CONTEXT_USER;

        trap_cx.sepc = APP_BASE_ADDRESS;
        trap_cx.set_sp(user_sp);
        trap_cx.user_satp = user_satp;
        trap_cx.trap_cx_user = trap_cx_vaddr;
        trap_cx.trap_cx_kernel = trap_cx_vaddr;

        let trap_cx_pa = task.get_trap_cx_ptr();
        println!("[kernel] App {}: entry={:#x}, sp={:#x}, satp={:#x}, trap_cx_pa={:#x}",
            i, APP_BASE_ADDRESS, user_sp, user_satp, trap_cx_pa);

        // 验证: kernel_sp - PAGE_SIZE == TrapContext 物理地址
        assert_eq!(trap_cx.kernel_sp - PAGE_SIZE, trap_cx_pa,
            "kernel_sp mismatch! kernel_sp={:#x} PAGE_SIZE={} trap_cx_pa={:#x}",
            trap_cx.kernel_sp, PAGE_SIZE, trap_cx_pa);

        task.memory_set = Some(memory_set);
    }

    // 刷新指令缓存: copy_data 通过内核恒等映射写入物理内存，
    // CPU I-Cache 可能还存有旧数据，sret 后取指将读到非法指令
    unsafe {
        core::arch::asm!("fence.i");
    }
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
