  .file "project-eval/testcases/functional/27_while_test2.sy"
  .option nopic
  .attribute unaligned_access, 0
  .attribute stack_align, 16
  .text
  .align 1
  .global FourWhile
  .type FourWhile, @function
FourWhile:
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
  L_8:
  L_9:
  L_10:
  L_11:
  L_12:
  L_13:
    addi a2, x0, 7
    addi a5, x0, 5
    addi a4, x0, 10
    addi a1, x0, 6
  L_14:
  L_15:
    slti a0, a5, 20
    bne a0, x0, L_16
  L_17:
  L_18:
  L_19:
  L_20:
  L_21:
    addw a0, a1, a4
    addw a0, a5, a0
  L_22:
    addw a0, a0, a2
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
  L_16:
  L_23:
  L_24:
    addi a5, a5, 3
  L_25:
    addw a3, x0, a1
    addw a2, x0, a2
    addw a1, x0, a4
  L_26:
  L_27:
    slti a0, a3, 10
    bne a0, x0, L_28
  L_29:
  L_30:
  L_31:
  L_32:
  L_33:
    addw a2, x0, a2
    addw a5, x0, a5
    addw a4, x0, a1
    li a0, 2
    subw a0, a3, a0
    addw a1, x0, a0
    beq x0, x0, L_14
  L_28:
  L_34:
  L_35:
    addi a3, a3, 1
  L_36:
    addw a1, x0, a1
    addw a2, x0, a2
  L_37:
  L_38:
    xori a0, a2, 7
    sltiu a0, a0, 1
    bne a0, x0, L_39
  L_40:
  L_41:
  L_42:
  L_43:
  L_44:
    addw a1, x0, a1
    addw a3, x0, a3
    addi a0, a2, 1
    addw a2, x0, a0
    beq x0, x0, L_26
  L_39:
  L_45:
  L_46:
    li a0, 1
    subw a2, a2, a0
  L_47:
    addw a1, x0, a1
  L_48:
  L_49:
    slti a0, a1, 20
    bne a0, x0, L_50
  L_51:
  L_52:
  L_53:
  L_54:
  L_55:
    addw a2, x0, a2
    li a0, 1
    subw a0, a1, a0
    addw a1, x0, a0
    beq x0, x0, L_37
  L_50:
  L_56:
  L_57:
    addi a0, a1, 3
    addw a1, x0, a0
    beq x0, x0, L_48
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
  L_58:
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
    jal ra, FourWhile
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
