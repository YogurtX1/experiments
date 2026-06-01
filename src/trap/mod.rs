use core::arch::global_asm; // 🎯 关键：引入全局汇编宏
use riscv::register::{
    scause::{self, Trap, Exception},
    stval,
    stvec,
    utvec::TrapMode,
};
use crate::syscall::syscall;
use crate::task::run_next_task;

// 🎯 核心修复：把保存/恢复用户态上下文的 trap.S 嵌入进来！
global_asm!(include_str!("trap.S"));

pub use context::TrapContext;
pub mod context;

pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::InstructionFault) => {
            println!("[kernel] InstructionFault in application, bad addr = {:#x}, core dumped.", stval);
            run_next_task();
        }
        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
    cx
}
