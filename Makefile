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
	-machine $(MACHINE) \
	-m 1024M -cpu $(CPU) \
	-smp $(CORES) \
	-kernel $(KERNEL_ELF) \
	-d int,mmu,guest_errors,page \
	-nographic \
	-serial null \
	-serial mon:stdio

OBJDUMP = aarch64-none-elf-objdump
OBJDUMP_CMD = $(OBJDUMP) --disassemble-all $(KERNEL_ELF)

GDB = rust-gdb
#gdb-multiarch
GDB_SCRIPT = debug.gdb
GDB_CMD = $(GDB) -x $(GDB_SCRIPT)

.PHONY: all

all: build doc-noopen

qemu:
	$(QEMU_CMD) -S -s

build:
	$(BUILD_CMD)

image:
	llvm-objcopy --output-target=aarch64-unknown-none --strip-all -O binary target/aarch64-unknown-none/debug/graph_os kernel8.img

run: $(KERNEL_ELF)
	$(QEMU_CMD)

dump:
	$(OBJDUMP_CMD)

nm:
	nm $(KERNEL_ELF)

readelf:
	aarch64-none-elf-readelf --header $(KERNEL_ELF)

gdb:
	$(GDB_CMD)

clean:
	cargo clean
	rm -f *.img

doc:
	cargo doc --features=$(PLATFORM) --open

doc-noopen:
	cargo doc --features=$(PLATFORM)

test:
	cargo test --features=$(PLATFORM) -- --nocapture
