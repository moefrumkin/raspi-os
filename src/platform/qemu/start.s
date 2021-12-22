.global _start
.extern STACK_PTR

.section ".text.boot"

_start:
    ldr x30, =_start
    mov sp, x30
    bl main