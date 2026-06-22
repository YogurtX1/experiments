pub mod address;
pub mod frame;
pub mod heap;
pub mod map_area;
pub mod memory_set;
pub mod page_table;
pub mod vpn_range;

use crate::config::{KERNEL_HEAP_SIZE, MEMORY_END, PAGE_SIZE, TRAMPOLINE};
use frame::init_frame_allocator;
use lazy_static::lazy_static;

pub use frame::{frame_alloc, frame_dealloc, alloc_pte_frame};
pub use map_area::{MapArea, MapType};
pub use memory_set::MemorySet;
pub use page_table::{
    PageTable, PageTableEntry,
    translated_byte_buffer,
    translated_str,
    translated_refmut,
};
pub use page_table::PTEFlags as MapPermission;

lazy_static! {
    pub static ref KERNEL_SPACE: crate::sync::UPSafeCell<MemorySet> =
        unsafe { crate::sync::UPSafeCell::new(MemorySet::new()) };
}

pub fn init_kernel_memory() {
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
        fn ekernel();
    }

    // 1. 初始化堆
    heap::init_heap();

    // 2. 初始化帧分配器 (跳过堆占用的物理页)
    let ekernel_addr = ekernel as *const () as usize;

    let _ek_floor = address::PhysPageNum(ekernel_addr / PAGE_SIZE);
    let ek_ceil = address::PhysPageNum((ekernel_addr + PAGE_SIZE - 1) / PAGE_SIZE);
    let heap_pages = (KERNEL_HEAP_SIZE + PAGE_SIZE - 1) / PAGE_SIZE;
    let frame_start = address::PhysPageNum(ek_ceil.0 + heap_pages);
    let end = address::PhysPageNum(MEMORY_END / PAGE_SIZE);

    init_frame_allocator(frame_start, end);
    println!(
        "[kernel] FrameAllocator: ppn [{:#x}, {:#x})",
        frame_start.0, end.0
    );

    // 3. 初始化 trampoline 页面
    let trampoline_frame = init_trampoline();
    println!("[kernel] Trampoline at phys ppn={:?}, virt={:#x}", trampoline_frame, TRAMPOLINE);

    // 4. 构建内核地址空间 (所有内核段统一 R|W|X, 避免页边界重叠)
    let mut kernel_space = KERNEL_SPACE.exclusive_access();

    let stext_addr = stext as *const () as usize;
    let ekernel_addr_val = ekernel as *const () as usize;

    println!(
        "[kernel] .text [{:#x}, {:#x})",
        stext_addr, etext as *const () as usize
    );
    println!(
        "[kernel] .rodata [{:#x}, {:#x})",
        srodata as *const () as usize, erodata as *const () as usize
    );
    println!(
        "[kernel] .data [{:#x}, {:#x})",
        sdata as *const () as usize, edata as *const () as usize
    );
    println!(
        "[kernel] .bss [{:#x}, {:#x})",
        sbss as *const () as usize, ebss as *const () as usize
    );

    // 映射整个内核区域 (stext..ekernel, R|W|X)
    kernel_space.push(
        MapArea::new(
            stext_addr.into(),
            ekernel_addr_val.into(),
            MapType::Identical,
            page_table::PTEFlags::R | page_table::PTEFlags::W | page_table::PTEFlags::X,
        ),
        None,
    );

    // 映射内核之后、帧分配器之前的内存 (堆区域, R|W)
    let frame_start_addr = frame_start.0 * PAGE_SIZE;
    if frame_start_addr > ekernel_addr_val {
        kernel_space.push(
            MapArea::new(
                ekernel_addr_val.into(),
                frame_start_addr.into(),
                MapType::Identical,
                page_table::PTEFlags::R | page_table::PTEFlags::W,
            ),
            None,
        );
    }

    // 映射帧分配器范围 (使用 2MB 大页 + 少量 4KB 页补齐非对齐头部)
    {
        let huge_size: usize = 2 * 1024 * 1024; // 2MB
        let huge_aligned = |addr: usize| (addr + huge_size - 1) / huge_size * huge_size;

        let alloc_start = frame_start_addr;
        let alloc_end = MEMORY_END;
        let huge_start = huge_aligned(alloc_start);

        // 非对齐头: 4KB 页映射
        if huge_start > alloc_start {
            let head_pages = (huge_start - alloc_start) / PAGE_SIZE;
            kernel_space.push(
                MapArea::new(
                    VirtAddr(alloc_start),
                    VirtAddr(huge_start),
                    MapType::Identical,
                    page_table::PTEFlags::R | page_table::PTEFlags::W,
                ),
                None,
            );
            println!(
                "[kernel] Mapped {} 4KB head pages [{:#x}, {:#x})",
                head_pages, alloc_start, huge_start
            );
        }

        // 对齐部分: 2MB 大页映射
        let mut huge_va = huge_start;
        let mut count = 0;
        while huge_va < alloc_end {
            let vpn = VirtAddr(huge_va).floor();
            let ppn = PhysPageNum(huge_va / PAGE_SIZE);
            kernel_space.page_table.map_huge(
                vpn, ppn,
                page_table::PTEFlags::R | page_table::PTEFlags::W,
            );
            huge_va += huge_size;
            count += 1;
        }
        println!("[kernel] Mapped {} x 2MB huge pages [{:#x}, {:#x})",
            count, huge_start, alloc_end);
    }

    // 映射 trampoline 页面到 TRAMPOLINE 虚拟地址
    kernel_space.push_trampoline(trampoline_frame, TRAMPOLINE);

    println!("[kernel] Kernel page table created");
}

/// 初始化 trampoline: 将 trap.S 代码复制到独立页帧并映射
/// 返回物理页号
pub fn init_trampoline() -> PhysPageNum {
    extern "C" {
        fn __alltraps();
        fn __trampoline_end();
    }

    let frame = frame_alloc().expect("trampoline: alloc frame failed");
    let src_start = __alltraps as *const () as usize;
    let src_end = __trampoline_end as *const () as usize;
    let len = src_end - src_start;

    assert!(len <= PAGE_SIZE, "Trampoline code too large: {} bytes", len);

    let pa: usize = frame.0 * PAGE_SIZE;
    unsafe {
        core::ptr::copy_nonoverlapping(
            src_start as *const u8,
            pa as *mut u8,
            len,
        );
    }

    println!(
        "[kernel] Trampoline: copied {} bytes from {:#x} to ppn {:?}",
        len, src_start, frame
    );

    frame
}

// 导出地址类型别名
pub use address::{PhysAddr, VirtAddr, PhysPageNum, VirtPageNum};

pub fn init() {
    init_kernel_memory();
    println!("[kernel] Memory system initialized");

    // 激活内核页表
    let kernel_satp = KERNEL_SPACE.exclusive_access().page_table.satp();
    unsafe {
        riscv::register::satp::write(kernel_satp);
        core::arch::asm!("sfence.vma", options(nomem, nostack));
    }
    println!("[kernel] Kernel page table activated, satp={:#x}", kernel_satp);

    // 更新 stvec 为 trampoline 地址
    crate::trap::set_trampoline_stvec(TRAMPOLINE);
}

pub fn remap_test() {
    // 简单的页表重映射测试：验证内核虚拟地址空间正常工作
    let kernel_space = KERNEL_SPACE.exclusive_access();
    if let Some(_) = kernel_space.translate(TRAMPOLINE.into()) {
        println!("[kernel] remap_test: trampoline accessible OK");
    }
}
