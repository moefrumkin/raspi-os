CC = clang

CFLAGS = -target aarch64-none-elf -nostdlib

BUILD_DIR = build

ASM_SOURCES := $(wildcard asm/*.s)
ELFS := $(patsubst asm/%.s, $(BUILD_DIR)/%.elf, $(ASM_SOURCES))

all: $(ELFS)

clean:
	rm -rf $(BUILD_DIR)

build/%.elf: asm/%.s | build_dir
	$(CC) $(CFLAGS) -o $@ $<

build_dir:
	mkdir -p $(BUILD_DIR)