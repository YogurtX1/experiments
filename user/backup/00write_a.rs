#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::panic::PanicInfo;
use core::arch::asm;

const SYSCALL_YIELD: usize = 124;
const SYSCALL_WRITE: usize = 64;

fn syscall(id: usize, args: [usize; 3]) -> isize {
    let ret: isize;
    unsafe {
        asm!(
            "ecall",
            in("a7") id,
            in("a0") args[0],
            in("a1") args[1],
            in("a2") args[2],
            lateout("a0") ret,
            options(nostack)
        );
    }
    ret
}

fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    syscall(SYSCALL_WRITE, [fd, buf as usize, len])
}

fn sys_yield() -> isize {
    syscall(SYSCALL_YIELD, [0, 0, 0])
}

fn print(s: &str) {
    sys_write(1, s.as_ptr(), s.len());
}

macro_rules! print {
    ($($arg:tt)*) => ({
        let mut s = stringify!($($arg)*);
        print(s);
    });
}

macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ({
        print!($($arg)*);
        print!("\n");
    });
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

const WIDTH: usize = 10;
const HEIGHT: usize = 5;

#[no_mangle]
fn main() -> i32 {
    for i in 0..HEIGHT {
        for _ in 0..WIDTH { print!("A"); }
        println!(" [{}/{}]", i + 1, HEIGHT);
        sys_yield();
    }
    println!("Test write_a OK!");
    0
}
