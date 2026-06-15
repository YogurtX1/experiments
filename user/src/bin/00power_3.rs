#![no_std]
#![no_main]

extern crate user;
use user::println;

#[no_mangle]
fn main() -> i32 {
    let mut val: u64 = 1;
    for i in 0..100 {
        println!("Power_3 test {}: val = {}", i, val);
        val = val.wrapping_mul(3);
    }
    println!("Power_3 OK!");
    0
}
