  .file "project-eval/testcases/functional/04_arr_defn3.sy"
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
    addi sp, sp, -32
    add a7, x0, sp
  L_2:
    addi sp, sp, -32
    add a1, x0, sp
    li a0, 4
    add a3, a1, a0
    li a0, 1
    sw a0, 0(a1)
    addi sp, sp, 32
  L_3:
    li a0, 4
    add a2, a3, a0
    li a0, 2
    sw a0, 0(a3)
  L_4:
    li a0, 4
    add a1, a2, a0
    li a0, 3
    sw a0, 0(a2)
  L_5:
    li a0, 4
    add a2, a1, a0
    li a0, 4
    sw a0, 0(a1)
  L_6:
    li a0, 4
    add a1, a2, a0
    li a0, 5
    sw a0, 0(a2)
  L_7:
    li a0, 4
    add a2, a1, a0
    li a0, 6
    sw a0, 0(a1)
  L_8:
    li a0, 4
    add a1, a2, a0
    li a0, 7
    sw a0, 0(a2)
  L_9:
    li a0, 4
    add a0, a1, a0
    li a0, 8
    sw a0, 0(a1)
  L_10:
    addi sp, sp, -32
    add a6, x0, sp
    li a0, 4
    add a1, a6, a0
    li a0, 1
    sw a0, 0(a6)
  L_11:
    li a0, 4
    add a2, a1, a0
    li a0, 2
    sw a0, 0(a1)
  L_12:
    li a0, 4
    add a1, a2, a0
    li a0, 3
    sw a0, 0(a2)
  L_13:
    li a0, 4
    add a2, a1, a0
    li a0, 4
    sw a0, 0(a1)
  L_14:
    li a0, 4
    add a1, a2, a0
    li a0, 5
    sw a0, 0(a2)
  L_15:
    li a0, 4
    add a2, a1, a0
    li a0, 6
    sw a0, 0(a1)
  L_16:
    li a0, 4
    add a1, a2, a0
    li a0, 7
    sw a0, 0(a2)
  L_17:
    li a0, 4
    add a0, a1, a0
    li a0, 8
    sw a0, 0(a1)
  L_18:
    addi sp, sp, -32
    add a3, x0, sp
    li a0, 4
    add a1, a3, a0
    li a0, 1
    sw a0, 0(a3)
  L_19:
    li a0, 4
    add a2, a1, a0
    li a0, 2
    sw a0, 0(a1)
  L_20:
    li a0, 4
    add a1, a2, a0
    li a0, 3
    sw a0, 0(a2)
    li a0, 4
    add a2, a1, a0
    sw x0, 0(a1)
  L_21:
    li a0, 4
    add a1, a2, a0
    li a0, 5
    sw a0, 0(a2)
    li a0, 4
    add a2, a1, a0
    sw x0, 0(a1)
  L_22:
    li a0, 4
    add a1, a2, a0
    li a0, 7
    sw a0, 0(a2)
  L_23:
    li a0, 4
    add a0, a1, a0
    li a0, 8
    sw a0, 0(a1)
  L_24:
    addi sp, sp, -32
    add a5, x0, sp
  L_25:
    li a1, 2
    li a0, 8
    mulw a0, a1, a0
    add a2, a3, a0
    addi sp, sp, 32
  L_26:
    li a0, 4
    add a4, a5, a0
    li a0, 1
    li a1, 4
    mulw a0, a0, a1
    add a0, a2, a0
    lw a0, 0(a0)
    sw a0, 0(a5)
  L_27:
  L_28:
    li a1, 2
    li a0, 8
    mulw a0, a1, a0
    add a2, a6, a0
    addi sp, sp, 32
  L_29:
    li a0, 4
    add a3, a4, a0
    li a1, 1
    li a0, 4
    mulw a0, a1, a0
    add a0, a2, a0
    lw a0, 0(a0)
    sw a0, 0(a4)
  L_30:
    li a0, 4
    add a2, a3, a0
    li a0, 3
    sw a0, 0(a3)
  L_31:
    li a0, 4
    add a1, a2, a0
    li a0, 4
    sw a0, 0(a2)
  L_32:
    li a0, 4
    add a2, a1, a0
    li a0, 5
    sw a0, 0(a1)
  L_33:
    li a0, 4
    add a3, a2, a0
    li a0, 6
    sw a0, 0(a2)
  L_34:
    li a0, 4
    add a1, a3, a0
    li a0, 7
    sw a0, 0(a3)
  L_35:
    li a0, 4
    add a0, a1, a0
    li a0, 8
    sw a0, 0(a1)
  L_36:
  L_37:
    li a1, 3
    li a0, 8
    mulw a0, a1, a0
    add a2, a5, a0
  L_38:
    li a0, 1
    li a1, 4
    mulw a0, a0, a1
    add a0, a2, a0
    lw a2, 0(a0)
  L_39:
  L_40:
    li a0, 8
    mulw a0, x0, a0
    add a1, a5, a0
  L_41:
    li a0, 4
    mulw a0, x0, a0
    add a0, a1, a0
    lw a0, 0(a0)
    addw a3, a2, a0
  L_42:
  L_43:
    li a0, 8
    mulw a0, x0, a0
    add a2, a5, a0
    addi sp, sp, 32
  L_44:
    li a1, 1
    li a0, 4
    mulw a0, a1, a0
    add a0, a2, a0
    lw a0, 0(a0)
    addw a2, a3, a0
  L_45:
  L_46:
    li a1, 2
    li a0, 8
    mulw a0, a1, a0
    add a1, a7, a0
    addi sp, sp, 32
  L_47:
    li a0, 4
    mulw a0, x0, a0
    add a0, a1, a0
    lw a0, 0(a0)
    addw a0, a2, a0
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
