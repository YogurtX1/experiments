OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
    . = 0x80200000;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    .bss : {
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }
    .stack : {
        *(.bss.stack)
    }
    /DISCARD/ : {
        *(.eh_frame)
        *(.comment)
        *(.riscv.attributes)
        *(.note*)
        /* 不要丢弃 .shstrtab 等 */
    }
}
