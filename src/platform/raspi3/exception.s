.global _exception_vector

.macro call_handler handler source type
    msr daifset, 0b1111 // Disable interrupts
    str lr, [sp, #-16]! // Note: lr is x30 which is overwritten by bl
    bl push_frame
    mov     x0, \source
    mov     x1, \type
    mov     x2, sp
    bl       \handler
    bl pop_frame
    ldr lr, [sp], #16 
    //msr daifclr, 0b111 // Enable Interrupts
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
    msr daifset, 0b1111 // Disable interrupts
    str lr, [sp, #-16]!
    bl push_frame
    mov x3, sp
    bl handle_synchronous_exception
    bl pop_frame
    ldr lr, [sp], #16
    //msr daifclr, 0b10 // Enable Interrupts
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
    sub sp, sp, 0x360 // TODO: check math
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
    mrs x21, elr_el1 // TODO: with user programs may also need to save spsr
    mrs x22, spsr_el1
    stp x21, x22, [sp, 0x100] // Dont want to mess with syscall argument registers

    stp q0, q1, [sp, 0x110]
    stp q2, q3, [sp, 0x130]
    stp q4, q5, [sp, 0x150]
    stp q6, q7, [sp, 0x170]
    stp q8, q9, [sp, 0x190]
    stp q10, q11, [sp, 0x1b0]
    stp q12, q13, [sp, 0x1e0]
    stp q14, q15, [sp, 0x210]
    stp q16, q17, [sp, 0x230]
    stp q18, q19, [sp, 0x250]
    stp q20, q21, [sp, 0x270]
    stp q22, q23, [sp, 0x290]
    stp q24, q25, [sp, 0x2b0]
    stp q26, q27, [sp, 0x2e0]
    stp q28, q29, [sp, 0x310]
    stp q30, q31, [sp, 0x330]
    mrs x21, fpsr
    str x21, [sp, 0x350]
    ret

pop_frame:
    ldp x0, x1, [sp, 0x100]
    msr elr_el1, x0
    msr spsr_el1, x1
    ldr x1, [sp, 0x350]
    msr fpsr, x1
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
    
    ldp q0, q1, [sp, 0x110]
    ldp q2, q3, [sp, 0x130]
    ldp q4, q5, [sp, 0x150]
    ldp q6, q7, [sp, 0x170]
    ldp q8, q9, [sp, 0x190]
    ldp q10, q11, [sp, 0x1b0]
    ldp q12, q13, [sp, 0x1e0]
    ldp q14, q15, [sp, 0x210]
    ldp q16, q17, [sp, 0x230]
    ldp q18, q19, [sp, 0x250]
    ldp q20, q21, [sp, 0x270]
    ldp q22, q23, [sp, 0x290]
    ldp q24, q25, [sp, 0x2b0]
    ldp q26, q27, [sp, 0x2e0]
    ldp q28, q29, [sp, 0x310]
    ldp q30, q31, [sp, 0x330]
    add sp, sp, 0x360
    ret
