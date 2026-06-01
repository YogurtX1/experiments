#![no_std]
#![no_main]
use user::*;


#[no_mangle]
use crate::yield_;
fn main() -> i32 {
    println!("Hello, world!");
    0
}
