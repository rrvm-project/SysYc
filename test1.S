  .file "./project-eval/testcases/performance/recursive_call_3.sy"
  .option nopic
  .attribute unaligned_access, 0
  .attribute stack_align, 16
  .text
  .text
  .global main
  .align 1
  .type func, @function
func:
    addi sp, sp, -16
    sd s1, 0(sp)
    sd ra, 8(sp)
    addi sp, sp, -16
    mv s1, sp
    addi sp, sp, -16
    fsd fa0, 0(sp)
    mv a1, s1
    call func_calc_coef
    fld fa0, 0(sp)
    addi sp, sp, 16
    flw fa2, 0(s1)
    li a1, 4
    add s1, s1, a1
    flw fa1, 0(s1)
    mulw fa0, fa2, fa0
    addw fa0, fa1, fa0
    addi sp, sp, 16
    ld s1, 0(sp)
    ld ra, 8(sp)
    addi sp, sp, 16
    ret
  .size func, .-func
  .align 1
  .type func_calc_coef, @function
func_calc_coef:
    addi sp, sp, -16
    sd ra, 0(sp)
    sd s1, 8(sp)
    li s1, 0
    blt a1, s1, L_1
    addiw a1, a1, -1
    call func_calc_coef
    la fa2, addr
    li a0, 4
    add a0, fa2, a0
    flw fa1, 0(a0)
    li a1, 1065353216
    addi s1, a1, 0
    fmv.w.x fa0, s1
    fadd.s fa0, fa0, fa2
    fmul.s fa3, fa2, fa0
    fmul.s fa4, fa2, fa1
    fadd.s fa4, fa1, fa4
    fsub.s fa3, fa0, fa3
    fsub.s fa1, fa1, fa4
    fsw fa3, 0(fa2)
    fsw fa1, 0(a0)
    j L_2
  L_1:
    li s1, 4
    add a1, a0, s1
    fmv.w.x fa2, x0
    fsw fa2, 0(a1)
    sw x0, 0(a0)
  L_2:
    ld ra, 0(sp)
    ld s1, 8(sp)
    addi sp, sp, 16
    ret
  .size func_calc_coef, .-func_calc_coef
  .align 1
  .type main, @function
main:
    addi sp, sp, -16
    sd ra, 0(sp)
    sd s1, 8(sp)
    li a0, 21
    addi sp, sp, -16
    sd a0, 0(sp)
    call _sysy_starttime
    ld a0, 0(sp)
    addi sp, sp, 16
    call getint
    li s1, 1065361408
    addi s1, s1, 197
    fmv.w.x fa1, s1
    addi sp, sp, -16
    fsd fa1, 0(sp)
    fmv.s fa0, fa1
    call func
    fld fa1, 0(sp)
    addi sp, sp, 16
    fsub.s fa0, fa0, fa1
    fmv.w.x fa1, x0
    feq.s s1, fa0, fa1
    bne s1, x0, L_3
    j L_4
  L_3:
    li a0, 112
    call putch
  L_4:
    li a0, 32
    call _sysy_stoptime
    mv a0, x0
    ld ra, 0(sp)
    ld s1, 8(sp)
    addi sp, sp, 16
    ret
  .size main, .-main
  .ident "SYSYC: (made by RRVM) 1.0.0"
