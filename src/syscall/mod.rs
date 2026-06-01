const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;

use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_WRITE => {
            let fd = args[0];
            let buffer = args[1] as *const u8;
            let len = args[2];
            if fd == 1 {
                let slice = unsafe { core::slice::from_raw_parts(buffer, len) };
                if let Ok(str) = core::str::from_utf8(slice) {
                    print!("{}", str);
                    return len as isize;
                }
            }
            -1
        }
        SYSCALL_EXIT => {
            println!("[kernel] Application exited with code {}", args[0] as i32);
            // 🎯 联动应用的主动退出
            exit_current_and_run_next();
            0
        }
        SYSCALL_YIELD => {
            // 🎯 联动应用的主动让出 CPU（多道程序协作的关键！）
            suspend_current_and_run_next();
            0
        }
        _ => {
            panic!("Unsupported syscall_id: {}", syscall_id);
        }
    }
}
