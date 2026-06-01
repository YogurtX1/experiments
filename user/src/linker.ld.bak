OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
  . = 0x80400000;
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
    start_bss = .;
    *(.bss .bss.*)
    *(.sbss .sbss.*)
    end_bss = .;
  }
  /* 显式包含这些段，这是解决链接错误的关键 */
  .shstrtab : { *(.shstrtab) }
  .strtab : { *(.strtab) }
  .symtab : { *(.symtab) }

  /DISCARD/ : {
    *(.eh_frame)
    *(.comment)
    *(.riscv.attributes)
    *(.note*)
  }
}
