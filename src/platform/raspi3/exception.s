.global _exception_vector

.macro call_handler handler source type
    mov     x0, \source
    mov     x1, \type
    b       \handler
.endm   

.align 11
_exception_vector:
.org 0x0
    bl handle_synchronous_exception
    eret
.org 0x80
    call_handler handle_exception 0 1
.org 0x100
    call_handler handle_exception 0 2
.org 0x180
    call_handler handle_exception 0 3

.org 0x200
    str lr, [sp, #-16]!
    bl handle_synchronous_exception
    ldr lr, [sp], #16
    eret
.org 0x280
    call_handler handle_exception 1 1
.org 0x300
    call_handler handle_exception 1 2
.org 0x380
    call_handler handle_exception 1 3

.org 0x400
    call_handler handle_exception 2 0
.org 0x480
    call_handler handle_exception 2 1
.org 0x500
    call_handler handle_exception 2 2
.org 0x580
    call_handler handle_exception 2 3

.org 0x600
    call_handler handle_exception 3 0
.org 0x680
    call_handler handle_exception 3 1
.org 0x700
    call_handler handle_exception 3 2
.org 0x780
    call_handler handle_exception 3 3