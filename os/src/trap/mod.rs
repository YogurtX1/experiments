pub mod context;

use core::arch::global_asm;
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

use crate::config::PAGE_SIZE;
use crate::mm::address::VirtAddr;
use crate::syscall::syscall;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next, TASK_MANAGER};
pub use context::TrapContext;

global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as *const () as usize, stvec::TrapMode::Direct);
    }
}

pub fn set_trampoline_stvec(trampoline_va: usize) {
    unsafe {
        stvec::write(trampoline_va, stvec::TrapMode::Direct);
    }
    println!("[kernel] stvec set to trampoline: {:#x}", trampoline_va);
}

pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &TrapContext {
    let scause = scause::read();
    let stval = stval::read();

    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            let syscall_id = cx.x[17];
            let args = [cx.x[10], cx.x[11], cx.x[12]];

            if syscall_id == 64 {
                // sys_write
                let fd = args[0];
                let user_buf = args[1];
                let len = args[2];

                if fd == 1 {
                    let tm = TASK_MANAGER.exclusive_access();
                    let cur = tm.inner.current_task;
                    let ms = tm.inner.tasks[cur].memory_set.as_ref()
                        .expect("No user memory set");

                    let mut remaining = len;
                    let mut current_va = VirtAddr(user_buf);

                    while remaining > 0 {
                        let page_offset = current_va.page_offset();
                        let chunk_size = core::cmp::min(remaining, PAGE_SIZE - page_offset);

                        let pa = ms.page_table.translate_va(current_va)
                            .expect("sys_write: unmapped user buffer");

                        let phys_addr = pa.0;
                        let slice = unsafe {
                            core::slice::from_raw_parts(phys_addr as *const u8, chunk_size)
                        };
                        if let Ok(s) = core::str::from_utf8(slice) {
                            print!("{}", s);
                        }

                        remaining -= chunk_size;
                        current_va = VirtAddr(current_va.0 + chunk_size);
                    }

                    cx.x[10] = len;
                } else {
                    cx.x[10] = (-1isize) as usize;
                }
            } else if syscall_id == 93 {
                // sys_exit
                println!("[kernel] App exited with code={}", args[0] as isize);
                exit_current_and_run_next();
            } else {
                let result = syscall(syscall_id, args);
                cx.x[10] = result as usize;
            }
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            println!(
                "[kernel] PageFault in app, addr={:#x}, ip={:#x}, kernel killed it",
                stval, cx.sepc
            );
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!(
                "[kernel] IllegalInstruction in app, ip={:#x}, kernel killed it",
                cx.sepc
            );
            exit_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            crate::timer::set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    cx
}
