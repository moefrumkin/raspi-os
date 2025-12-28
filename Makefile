PLATFORM ?= raspi3
ARCH = aarch64-unknown-none

#TODO: do we need -g flag?
BUILD_CMD = cargo rustc --features=$(PLATFORM) --target=$(ARCH) -- -g -C link-arg=-Taarch64-raspi3.ld

KERNEL_ELF = target/$(ARCH)/debug/graph_os

QEMU_ARCH = qemu-system-aarch64

ifeq ($(PLATFORM), raspi3)
	MACHINE = raspi3b # Is this correct?
	CPU = cortex-a53
	CORES = 4
else
	$(error unsupported platform $(PLATFORM))
endif

QEMU_CMD = $(QEMU_ARCH) \
	-M $(MACHINE) \
	-kernel $(KERNEL_ELF) \
	-serial null \
	-serial mon:stdio \
	-drive file=raspi.img,if=sd,format=raw

OBJDUMP = objdump
OBJDUMP_CMD = $(OBJDUMP) -C --disassemble-all $(KERNEL_ELF)

GDB = gdb
GDB_SCRIPT = debug.gdb
GDB_CMD = $(GDB) -x $(GDB_SCRIPT)

.PHONY: all dump

all: build doc-noopen

qemu:
	$(QEMU_CMD) -S -s

qemu-nogui:
	$(QEMU_CMD) -S -s -nographic

build:
	$(BUILD_CMD)

image:
	llvm-objcopy --output-target=aarch64-unknown-none --strip-all -O binary target/aarch64-unknown-none/debug/graph_os kernel8.img

run: $(KERNEL_ELF)
	$(QEMU_CMD)

run-nogui: $(KERNEL_ELF)
	$(QEMU_CMD) -nographic

dump:
	$(OBJDUMP_CMD)

gdb: 
	$(GDB_CMD)

clean:
	cargo clean
	rm -f kernel8.img

doc:
	cargo doc --features=$(PLATFORM) --open

test:
	cargo test --features=$(PLATFORM) -- --nocapture

DISK_IMG = raspi.img
IMG_MOUNT_PT = /Volumes/BOOT

attach-fs:
	hdiutil attach -section 8193 $(DISK_IMG)

detach-fs:
	hdiutil detach $(IMG_MOUNT_PT)

copy-programs:
	cp programs/*.elf $(IMG_MOUNT_PT)/users/moe