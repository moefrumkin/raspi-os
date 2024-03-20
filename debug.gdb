set verbose on
set disassemble-next-line on
set confirm off
add-symbol-file target/aarch64-unknown-none/debug/graph_os
target remote tcp::1234
set arch aarch64
layout regs
layout asm
handle SIGTRAP nostop noprint noignore
