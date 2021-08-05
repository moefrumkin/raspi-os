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
    mov     sp, x1

    // clear bss
    ldr     x1, =__bss_start
    ldr     w2, =__bss_size
3:  cbz     w2, 4f
    str     xzr, [x1], #8
    sub     w2, w2, #1
    cbnz    w2, 3b

    // jump to C code, should not return
4:  mov x2, 0x8
    mov x0, 0x1c
    movk x2, #0x3f20, lsl #16
    movk x0, #0x3f20, lsl #16
    mov w3, #0x8
    mov w1, #0x200000
    str w3, [x2]
    str w1, [x0]
    b       main
    b       1b