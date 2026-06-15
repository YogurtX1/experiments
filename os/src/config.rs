pub const USER_STACK_SIZE: usize = 4096 * 4;
pub const KERNEL_STACK_SIZE: usize = 4096 * 4;
pub const MAX_APP_NUM: usize = 16;
pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000;
pub const CLOCK_FREQ: usize = 12500000;

// 虚拟内存相关常量
pub const PAGE_SIZE_BITS: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SIZE_BITS; // 4KB
pub const MEMORY_END: usize = 0x88000000; // 128MiB 物理内存结束
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1; // 跳板页面: 最高一页
pub const TRAP_CONTEXT_USER: usize = TRAMPOLINE - PAGE_SIZE; // 用户态 Trap 上下文
pub const KERNEL_HEAP_SIZE: usize = 0x10_0000; // 1MiB 内核堆
pub const KERNEL_PAGE_TABLE_BASE: usize = 0x8080_0000; // 内核初始页表位置
