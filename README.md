# raspi-os
Raspi OS is a bare metal operating system being developed for the raspberry pi

# Installation
In order to compile and run graph_os, a few different tools are needed:
- [cargo-xbuild](https://docs.rs/crate/cargo-xbuild): provides cross compilation support
- [The GNU Cortex-A toolchain](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-a/downloads): provides tools for inspecting and disassembling arm assembly
- [qemu](https://www.qemu.org/): provides an emulated environment for development
- gdb-multiarch: provides debugging capabilities for qemu
