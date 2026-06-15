#![no_std]
#![no_main]

extern crate user;
use user::println;

#[no_mangle]
fn main() -> i32 {
    let mut val: u64 = 1;
    for i in 0..100 {
        println!("Power_5 test {}: val = {}", i, val);
        val = val.wrapping_mul(5);
    }
    println!("Power_5 OK!");
    0
}
