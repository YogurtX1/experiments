    .section .text
    .globl __switch
__switch:
    # a0: 当前任务 task_cx_ptr 的二级指针地址
    # a1: 下一个任务 task_cx_ptr 的二级指针地址

    # 1. 解引用 a0：拿到当前任务 TaskContext 的真实一级物理首地址
    ld t0, 0(a0)
    
    # 如果 t0 是 0（比如第一次运行 _unused），跳过保存逻辑
    beqz t0, .Lskip_save
    
    # 保存当前 CPU 状态到旧的 TaskContext 结构体中
    sd ra, 0(t0)
    sd sp, 8(t0)
    sd s0, 16(t0)
    sd s1, 24(t0)
    sd s2, 32(t0)
    sd s3, 40(t0)
    sd s4, 48(t0)
    sd s5, 56(t0)
    sd s6, 64(t0)
    sd s7, 72(t0)
    sd s8, 80(t0)
    sd s9, 88(t0)
    sd s10, 96(t0)
    sd s11, 104(t0)

.Lskip_save:
    # 2. 解引用 a1：从二级指针中，加载出下一个任务 TaskContext 的真实物理首地址
    ld t1, 0(a1)

    # 3. 🌟 从完美的 TaskContext 中精准恢复 CPU 寄存器（严格对应字节偏移！）
    ld ra, 0(t1)    # 恢复 ra：如果是第二次切回，这里拿到的就是它之前在内核里的返回地址！
    ld sp, 8(t1)    # 恢复 sp：恢复正确的内核栈顶！
    ld s0, 16(t1)
    ld s1, 24(t1)
    ld s2, 32(t1)
    ld s3, 40(t1)
    ld s4, 48(t1)
    ld s5, 56(t1)
    ld s6, 64(t1)
    ld s7, 72(t1)
    ld s8, 80(t1)
    ld s9, 88(t1)
    ld s10, 96(t1)
    ld s11, 104(t1)

    # 4. 完美返回！
    ret
