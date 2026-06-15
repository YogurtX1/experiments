use crate::config::{APP_BASE_ADDRESS, APP_SIZE_LIMIT};

pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe { (_num_app as *const () as usize as *const usize).read_volatile() }
}

pub fn load_apps() {
    extern "C" {
        fn _num_app();
    }
    let num_app = get_num_app();
    let app_start = unsafe {
        core::slice::from_raw_parts(
            (_num_app as *const () as usize as *const usize).add(1),
            num_app + 1,
        )
    };

    // 清空之前的指令缓存（i-cache）
    unsafe {
        core::arch::asm!("fence.i");
    }

    for i in 0..num_app {
        let base_i = i;
        let src = app_start[base_i];
        let dst = APP_BASE_ADDRESS + base_i * APP_SIZE_LIMIT;
        let len = app_start[base_i + 1] - src;

        unsafe {
            core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);
        }
    }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    extern "C" {
        fn _num_app();
    }
    let num_app = get_num_app();
    let app_start = unsafe {
        core::slice::from_raw_parts(
            (_num_app as *const () as usize as *const usize).add(1),
            num_app + 1,
        )
    };
    if app_id >= num_app {
        panic!("app_id out of range!");
    }
    let src = app_start[app_id];
    let len = app_start[app_id + 1] - src;
    unsafe { core::slice::from_raw_parts(src as *const u8, len) }
}
