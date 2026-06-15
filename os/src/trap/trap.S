    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .globl __goto_restore
    .globl __trampoline_end
    .align 6

__alltraps:
    csrrw sp, sscratch, sp
    # sp = TrapContext 用户虚拟地址, sscratch = 用户 sp
    # ★ 先读取内核关键字段 (slot 34-36), 这些是后续切换所需的元数据
    #    必须在存储 user 寄存器前读取, 因为 user sp/x31 等值可能与
    #    这些字段的 VA 落在同一 cache line, store 可能使后续 load 读到 stale 数据
    ld t0, 34*8(sp)     # kernel_satp (立即加载到寄存器)
    ld t3, 35*8(sp)     # kernel_sp   (立即加载到寄存器)  
    ld t2, 36*8(sp)     # trap_handler (立即加载到寄存器)
    # 现在安全地存储用户寄存器 —— 覆盖 x1/x3/x5-x31/sstatus/sepc 等
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
    # ★ fence: 确保所有 sd 对后续操作可见
    fence iorw, iorw
    # 切换到内核地址空间
    csrw satp, t0            # t0 = kernel_satp (之前已加载)
    sfence.vma
    mv sp, t3                # sp = kernel_sp (之前已加载)
    # a0 = &TrapContext (物理地址 = sp - PAGE_SIZE, TrapContext 页对齐)
    addi a0, sp, -2048
    addi a0, a0, -2048
    jalr ra, t2              # jump to trap_handler

__restore:
    # a0 = &TrapContext (内核物理地址，通过恒等映射访问)
    # ⚠️ 关键: sstatus/sepc 必须在 satp 切换前从内核映射加载，
    #    否则通过用户页表映射读取可能因缓存/TLB 读到 stale 数据
    ld t0, 37*8(a0)     # user_satp
    ld t1, 38*8(a0)     # trap_cx_user (用户态地址)
    ld t2, 39*8(a0)     # trap_cx_kernel (用于 sscratch)
    ld t3, 32*8(a0)     # sstatus (内核映射读取)
    ld t4, 33*8(a0)     # sepc    (内核映射读取)
    csrw satp, t0
    sfence.vma
    csrw sscratch, t2   # 为下次陷阱设置 sscratch
    mv sp, t1            # sp = 用户态 TrapContext 地址
    csrw sstatus, t3
    csrw sepc, t4
    # 恢复通用寄存器
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .irp n,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31
    ld x\n, \n*8(sp)
    .endr
    ld x4, 4*8(sp)
    ld x2, 2*8(sp)
    sret

__goto_restore:
    # sp = 内核栈顶, TrapContext = sp - PAGE_SIZE (页对齐)
    # a0 = &TrapContext, 由 __restore 读取所有字段
    addi a0, sp, -2048
    addi a0, a0, -2048    # a0 = sp - PAGE_SIZE
    j __restore

__trampoline_end:
