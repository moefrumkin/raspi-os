set verbose on
set disassemble-next-line on
set confirm off
set print pretty on
source gdb-scripts/ll_alloc.py
add-symbol-file target/aarch64-unknown-none/debug/graph_os
target remote tcp::1234
set arch aarch64
layout regs
layout asm
handle SIGTRAP nostop noprint noignore
