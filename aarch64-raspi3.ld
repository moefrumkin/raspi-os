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

    GPIO_START = 0x3F200000;

   /DISCARD/ : { *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) }
}
HEAP_SIZE = 1048576;
bss_size = (bss_end - bss_start)>>3;
