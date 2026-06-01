TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os

$(KERNEL_ELF): src/*.rs src/*/*.rs src/*.S build_user
	cargo build --release

build_user:
	@make -C user build

run: $(KERNEL_ELF)
	qemu-system-riscv64 -machine virt -nographic -bios default -kernel $(KERNEL_ELF)

clean:
	cargo clean
	@make -C user clean
	rm -f target/$(TARGET)/$(MODE)/os.bin

.PHONY: run clean build_user
