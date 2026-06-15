# entry.S: 用户程序入口和系统调用蹦床
# 所有关键路径均为纯汇编，避免 Rust 编译器干扰寄存器

# ─── 入口 ──────────────────────────────────────────
# ★ 必须放在最前面: _start 在专用段 .text.start, 确保它是镜像第一个符号
.section .text.start

.globl _start
.type _start, @function
.align 4
_start:
    # 1. 设置全局指针 gp
    .option push
    .option norelax
    la gp, __global_pointer$
    .option pop

    # 2. 设置栈指针
    la sp, _stack_start

    # 3. 清零 BSS
    call clear_bss_asm

    # 4. 调用 Rust main() → 返回值在 a0
    call main

    # 5. sys_exit(exit_code)
    #    a1 = arg0 = exit_code (main 的返回值)
    #    a0 = id = 93
    mv a1, a0
    li a0, 93
    li a2, 0
    li a3, 0
    j _user_syscall
.size _start, . - _start

.section .text.entry

# ─── 系统调用蹦床 ────────────────────────────────────
# 调用约定: C ABI → ecall ABI
#   C ABI:  a0=id, a1=arg0, a2=arg1, a3=arg2
#   ecall:  a7=id, a0=arg0, a1=arg1, a2=arg2 → 内核 trap
.globl _user_syscall
.type _user_syscall, @function
.align 4
_user_syscall:
    .option push
    .option norvc
    mv a7, a0          # a7 = id
    mv a0, a1          # a0 = arg0
    mv a1, a2          # a1 = arg1
    mv a2, a3          # a2 = arg2
    ecall
    ret
    .option pop
.size _user_syscall, . - _user_syscall

# ─── BSS 清零 ───────────────────────────────────────
.type clear_bss_asm, @function
.align 4
clear_bss_asm:
    la t0, sbss
    la t1, ebss
1:  bge t0, t1, 2f
    sd zero, 0(t0)
    addi t0, t0, 8
    j 1b
2:  ret
.size clear_bss_asm, . - clear_bss_asm
