.global _exception_vector

.macro call_handler handler source type
    msr daifset, 0b10 // Disable interrupts
    mov     x0, \source
    mov     x1, \type
    str lr, [sp, #-16]!
    bl push_frame
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


push_frame:
    sub sp, sp, #192
    stp x0, x1, [sp, #0]
    stp x2, x3, [sp, #16]
    stp x4, x5, [sp, #32]
    stp x6, x7, [sp, #48]
    stp x8, x9, [sp, #64]
    stp x10, x11, [sp, #80]
    stp x12, x13, [sp, #96]
    stp x14, x15, [sp, #112]
    stp x16, x17, [sp, #128]
    stp x18, x29, [sp, #144]
    // stp x30, xzr, [sp, #160]
    ret

pop_frame:
    ldp x0, x1, [sp, #0]
    ldp x2, x3, [sp, #16]
    ldp x4, x5, [sp, #32]
    ldp x6, x7, [sp, #48]
    ldp x8, x9, [sp, #64]
    ldp x10, x11, [sp, #80]
    ldp x12, x13, [sp, #96]
    ldp x14, x15, [sp, #112]
    ldp x16, x17, [sp, #128]
    ldp x18, x29, [sp, #144]
    // ldp x30, xzr, [sp, #160]
    add sp, sp, #192
    ret
