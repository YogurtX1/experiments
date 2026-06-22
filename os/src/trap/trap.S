    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .globl __goto_restore
    .globl __trampoline_end
    .align 6

__alltraps:
    csrrw sp, sscratch, sp
    # sp = TrapContext 用户虚拟地址(或物理地址), sscratch = 用户 sp(或内核 sp)
    # ★ 检测 trap 来自内核态还是用户态
    csrr t0, sstatus
    srli t0, t0, 8        # 提取 SPP 位 (bit 8)
    andi t0, t0, 1
    bnez t0, __from_kernel # SPP=1 → 来自内核态
    # === 用户态 trap: 正常流程 ===
    # ★ 先读取内核关键字段 (slot 34-36, 40), 这些是后续切换所需的元数据
    ld t0, 34*8(sp)     # kernel_satp
    ld t3, 35*8(sp)     # kernel_sp
    ld t2, 36*8(sp)     # trap_handler
    ld t5, 40*8(sp)     # trap_cx_ppn_pa (内核物理地址，恒等映射可访问)
    # 现在安全地存储用户寄存器
    sd x31, 31*8(sp)
    sd x1, 1*8(sp)
    sd x3, 3*8(sp)
    .irp n,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30
    sd x\n, \n*8(sp)
    .endr
    mv x31, sp
    csrr t4, sscratch       # t4 = user_sp
    sd t4, 2*8(x31)          # 存储 user_sp
    sd x4, 4*8(x31)          # 存储 tp
    csrr t4, sstatus
    sd t4, 32*8(x31)         # 存储 sstatus
    csrr t4, sepc
    sd t4, 33*8(x31)         # 存储 sepc
    # fence: 确保所有 sd 对后续操作可见
    fence iorw, iorw
    # 切换到内核地址空间
    csrw satp, t0
    sfence.vma
    mv sp, t3                # sp = kernel_sp
    mv a0, t5                # a0 = TrapContext 内核物理地址
    jalr ra, t2              # jump to trap_handler

__from_kernel:
    # === 内核态 trap: 不保存用户寄存器 ===
    # sp = TrapContext 物理地址 (sscratch 被预先设置), sscratch = 内核 sp
    # 直接从 TrapContext 读取内核环境信息
    ld t0, 34*8(sp)     # kernel_satp
    ld t3, 35*8(sp)     # kernel_sp
    ld t2, 36*8(sp)     # trap_handler
    ld t5, 40*8(sp)     # trap_cx_ppn_pa
    # 切换到内核地址空间（已经是，但确保一致性）
    csrw satp, t0
    sfence.vma
    mv sp, t3            # sp = kernel_sp
    mv a0, t5            # a0 = TrapContext 物理地址
    jalr ra, t2          # jump to trap_handler

__restore:
    # a0 = &TrapContext (内核物理地址，通过恒等映射访问)
    ld t0, 37*8(a0)     # user_satp
    ld t1, 38*8(a0)     # trap_cx_user (用户态地址)
    ld t2, 39*8(a0)     # trap_cx_kernel (用于 sscratch)
    ld t3, 32*8(a0)     # sstatus
    ld t4, 33*8(a0)     # sepc
    csrw satp, t0
    sfence.vma
    csrw sscratch, t2   # 为下次陷阱设置 sscratch
    mv sp, t1            # sp = 用户态 TrapContext 地址
    csrw sepc, t4
    # 恢复通用寄存器
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .irp n,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31
    ld x\n, \n*8(sp)
    .endr
    ld x4, 4*8(sp)
    ld x2, 2*8(sp)
    # ★ 关键: sstatus 必须在 sret 之前紧邻设置,
    #    使 S-mode 中断启用与用户态切换原子化, 避免 timer 在窗口期触发损坏上下文
    csrw sstatus, t3
    sret

__goto_restore:
    # s0 = TrapContext 内核物理地址 (由 TaskContext.s0 传递)
    mv a0, s0
    j __restore

__trampoline_end:
