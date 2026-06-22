use crate::config::KERNEL_HEAP_SIZE;
use buddy_system_allocator::LockedHeap;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init_heap() {
    let heap_ptr = unsafe { core::ptr::addr_of_mut!(HEAP_SPACE) };
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(heap_ptr as usize, KERNEL_HEAP_SIZE);
    }
    println!(
        "[kernel] Heap initialized: {:#x} - {:#x}",
        heap_ptr as usize,
        heap_ptr as usize + KERNEL_HEAP_SIZE
    );
}
