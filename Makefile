TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
OBJCOPY := rust-objcopy --binary-architecture=riscv64

APP_BASE_ADDRESS := 2151677952  # 0x80400000
APP_SIZE_LIMIT := 131072       # 0x20000

$(KERNEL_ELF): build_user_apps
	@echo "[kernel] Building kernel..."
	cargo build --release

build_user_apps:
	@echo "[user] Clear old targets..."
	@cd user && cargo clean
	@echo "[user] Dynamically linking and building user applications..."
	@mkdir -p user/target/$(TARGET)/$(MODE)/
	@idx=0; \
	for app_path in user/src/bin/*.rs; do \
		app_name=$$(basename $$app_path .rs); \
		app_addr=$$(($(APP_BASE_ADDRESS) + $$idx * $(APP_SIZE_LIMIT))); \
		app_hex=$$(printf "0x%x" $$app_addr); \
		echo "[user] Building $$app_name linked at $$app_hex"; \
		\
		# 使用圆括号 ( ) 包裹，确保 cd user 只在子进程生效，不会污染下一次循环 \
		(cd user && APP_NAME=$$app_name BASE_ADDRESS=$$app_hex cargo build --bin $$app_name --release); \
		\
		if [ -f user/target/$(TARGET)/$(MODE)/$$app_name ]; then \
			$(OBJCOPY) --strip-all -O binary \
				user/target/$(TARGET)/$(MODE)/$$app_name \
				user/target/$(TARGET)/$(MODE)/$$app_name.bin; \
		fi; \
		idx=$$((idx + 1)); \
	done
	@echo "[user] All applications are perfectly linked and ready!"

run: $(KERNEL_ELF)
	qemu-system-riscv64 -machine virt -nographic -bios default -kernel $(KERNEL_ELF)

clean:
	cargo clean
	cd user && cargo clean
	rm -f target/$(TARGET)/$(MODE)/os.bin

.PHONY: run clean build_user_apps
