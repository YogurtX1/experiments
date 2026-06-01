use crate::config::*;
use crate::task::TaskContext;

extern "C" {
    fn _num_app();
}

pub fn get_num_app() -> usize {
    unsafe { (_num_app as *const usize).read_volatile() }
}

pub fn init_app_cx(app_id: usize) -> TaskContext {
    let app_entry = 0x80400000 + app_id * 0x20000;
    
    unsafe {
        static mut KERNEL_STACK: [[u8; KERNEL_STACK_SIZE]; MAX_APP_NUM] = [[0; KERNEL_STACK_SIZE]; MAX_APP_NUM];
        static mut USER_STACK: [[u8; USER_STACK_SIZE]; MAX_APP_NUM] = [[0; USER_STACK_SIZE]; MAX_APP_NUM];
        
        let kstack_top = KERNEL_STACK[app_id].as_ptr() as usize + KERNEL_STACK_SIZE;
        let ustack_top = USER_STACK[app_id].as_ptr() as usize + USER_STACK_SIZE;

        // 🎯 修复警告：去掉了这里的 mut 关键字
        let trap_cx = crate::trap::TrapContext::app_init_context(app_entry, ustack_top);
        
        let trap_cx_ptr = (kstack_top - core::mem::size_of::<crate::trap::TrapContext>()) as *mut crate::trap::TrapContext;
        trap_cx_ptr.write_volatile(trap_cx);

        TaskContext::goto_trap_return(trap_cx_ptr as usize)
    }
}

pub fn load_apps() {
    unsafe {
        let num_app_ptr = _num_app as *const usize;
        let num_app = num_app_ptr.read_volatile();
        let app_start_ptr = num_app_ptr.add(1);

        for i in 0..num_app {
            let base_addr = 0x80400000 + i * 0x20000;
            let start = app_start_ptr.add(i).read_volatile();
            let end = app_start_ptr.add(i + 1).read_volatile();
            let len = end - start;

            println!("[kernel] Loading app_{} into physical address {:#x} (size: {} B)", i, base_addr, len);

            let dst = core::slice::from_raw_parts_mut(base_addr as *mut u8, len);
            let src = core::slice::from_raw_parts(start as *const u8, len);
            dst.copy_from_slice(src);
        }
        core::arch::asm!("fence.i");
    }
}
