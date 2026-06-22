use crate::config::TRAMPOLINE;
use crate::mm::address::PhysPageNum;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct TaskContext {
    pub ra: usize,
    pub sp: usize,
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
}

impl TaskContext {
    pub const fn zero_init() -> Self {
        Self {
            ra: 0, sp: 0,
            s0: 0, s1: 0, s2: 0, s3: 0, s4: 0, s5: 0,
            s6: 0, s7: 0, s8: 0, s9: 0, s10: 0, s11: 0,
        }
    }

    pub fn goto_trap_return(kstack_top: usize, trap_cx_ppn: PhysPageNum) -> Self {
        extern "C" {
            fn __alltraps();
            fn __goto_restore();
        }
        let offset = __goto_restore as *const () as usize - __alltraps as *const () as usize;
        let trampoline_ra = TRAMPOLINE + offset;
        let trap_cx_phys = trap_cx_ppn.0 * crate::config::PAGE_SIZE;

        #[cfg(feature = "debug")]
        println!("[kernel] goto_trap_return: ra={:#x}, sp={:#x}, s0(trap_cx_phys)={:#x}",
            trampoline_ra, kstack_top, trap_cx_phys);

        Self {
            ra: trampoline_ra,
            sp: kstack_top,
            s0: trap_cx_phys,
            s1: 0, s2: 0, s3: 0, s4: 0, s5: 0,
            s6: 0, s7: 0, s8: 0, s9: 0, s10: 0, s11: 0,
        }
    }
}
