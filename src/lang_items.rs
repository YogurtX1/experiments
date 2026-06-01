use core::panic::PanicInfo; // 🎯 修复：导入 PanicInfo 类型
use crate::sbi::shutdown;   // 🎯 修复：引入刚刚实现的关机服务

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "[kernel] Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("[kernel] Panicked: {}", info.message().unwrap());
    }
    
    // 🎯 终极修复：直接关机！
    // 关机后 QEMU 进程在宿主机上会立刻死掉，容器的 CPU 占用率瞬间归零！
    shutdown();
}
