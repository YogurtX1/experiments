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

pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn)
}
