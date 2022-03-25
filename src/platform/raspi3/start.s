.section ".text.boot"

.global _start

_start: //spin if not main core
    mrs     x1, mpidr_el1 // the mpidr_el1 register contains the core id for each core in bits 0-7
    and     x1, x1, 0b11 // since there are only 4 cores, bits 0-1 will suffice
    cbz     x1, 2f // branch to 2f if the core id is 0

1:  wfe
    b       1b // otherwise, spin

2:  // cpu id == 0

    // set top of stack just before our code (stack grows to a lower address per AAPCS64)
    //ldr     x1, =_start

    // detect the current el
    mrs     x0, CurrentEL // CurrentEL stores the exception level in bits 3-2
    and     x0, x0, 0b1100

    // switch depending on the current EL
    cmp     x0, #12 // Since the EL is stored at a 2 bit offset, EL3 is represented by 3 << 2 = 12
    bne     5f // if the exception level is not EL3 jump to 5f

    // otherwise, switch 5f in EL2

    /*
        Secure Configuration Register
        [0]: NS: 1 indicates lower ELs run in non-secure state
        [1]: IRQ: 0 indicates IRQ interrupts not taken to EL3
        [2]: FIQ: 0 indicates FIQ interrupts not taken to EL3
        [3]: EA: 0 indicates External aborts and SError interrupts are not taken to EL3
        [5:4]: RES1
        [6]: RES0
        [7]: SMD: 1 indicates SMC instructions are disabled
        [8]: HCE: 1 indicates HVC instructions are enabled
        [9]: SIF: 0 Allows secure access of non-secure memory
        [10]: RW: 1 indicates that EL2 runs in AArch64 mode
    */
    mov     x2, 0b10110110001
    msr     scr_el3, x2

    /*
        Saved Program Status Register
        [0:3]: 1001 indicates EL2h stack pointer used. h indicates SP_EL2 used
        [4]: 0 indicates AArch64 execution state
        [5]: RES0
        [6]: F: 1 masks FIQ exceptions
        [7]: I: 1 masks IRQ exceptions
        [8]: A: 1 masks SErrors
        [9]: D: 1 masks Watchpoint, Breakpoint and Software Step exceptions
    */
    mov     x2, 0b1111001001
    msr     spsr_el3, x2

    //set the exception link register to point to 5f
    adr     x2, 5f
    msr     elr_el3, x2
    
    eret

    // from el2, return to 5f in EL1
5:  cmp     x0, #4 // if in el1, all set
    beq     5f

    //msr     sp_el1, x1
    
    /*
        Counter Timer Hypervisor Control Register
        read it as to avoid messing around with the other bits, and set bits 0 and 1 to high which allows EL1 to access timer and couner registers
    */
    mrs     x0, cnthctl_el2
    orr     x0, x0, #3
    msr     cnthctl_el2, x0
    msr     cntvoff_el2, xzr // set the virtual counter offest to 0

    /*
        Architectural Feature Trap Register
        [20:21]: 11 untraps SIMD and FP instructions and registers
    */
    mov     x0, (0b11 << 20)
    msr     cptr_el2, x0
    msr     cpacr_el1, x0

    
    // Hypervisor Control Register
    mov     x0, #(1 << 31)  // RW: 1 sets EL1 to AArch64
    orr     x0, x0, #(1 << 1)   // cache invalidate by Set/Way will also clean
    msr     hcr_el2, x0
    mrs     x0, hcr_el2

    /*
        Saved Program Status Register
        [0:3]: 0100 sets the stack pointer to EL1t. t indicates SP_EL0
        [4]: 0 indicates AArch64 execution state
        [5]: RES0
        [6]: F: 1 masks FIQ exceptions
        [7]: I: 1 masks IRQ exceptions
        [8]: A: 1 masks SErrors
        [9]: D: 1 masks Watchpoint, Breakpoint and Software Step exceptions

    */
    mov     x2, 0b1111000100
    msr     spsr_el2, x2

    //set the exception link register to point to 5f
    adr     x2, 5f
    msr     elr_el2, x2
    
    eret

    //set stack pointer
5:  ldr     x0, =_start
    mov     sp, x0

    // clear bss
3:  ldr     x1, =bss_start
    ldr     w2, =bss_size
    cbz     w2, 4f
    str     xzr, [x1], #8
    sub     w2, w2, #1
    cbnz    w2, 3b

    // set up exception handlers and jump to Rust, should not return
4:  ldr     x0, =_exception_vector
    msr     VBAR_EL1, x0
    ldr     x0, =HEAP_START
    ldr     x1, =HEAP_SIZE
    ldr     x2, =MAILBOX_BUFFER_START
    ldr     x3, =TABLE_START

    b       main
