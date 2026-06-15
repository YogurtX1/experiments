OUTPUT_ARCH(riscv)
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
    . = ALIGN(CONSTANT(COMMONPAGESIZE));
    __global_pointer$ = . + 0x800;
    .bss : {
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        ebss = .;
    }
    /DISCARD/ : {
        *(.eh_frame)
    }
}

_stack_start = 0x80430000;
