OUTPUT_ARCH(riscv64)
ENTRY(_start)

BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;
    .text : {
        stext = .;
        *(.text.entry)
        *(.text .text.*)
        etext = .;
    }
    .rodata : {
        srodata = .;
        *(.rodata .rodata.*)
        erodata = .;
    }
    .data : {
        sdata = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        edata = .;
    }
    .bss : {
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        ebss = .;
    }
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}
