.global sqrt_asm
.global abs_asm
.global round_asm

sqrt_asm:
    fsqrt d0, d0

abs_asm:
    fabs d0, d0

round_asm:
    fcvtzs x0, d0