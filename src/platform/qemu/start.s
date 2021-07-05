.global _start
.extern LD_STACK_PTR

.section ".text.boot"

_start:
    ldr     x30, =LD_STACK_PTR //Load the initial stack pointer value into x30
    mov     sp, x30 //Load x30 into the stack pointer
    bl      start //Enter the start function

.equ PSCI_SYSTEM_OFF, 0x84000008
.global system_off
system_off: //Instruct the hypervisor to shut down
    ldr     x0, =PSCI_SYSTEM_OFF
    hvc     #0