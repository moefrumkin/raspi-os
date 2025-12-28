CC = clang

CFLAGS = -target aarch64-none-elf -nostdlib

SOURCES := $(wildcard *.s)
ELFS := $(patsubst %.s, %.elf, $(SOURCES))

all: $(ELFS)

clean:
	rm $(ELFS)

%.elf: %.s
	$(CC) $(CFLAGS) -o $@ $<