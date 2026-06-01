#!/bin/bash
set -e

# 确保在用户态目录下
cd /os/user

# 清理历史包袱
rm -rf target/riscv64gc-unknown-none-elf/release/*.bin

# 1. 编译 00write_a (基地址: 0x80400000)
sed -i 's/BASE_ADDRESS = .*/BASE_ADDRESS = 0x80400000;/' src/linker.ld
cargo build --release --bin 00write_a
llvm-objcopy target/riscv64gc-unknown-none-elf/release/00write_a --strip-all -O binary target/riscv64gc-unknown-none-elf/release/00write_a.bin

# 2. 编译 01write_b (基地址: 0x80420000)
sed -i 's/BASE_ADDRESS = .*/BASE_ADDRESS = 0x80420000;/' src/linker.ld
cargo build --release --bin 01write_b
llvm-objcopy target/riscv64gc-unknown-none-elf/release/01write_b --strip-all -O binary target/riscv64gc-unknown-none-elf/release/01write_b.bin

# 3. 编译 02write_c (基地址: 0x80440000)
sed -i 's/BASE_ADDRESS = .*/BASE_ADDRESS = 0x80440000;/' src/linker.ld
cargo build --release --bin 02write_c
llvm-objcopy target/riscv64gc-unknown-none-elf/release/02write_c --strip-all -O binary target/riscv64gc-unknown-none-elf/release/02write_c.bin

# 恢复默认
sed -i 's/BASE_ADDRESS = .*/BASE_ADDRESS = 0x80400000;/' src/linker.ld
echo "=== 所有应用地址完美对齐并转换为二进制！ ==="
