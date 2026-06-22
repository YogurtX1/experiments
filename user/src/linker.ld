OUTPUT_ARCH(riscv64)
ENTRY(_start)

SECTIONS {
    . = 0x80400000;
    .text : {
        *(.text.start)
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
    . = ALIGN(0x1000);
    PROVIDE(__global_pointer$ = . + 0x800);
    .bss : {
        PROVIDE(sbss = .);
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        PROVIDE(ebss = .);
    }
    _stack_start = 0x80430000;
    /DISCARD/ : {
        *(.eh_frame)
    }
}
