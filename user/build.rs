use std::fs::File;
use std::io::Write;

fn main() {
    let base_address = std::env::var("BASE_ADDRESS").unwrap_or_else(|_| "0x80400000".to_string());
    
    let mut f = File::create("src/linker.ld").unwrap();
    write!(f, r#"
OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{{
    . = {};
    .text : {{
        *(.text.entry)
        *(.text .text.*)
    }}
    .rodata : {{
        *(.rodata .rodata.*)
    }}
    .data : {{
        *(.data .data.*)
    }}
    .bss : {{
        sbss = .;
        *(.bss .bss.*)
        ebss = .;
    }}
    /DISCARD/ : {{
        *(.eh_frame)
    }}
}}
"#, base_address).unwrap();
}
