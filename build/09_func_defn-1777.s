  .file "project-eval/testcases/functional/09_func_defn.sy"
  .option nopic
  .attribute unaligned_access, 0
  .attribute stack_align, 16
  .text
  .align 1
  .global func
  .type func, @function
func:
    addi sp, sp, -96
    sd s1, 0(sp)
    sd s2, 8(sp)
    sd s3, 16(sp)
    sd s4, 24(sp)
    sd s5, 32(sp)
    sd s6, 40(sp)
    sd s7, 48(sp)
    sd s8, 56(sp)
    sd s9, 64(sp)
    sd s10, 72(sp)
    sd s11, 80(sp)
  L_1:
  L_2:
  L_3:
    li a1, 1
    subw a0, a0, a1
  L_4:
    ld s1, 0(sp)
    ld s2, 8(sp)
    ld s3, 16(sp)
    ld s4, 24(sp)
    ld s5, 32(sp)
    ld s6, 40(sp)
    ld s7, 48(sp)
    ld s8, 56(sp)
    ld s9, 64(sp)
    ld s10, 72(sp)
    ld s11, 80(sp)
    addi sp, sp, 96
    ret
  .align 1
  .global main
  .type main, @function
main:
    addi sp, sp, -96
    sd s1, 0(sp)
    sd s2, 8(sp)
    sd s3, 16(sp)
    sd s4, 24(sp)
    sd s5, 32(sp)
    sd s6, 40(sp)
    sd s7, 48(sp)
    sd s8, 56(sp)
    sd s9, 64(sp)
    sd s10, 72(sp)
    sd s11, 80(sp)
  L_5:
  L_6:
    la a1, a
  L_7:
    li a0, 10
    sw a0, 0(a1)
  L_8:
  L_9:
    addi sp, sp, -128
    sd a1, 0(sp)
    sd a2, 8(sp)
    sd a3, 16(sp)
    sd a4, 24(sp)
    sd a5, 32(sp)
    sd a6, 40(sp)
    sd a7, 48(sp)
    sd t0, 56(sp)
    sd t1, 64(sp)
    sd t2, 72(sp)
    sd t3, 80(sp)
    sd t4, 88(sp)
    sd t5, 96(sp)
    sd t6, 104(sp)
    sd ra, 112(sp)
    addi a0, x0, 10
    jal ra, func
    ld a1, 0(sp)
    ld a2, 8(sp)
    ld a3, 16(sp)
    ld a4, 24(sp)
    ld a5, 32(sp)
    ld a6, 40(sp)
    ld a7, 48(sp)
    ld t0, 56(sp)
    ld t1, 64(sp)
    ld t2, 72(sp)
    ld t3, 80(sp)
    ld t4, 88(sp)
    ld t5, 96(sp)
    ld t6, 104(sp)
    ld ra, 112(sp)
    addi sp, sp, 128
  L_10:
    ld s1, 0(sp)
    ld s2, 8(sp)
    ld s3, 16(sp)
    ld s4, 24(sp)
    ld s5, 32(sp)
    ld s6, 40(sp)
    ld s7, 48(sp)
    ld s8, 56(sp)
    ld s9, 64(sp)
    ld s10, 72(sp)
    ld s11, 80(sp)
    addi sp, sp, 96
    ret
  .bss
  .global a
  .type a, @object
  .size a, 4
a:
  .space 4
  .ident "SYSYC: (made by RRVM) 0.0.1"
