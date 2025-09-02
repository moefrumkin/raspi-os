ENTRY(_start)
SECTIONS
{
    . = 0x80000;
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
    
    TABLE_START = .;
    . += 0x1010;

    . = ALIGN(0x1000); /*TODO: is this a reasonable aligment*/
    HEAP_START = .;
    . = . + HEAP_SIZE;

    MMIO_START = 0x3F000000;

    TIMER_REGISTERS = MMIO_START + 0x3000;
    MAILBOX_REGISTERS = MMIO_START + 0xB880;
    GPIO_REGISTERS = MMIO_START + 0x200000;
    EMMC_REGISTERS = MMIO_START + 0x300000;
    MINI_UART_REGISTERS = MMIO_START + 0x215000;

   /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
HEAP_SIZE = 1048576;
bss_size = (bss_end - bss_start)>>3;
