#![no_std]

#[no_mangle]
pub extern "C" fn _start() -> ! {
    extern "Rust" {
        fn main() -> i32;
    }
    unsafe {
        main();
    }
    loop {}
}
