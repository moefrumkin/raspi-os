.global _start
.extern LD_STACK_PTR

.section ".text.boot"

_start:
    mrs x0, mpidr_el1 //Copy the value from the aarch64 Multi-Processor Affinity Register to general purpose register 0
    and x0, x0, #0xFF //Save the unsigned lowest 8 bits of mpidr_el1  in x0
    cbz x0, master //if the value of x0 is 0, branch to master
    b hang //otherwise branch to hang

master:
    ldr x0, =BSS_START //load bss bounds into registers x0 and x1
    ldr x1, =BSS_END
    b bss_init

bss_init:
    cmp x0, x1
    beq enter_rust
    str xzr, [x0]
    add x0, x0, #8

enter_rust:
    ldr     x30, =LD_STACK_PTR //Copy intial value of the Stack Pointer, as defined by the linker to general purpose register 30 
    mov     sp, x30 //Copy the value from x30 to the stack pointer
    bl start //Enter the start function

hang:
    b hang //Loop by branching back to the hang label