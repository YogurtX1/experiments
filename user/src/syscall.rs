use core::arch::asm;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") args[0] => ret,
            in("a1") args[1],
            in("a2") args[2],
            in("a7") id,
        );
    }
    ret
}

// 🎯 核心修复：定义标准的符合第三章预期的底层的 sys_exit
pub fn sys_exit(exit_code: i32) -> ! {
    syscall(93, [exit_code as usize, 0, 0]);
    panic!("sys_exit never returns!");
}

pub fn sys_write(fd: usize, buffer: *const u8, len: usize) -> isize {
    syscall(64, [fd, buffer as usize, len])
}

pub fn sys_yield() -> isize {
    syscall(124, [0, 0, 0])
}
