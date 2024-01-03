  .file "project-eval/testcases/functional/26_while_test1.sy"
  .option nopic
  .attribute unaligned_access, 0
  .attribute stack_align, 16
  .text
  .align 1
  .global doubleWhile
  .type doubleWhile, @function
doubleWhile:
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
  L_4:
  L_5:
  L_6:
  L_7:
    addi a2, x0, 5
    addi a1, x0, 7
  L_8:
  L_9:
    slti a0, a2, 100
    bne a0, x0, L_10
  L_11:
  L_12:
  L_13:
    add a0, x0, a1
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
  L_10:
  L_14:
  L_15:
    addi a2, a2, 30
  L_16:
    addw a1, x0, a1
  L_17:
  L_18:
    slti a0, a1, 100
    bne a0, x0, L_19
  L_20:
  L_21:
  L_22:
  L_23:
  L_24:
    addw a2, x0, a2
    li a0, 100
    subw a0, a1, a0
    addw a1, x0, a0
    beq x0, x0, L_8
  L_19:
  L_25:
  L_26:
    addi a0, a1, 6
    addw a1, x0, a0
    beq x0, x0, L_17
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
  L_27:
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
    jal ra, doubleWhile
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
  .ident "SYSYC: (made by RRVM) 0.0.1"
