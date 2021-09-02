.global sqrt_asm
.global abs_asm
.global round_asm

sqrt_asm:
    fsqrt d0, d0
    ret

abs_asm:
    fabs d0, d0
    ret

round_asm:
    fcvtzs x0, d0
    ret