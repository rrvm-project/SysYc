  .file "project-eval/testcases/functional/34_arr_expr_len.sy"
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
    addi a3, x0, 0
    addi a1, x0, 0
  L_4:
  L_5:
    slti a0, a1, 6
    bne a0, x0, L_6
  L_7:
  L_8:
  L_9:
    add a0, x0, a3
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
  L_6:
  L_10:
  L_11:
    la a2, arr
  L_12:
    li a0, 4
    mulw a0, a1, a0
    add a0, a2, a0
    lw a0, 0(a0)
    addw a0, a3, a0
  L_13:
  L_14:
  L_15:
    addw a3, x0, a0
    addi a0, a1, 1
    addw a1, x0, a0
    beq x0, x0, L_4
  .data
  .global arr
  .type arr, @object
  .size arr, 24
arr:
  .word 1
  .word 2
  .word 33
  .word 4
  .word 5
  .word 6
  .bss
  .global N
  .type N, @object
  .size N, 4
N:
  .space 4
  .ident "SYSYC: (made by RRVM) 0.0.1"
