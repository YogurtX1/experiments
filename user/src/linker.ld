
OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
    . = 0x80440000;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    .rodata : {
        *(.rodata .rodata.*)
    }
    .data : {
        *(.data .data.*)
    }
    .bss : {
        sbss = .;
        *(.bss .bss.*)
        ebss = .;
    }
    /DISCARD/ : {
        *(.eh_frame)
    }
}
