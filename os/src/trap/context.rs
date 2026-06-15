use riscv::register::sstatus;

// TrapContext 大小: 40 * 8 = 320 字节
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],       // 0-31: 通用寄存器
    pub sstatus: usize,       // 32: sstatus
    pub sepc: usize,          // 33: sepc
    pub kernel_satp: usize,   // 34: 内核地址空间 satp
    pub kernel_sp: usize,     // 35: 内核栈指针
    pub trap_handler: usize,  // 36: trap_handler 函数地址
    pub user_satp: usize,     // 37: 用户地址空间 satp
    pub trap_cx_user: usize,  // 38: 用户态 TrapContext 虚拟地址
    pub trap_cx_kernel: usize,// 39: 内核态 TrapContext 地址 (设置 sscratch)
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
        user_satp: usize,
        trap_cx_user: usize,
        trap_cx_kernel: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(sstatus::SPP::User);

        let mut cx = Self {
            x: [0; 32],
            sstatus: sstatus.bits(),
            sepc: entry,
            kernel_satp,
            kernel_sp,
            trap_handler,
            user_satp,
            trap_cx_user,
            trap_cx_kernel,
        };
        cx.set_sp(sp);
        cx
    }
}
