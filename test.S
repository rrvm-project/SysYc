  .file "./project-eval/testcases/performance/recursive_call_3.sy"
  .option nopic
	.attribute arch, "rv64i2p1_m2p0_a2p1_f2p2_d2p2_c2p0_zicsr2p0_zifencei2p0_zba1p0_zbb1p0"
  .attribute unaligned_access, 0
  .attribute stack_align, 16
  .text
  .text
  .global main
  .align 1
  .type main, @function
main:
    addi sp, sp, -32
    sd ra, 0(sp)
    sd s2, 8(sp)
    sd s1, 16(sp)
    fmv.w.x fa1, x0
    li a0, 21
    addi sp, sp, -16
    fsd fa1, 0(sp)
    sd a0, 8(sp)
    call _sysy_starttime
    fld fa1, 0(sp)
    ld a0, 8(sp)
    addi sp, sp, 16
    addi sp, sp, -16
    fsd fa1, 0(sp)
    call getint
    fld fa1, 0(sp)
    addi sp, sp, 16
    li s1, 1065361408
    addi s1, s1, 197
    slti s2, a0, 0
    seqz s2, s2
    andi a0, a0, 1
    fmv.w.x fa0, s1
    mulw s2, a0, s2
    fcvt.s.w fa2, s2
    fmul.s fa2, fa2, fa0
    fsub.s fa0, fa2, fa0
    feq.s a0, fa0, fa1
    bne a0, x0, L_1
    j L_2
  L_1:
    li a0, 112
    call putch
  L_2:
    li a0, 32
    call _sysy_stoptime
    mv a0, x0
    ld ra, 0(sp)
    ld s2, 8(sp)
    ld s1, 16(sp)
    addi sp, sp, 32
    ret
  .size main, .-main
  .ident "SYSYC: (made by RRVM) 1.0.0"
