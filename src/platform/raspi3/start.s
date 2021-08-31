.section ".text.boot"

.global _start

_start:
    // read cpu id, stop slave cores
    mrs     x1, mpidr_el1
    and     x1, x1, #3
    cbz     x1, 2f
    // cpu id > 0, stop
1:  wfe
    b       1b
2:  // cpu id == 0

    // set stack before our code
    ldr     x1, =_start
    //msr     SPSel, xzr //use sp_el0 as the default sp
    mov     sp, x1

    // clear bss
    ldr     x1, =bss_start
    ldr     w2, =bss_size
3:  cbz     w2, 4f
    str     xzr, [x1], #8
    sub     w2, w2, #1
    cbnz    w2, 3b

    // set up exception handlers and jump to Rust, should not return
4:  ldr     x0, =_exception_vector
    msr     VBAR_EL1, x0
    ldr     x0, =HEAP_START
    b       main