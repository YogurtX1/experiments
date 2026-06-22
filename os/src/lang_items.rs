use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "[kernel] Panic at {}:{}: {}",
            location.file(),
            location.line(),
            info.message(),
        );
    } else {
        println!("[kernel] Panic: {}", info.message());
    }
    crate::sbi::shutdown();
}
