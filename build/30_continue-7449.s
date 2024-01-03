  .file "project-eval/testcases/functional/30_continue.sy"
  .option nopic
  .attribute unaligned_access, 0
  .attribute stack_align, 16
  .text
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
  L_1:
  L_2:
  L_3:
  L_4:
  L_5:
  L_6:
  L_7:
    addi a1, x0, 0
    addi a2, x0, 0
  L_8:
  L_9:
    slti a0, a1, 100
    bne a0, x0, L_10
  L_11:
  L_12:
  L_13:
    add a0, x0, a2
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
    xori a0, a1, 50
    sltiu a0, a0, 1
    bne a0, x0, L_15
  L_16:
    addw a1, x0, a1
  L_17:
  L_18:
  L_19:
  L_20:
    addw a0, a2, a1
  L_21:
  L_22:
  L_23:
    addw a2, x0, a0
    addi a0, a1, 1
    addw a1, x0, a0
    beq x0, x0, L_8
  L_15:
  L_24:
  L_25:
    addi a0, a1, 1
  L_26:
    addw a1, x0, a0
    addw a1, x0, a0
    beq x0, x0, L_8
  .ident "SYSYC: (made by RRVM) 0.0.1"
