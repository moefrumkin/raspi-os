ENTRY(_start)
SECTIONS
{
    . = 0x0;
    .text.boot : { *(.text.boot) }
    .text : { *(.text) }
    .data : { *(.data) }
    .rodata : { *(.rodata) }
    
    BSS_START = .;
    .bss : { *(.bss) }
    BSS_END = .;

    . = ALIGN(8);
    . = . + 0x4000;
    LD_STACK_PTR = .;
}