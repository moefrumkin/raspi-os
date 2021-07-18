PLATFORM ?= raspi3

ARCH = aarch64-unknown-none
BUILD_CMD = cargo xbuild --features=$(PLATFORM) --target=$(ARCH).json --release

KERNEL_ELF = target/$(ARCH)/release/graph_os

QEMU = qemu-system-aarch64

ifeq ($(PLATFORM), raspi3)
	MACHINE = raspi3
	CORES = 4
else ifeq ($(PLATFORM), qemu)
	MACHINE = virt
	CORES = 1
else
	$(error unsupported platform $(PLATFORM))
endif

CPU = cortex-a53
QEMU_CMD = $(QEMU) -machine $(MACHINE) -m 1024M -cpu $(CPU) -smp $(CORES) -nographic -kernel $(KERNEL_ELF)

OBJDUMP = objdump
OBJDUMP_CMD = $(OBJDUMP) --disassemble-all $(KERNEL_ELF)

GDB = gdb-multiarch
GDB_SCRIPT = release.gdb
GDB_CMD = $(GDB) -x $(GDB_SCRIPT)

.PHONY: all

all: build test doc

qemu:
	make PLATFORM=qemu
	

build:
	$(BUILD_CMD)

image:
	objcopy --strip-all -O binary $(KERNEL_ELF) kernel8.img

run:
	$(QEMU_CMD)

dump:
	$(OBJDUMP_CMD)

debug:
	$(QEMU_CMD)	-S -s

gdb:
	$(GDB_CMD)

clean:
	cargo clean

doc:
	cargo doc --features=$(PLATFORM) --open

test:
	cargo test --features=$(PLATFORM)