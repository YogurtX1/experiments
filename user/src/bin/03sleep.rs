#![no_std]
#![no_main]

extern crate user;
use user::{println, sys_get_time, sys_yield};

#[no_mangle]
fn main() -> i32 {
    let start = sys_get_time();
    println!("[sleep] start at {}", start);
    let mut count = 0;
    loop {
        count += 1;
        if count == 30 {
            break;
        }
        sys_yield();
    }
    let end = sys_get_time();
    println!("[sleep] end at {}, duration = {}ms", end, end - start);
    0
}
