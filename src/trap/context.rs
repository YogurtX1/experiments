#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],   // 0 - 31 对应 32个通用寄存器 (偏移 0 到 248)
    pub sstatus: usize,  // 32 对应 sstatus 状态寄存器 (偏移 256)
    pub sepc: usize,     // 33 对应 sepc 程序计数器 (偏移 264)
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) { self.x[2] = sp; }
    
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        use riscv::register::sstatus::{self, Sstatus};
        let mut sstatus = sstatus::read();
        sstatus.set_spp(sstatus::SPP::User); // 确保 sret 后回到用户态
        
        let mut cx = Self {
            x: [0; 32],
            sstatus: sstatus.bits(),
            sepc: entry, // 🎯 这里的 entry 必须被 __restore 精准加载到 sepc
        };
        cx.set_sp(sp);
        cx
    }
}
