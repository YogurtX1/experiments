use crate::mm::address::PhysPageNum;
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use lazy_static::lazy_static;

// 帧分配器追踪可用物理帧
pub struct FrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

type FrameAllocatorImpl = UPSafeCell<FrameAllocator>;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: FrameAllocatorImpl =
        unsafe { UPSafeCell::new(FrameAllocator::new()) };
}

impl FrameAllocator {
    pub fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }

    pub fn init(&mut self, start: PhysPageNum, end: PhysPageNum) {
        self.current = start.0;
        self.end = end.0;
    }

    pub fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(PhysPageNum(ppn))
        } else if self.current < self.end {
            let ppn = PhysPageNum(self.current);
            self.current += 1;
            Some(ppn)
        } else {
            None
        }
    }

    pub fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn_val = ppn.0;
        if ppn_val >= self.current || self.recycled.contains(&ppn_val) {
            panic!("Frame dealloc error: ppn = {:#x}", ppn_val);
        }
        self.recycled.push(ppn_val);
    }
}

pub fn init_frame_allocator(start: PhysPageNum, end: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().init(start, end);
}

pub fn frame_alloc() -> Option<PhysPageNum> {
    FRAME_ALLOCATOR.exclusive_access().alloc()
}

/// 分配一个 2MB 对齐的物理帧（用于中间级页表）。
///
/// Sv39 规范要求：非叶 PTE 只使用 PPN[2:1]（35 bits），
/// PPN[0]（bits[18:10]）必须为 0。这意味着中间级页表页
/// 的物理地址必须 2MB 对齐（PPN & 0x1FF == 0）。
///
/// 非对齐的帧会被跳过并暂存，后续由 frame_alloc() 回收
/// 用于数据页分配。
pub fn alloc_pte_frame() -> Option<PhysPageNum> {
    let mut allocator = FRAME_ALLOCATOR.exclusive_access();
    // 从 current 开始寻找 2MB 对齐的帧
    while allocator.current < allocator.end {
        let ppn = PhysPageNum(allocator.current);
        allocator.current += 1;
        if ppn.0 & 0x1FF == 0 {
            return Some(ppn);
        }
        // 非对齐帧回收供数据页使用
        allocator.recycled.push(ppn.0);
    }
    None
}

pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn)
}
