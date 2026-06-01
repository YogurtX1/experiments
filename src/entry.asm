    .section .text.entry
    .globl _start
_start:
    la sp, boot_stack_top
    call rust_main

    .section .bss.stack
    .globl boot_stack_low
boot_stack_low:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
