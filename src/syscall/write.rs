use crate::sbi::console_putchar;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    if fd == 1 {
        for i in 0..len {
            unsafe {
                console_putchar(*buf.add(i) as usize);
            }
        }
        len as isize
    } else {
        -1
    }
}
