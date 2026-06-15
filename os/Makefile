MAKEFLAGS += -rR
.DEFAULT_GOAL := build

# 构建用户程序
user:
	cd ../user && cargo build --release

# 构建内核
kernel: user
	cargo build --release

# 构建 (默认)
build: kernel

# 运行
run: build
	qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios default \
		-kernel target/riscv64gc-unknown-none-elf/release/os

# 调试模式运行
debug: build
	qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios default \
		-kernel target/riscv64gc-unknown-none-elf/release/os \
		-s -S

# 清理
clean:
	cargo clean
	cd ../user && cargo clean

.PHONY: user kernel build run debug clean
