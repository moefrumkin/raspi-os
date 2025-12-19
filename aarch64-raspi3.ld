ENTRY(_start)
SECTIONS
{
    . = 0x80000;
    VM_START = 0xffff000000000000;
    .text : { KEEP( *(.text.boot)) *(.text .text.* .gnu.linkonce.t*) }
    .rodata : { *(.rodata .rodata.* .gnu.linkonce.r*) }
    PROVIDE(_data = .);
    .data : { *(.data .data.* .gnu.linkonce.d*) }
    .bss (NOLOAD) : {
        . = ALIGN(16);
        bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        bss_end = .;
    }

    . = ALIGN(0x1000);

    USER_TABLE_START = .;
    
    . = . + 0x10000;

    KERNEL_TABLE_START = .;

    . = . + 0x10000;

    . = ALIGN(0x1000); /*TODO: is this a reasonable aligment*/
    HEAP_START = .;
    . = . + HEAP_SIZE;

    . = ALIGN(0x10000);
    PAGE_SECTION_START = .;
    . = . + PAGE_SECTION_SIZE;

    MMIO_START = 0x3F000000;
    VIRTUAL_MMIO_START = VM_START + MMIO_START;

    TIMER_REGISTERS = MMIO_START + 0x3000;
    INTERRUPT_REGISTERS = MMIO_START + 0xB200;
    MAILBOX_REGISTERS = MMIO_START + 0xB880;
    GPIO_REGISTERS = MMIO_START + 0x200000;
    EMMC_REGISTERS = MMIO_START + 0x300000;
    MINI_UART_REGISTERS = MMIO_START + 0x215000;

   /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
HEAP_SIZE = 1048576;
PAGE_SECTION_SIZE = 6553600;
bss_size = (bss_end - bss_start)>>3;
