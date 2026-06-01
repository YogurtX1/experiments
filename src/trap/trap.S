.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro RESTORE_GP n
    ld x\n, \n*8(sp)
.endm

    .section .text
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    csrrw sp, sscratch, sp
    # 现在 sp 指向内核栈，sscratch 指向用户栈
    # 分配 TrapContext 的空间：34 * 8 = 272 字节
    addi sp, sp, -272

    # 保存 32 个通用寄存器
    .set n, 1
    .rept 31
        SAVE_GP %n
        .set n, n+1
    .endr

    # 特殊保存用户栈指针（从 sscratch 读出）
    csrr t0, sscratch
    sd t0, 2*8(sp)

    # 保存 sstatus 和 sepc
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)

    # 准备跳转到高级语言的 trap_handler
    mv a0, sp
    call trap_handler
    mv sp, a0              # <-- 改1: 在这里同步 sp，确保 sp 指向 TrapContext

__restore:
    # 改2: 删除了原来的 mv sp, a0 — sp 已由 __alltraps 末尾或 __switch 保证正确

    # 恢复 sstatus 和 sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1

    # 恢复除 sp(x2) 之外的通用寄存器
    ld x1, 1*8(sp)
    .set n, 3
    .rept 29
        RESTORE_GP %n
        .set n, n+1
    .endr

    # 恢复用户栈指针到 sscratch 备份
    ld t0, 2*8(sp)
    csrw sscratch, t0

    # 真正恢复内核栈到没有分配 TrapContext 之前的状态
    addi sp, sp, 272
    # 倒手：把 sp 换回用户栈，内核栈切回 sscratch
    csrrw sp, sscratch, sp
    sret
