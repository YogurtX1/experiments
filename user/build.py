import os

apps = os.listdir("src/bin")
apps.sort()
chapter = 3

# 第三章每个应用的基地址间隔 0x20000
base_address = 0x80400000
step = 0x20000

linker = "src/linker.ld"

for i, app in enumerate(apps):
    app = app[: app.find(".")]
    lines = []
    
    # 动态计算当前应用应该使用的基地址
    app_base = base_address + i * step
    print(f"[build.py] application {app} start with address {hex(app_base)}")
    
    # 生成临时的、符合当前应用地址的 linker.ld
    with open(linker, "w") as f:
        f.write(f"""OUTPUT_ARCH(riscv)
ENTRY(_start)

BASE_ADDRESS = {hex(app_base)};

SECTIONS
{{
    . = BASE_ADDRESS;
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
        *(.sbss .sbss.*)
        ebss = .;
    }}
    /DISCARD/ : {{
        *(.eh_frame)
    }}
}}""")

    # 编译当前应用
    os.system(f"cargo build --release --bin {app}")

print("\n--- All apps compiled successfully! Converting to binaries... ---")

# 统一转换为二进制文件
for app in apps:
    app = app[: app.find(".")]
    os.system(f"llvm-objcopy target/riscv64gc-unknown-none-elf/release/{app} --strip-all -O binary target/riscv64gc-unknown-none-elf/release/{app}.bin")
