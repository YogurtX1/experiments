pub mod context;

use core::arch::global_asm;
use riscv::register::{
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

use crate::config::TRAMPOLINE;
use crate::syscall::syscall;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next, current_trap_cx};
pub use context::TrapContext;

global_asm!(include_str!("trap.S"));

pub fn init() {
    unsafe {
        stvec::write(TRAMPOLINE, stvec::TrapMode::Direct);
    }
}

pub fn set_trampoline_stvec(trampoline_va: usize) {
    unsafe {
        stvec::write(trampoline_va, stvec::TrapMode::Direct);
    }
}

pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

pub fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_handler as *const () as usize, stvec::TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let scause = scause::read();
    let stval = stval::read();
    // ★ 诊断: 比较 CPU sstatus CSR 与内存 TrapContext.sstatus
    let cpu_sstatus = unsafe { riscv::register::sstatus::read().bits() };
    let mem_sstatus = current_trap_cx().sstatus;
    crate::println!("[trap_handler] entered! scause={:?}, stval={:#x}, sepc={:#x}",
        scause.cause(), stval, current_trap_cx().sepc);
    crate::println!("[trap_handler] CPU sstatus={:#x}, TrapCtx sstatus={:#x}, FS_CPU={}, FS_MEM={}",
        cpu_sstatus, mem_sstatus,
        (cpu_sstatus >> 13) & 3, (mem_sstatus >> 13) & 3
    );
    if cpu_sstatus != mem_sstatus {
        crate::println!("[trap_handler] *** MISMATCH: CPU sstatus != TrapCtx sstatus! ***");
    }
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // jump to next instruction anyway
            let cx = current_trap_cx();
            cx.sepc += 4;
            // get system call return value
            let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]);
            // cx is changed during sys_exec, so we have to get it again
            let cx = current_trap_cx();
            cx.x[10] = result as usize;
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) |
        Trap::Exception(Exception::InstructionFault) |
        Trap::Exception(Exception::InstructionPageFault) |
        Trap::Exception(Exception::LoadFault) |
        Trap::Exception(Exception::LoadPageFault) => {
            println!(
                "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.",
                scause.cause(),
                stval,
                current_trap_cx().sepc,
            );
            exit_current_and_run_next(-2);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            let cx = current_trap_cx();
            let sepc = cx.sepc;
            // ★ 诊断: 直接 dump 故障地址处的 32 字节 (内核恒等映射)
            crate::print!("[kernel] Hex dump at sepc={:#x}: ", sepc);
            for offset in 0..32 {
                let va = sepc.wrapping_add(offset);
                // 先尝试通过用户页表翻译
                let byte = {
                    use crate::mm::page_table::PageTable;
                    use crate::mm::VirtAddr;
                    let pt = PageTable::from_token(cx.user_satp);
                    pt.translate_va(VirtAddr::from(va))
                        .map(|pa| unsafe { *(pa.0 as *const u8) })
                };
                match byte {
                    Some(b) => crate::print!("{:02x} ", b),
                    None => { crate::print!("?? "); break; }
                }
            }
            crate::println!();
            println!(
                "[kernel] IllegalInstruction in application, sepc={:#x}, stval={:#x}, core dumped.",
                sepc, stval
            );
            exit_current_and_run_next(-3);
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            crate::timer::set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
    // ★ 确保定时器已启用 (首次返回用户态时激活, STIE 跨 trap 保持)
    enable_timer_interrupt();
    crate::timer::set_next_trigger();
    // ★ 修复 sstatus.FS: 确保返回用户态时 FS=Dirty, 防止 FP 指令触发 IllegalInstruction
    //    FS=Initial(01) 在 sret 跨特权级时可能被清零, FS=Dirty(11) 可被保留.
    {
        let cx = current_trap_cx();
        cx.sstatus &= !(3 << 13);   // 清除 FS 字段
        cx.sstatus |= (3 << 13);     // 设置 FS=Dirty
    }
    trap_return();
}

fn trap_return() -> ! {
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let alltraps_va = __alltraps as *const () as usize;
    let restore_offset = __restore as *const () as usize - alltraps_va;
    let restore_va = TRAMPOLINE + restore_offset;

    let cx = current_trap_cx();
    let cx_phys = cx as *const TrapContext as usize;

    // 将 stvec 重新指向 trampoline（为下次用户态 trap 准备）
    set_trampoline_stvec(TRAMPOLINE);

    unsafe {
        core::arch::asm!(
            "fence.i",
            "jr {restore}",
            restore = in(reg) restore_va,
            in("a0") cx_phys,
        );
    }
    unreachable!();
}
