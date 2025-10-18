.global _exception_vector

.macro call_handler handler source type
    msr daifset, 0b10 // Disable interrupts
    str lr, [sp, #-16]!
    bl push_frame
    mov     x0, \source
    mov     x1, \type
    mov     x2, sp
    bl       \handler
    bl pop_frame
    ldr lr, [sp], #16 
    msr daifclr, 0b10 // Enable Interrupts
    eret
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
    msr daifset, 0b10 // Disable interrupts
    str lr, [sp, #-16]!
    bl push_frame
    bl handle_synchronous_exception
    bl pop_frame
    ldr lr, [sp], #16
    msr daifclr, 0b10 // Enable Interrupts
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


push_frame: // TODO: push all registers
    sub sp, sp, 0x110 // TODO: check math
    stp x0, x1, [sp, 0x0]
    stp x2, x3, [sp, 0x10]
    stp x4, x5, [sp, 0x20]
    stp x6, x7, [sp, 0x30]
    stp x8, x9, [sp, 0x40]
    stp x10, x11, [sp, 0x50]
    stp x12, x13, [sp, 0x60]
    stp x14, x15, [sp, 0x70]
    stp x16, x17, [sp, 0x90]
    stp x18, x19, [sp, 0xa0]
    stp x20, x21, [sp, 0xb0]
    stp x22, x23, [sp, 0xc0]
    stp x24, x25, [sp, 0xd0]
    stp x26, x27, [sp, 0xe0]
    stp x28, x29, [sp, 0xf0]
    ret

pop_frame:
    ldp x0, x1, [sp, 0x0]
    ldp x2, x3, [sp, 0x10]
    ldp x4, x5, [sp, 0x20]
    ldp x6, x7, [sp, 0x30]
    ldp x8, x9, [sp, 0x40]
    ldp x10, x11, [sp, 0x50]
    ldp x12, x13, [sp, 0x60]
    ldp x14, x15, [sp, 0x70]
    ldp x16, x17, [sp, 0x90]
    ldp x18, x19, [sp, 0xa0]
    ldp x20, x21, [sp, 0xb0]
    ldp x22, x23, [sp, 0xc0]
    ldp x24, x25, [sp, 0xd0]
    ldp x26, x27, [sp, 0xe0]
    ldp x28, x29, [sp, 0xf0]
    add sp, sp, 0x110
    ret
