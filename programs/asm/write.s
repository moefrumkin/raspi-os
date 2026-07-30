.globl _start

.data

msg: 
    .ascii  "Hello, World!\n"
msg_len = . - msg

stdio:
    .ascii "stdio"
stdio_len = . - stdio

.text

_start:
    ldr x0, =stdio
    mov x1, stdio_len
    svc #6 // Open
    ldr x1, =msg
    mov x2, msg_len
    svc #9 // Write
    mov x0, 0
    svc #2 // Exit