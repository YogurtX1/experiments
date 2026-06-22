use riscv::register::sstatus;

// TrapContext 大小: 41 * 8 = 328 字节
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
    pub trap_cx_ppn_pa: usize,// 40: TrapContext 内核物理地址 (恒等映射可访问)
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
        trap_cx_ppn_pa: usize,
    ) -> Self {
        let sstatus_val = {
            let mut s = sstatus::read();
            s.set_spp(sstatus::SPP::User);
            // ★ 关键: SPIE=1, SIE=0
            // csrw sstatus 时 SIE=0 → __restore 窗口期无中断
            // sret 原子性: SIE←SPIE=1, SPP←User, SPIE←1
            // RISC-V sstatus: bit5=SPIE, bit1=SIE
            let mut val = (s.bits() | (1 << 5)) & !(1 << 1);
            // ★ 显式设置 FS=Dirty (11): 确保用户态 FPU 可用
            //    FS=Initial(01) 在 sret 跨特权级时会被某些 QEMU 版本清零,
            //    导致用户态 FP 指令触发 IllegalInstruction. FS=Dirty 可被保留.
            val &= !(3 << 13);       // 清除 FS 字段 (bits 14-13)
            val |= (3 << 13);         // 设置 FS=Dirty (11)
            val
        };

        let mut cx = Self {
            x: [0; 32],
            sstatus: sstatus_val,
            sepc: entry,
            kernel_satp,
            kernel_sp,
            trap_handler,
            user_satp,
            trap_cx_user,
            trap_cx_kernel,
            trap_cx_ppn_pa,
        };
        cx.set_sp(sp);
        cx
    }
}
