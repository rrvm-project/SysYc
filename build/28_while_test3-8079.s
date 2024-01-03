  .file "project-eval/testcases/functional/28_while_test3.sy"
  .option nopic
  .attribute unaligned_access, 0
  .attribute stack_align, 16
  .text
  .align 1
  .global EightWhile
  .type EightWhile, @function
EightWhile:
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
    addw t5, x0, a5
    addw a3, x0, t3
    addi a7, x0, 10
    addw a0, x0, t0
    addi t2, x0, 5
    addi a1, x0, 7
    addw t4, x0, a4
    addi a2, x0, 6
  L_14:
  L_15:
    slti a0, t2, 20
    bne a0, x0, L_16
  L_17:
  L_18:
  L_19:
  L_20:
  L_21:
    addw a0, a2, a7
    addw a0, t2, a0
  L_22:
    addw a1, a0, a1
  L_23:
  L_24:
    addw a0, a3, a7
  L_25:
    subw a0, a0, t5
  L_26:
    addw a0, a0, t4
    subw a0, a1, a0
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
  L_27:
  L_28:
    addi s1, t2, 3
  L_29:
    addw a3, x0, a4
    addw t4, x0, t0
    addw a1, x0, a1
    addw t2, x0, a5
    addw a6, x0, a2
    addw a7, x0, a7
    addw t1, x0, t3
  L_30:
  L_31:
    slti a0, a6, 10
    bne a0, x0, L_32
  L_33:
  L_34:
  L_35:
  L_36:
  L_37:
    addw t5, x0, t2
    addw a0, x0, t4
    addw t2, x0, s1
    addw a1, x0, a1
    addw t4, x0, a3
    addw a3, x0, t1
    addw a7, x0, a7
    li a0, 2
    subw a0, a6, a0
    addw a2, x0, a0
    beq x0, x0, L_14
  L_32:
  L_38:
  L_39:
    addi t6, a6, 1
  L_40:
    addw a2, x0, a1
    addw t2, x0, a5
    addw a3, x0, a4
    addw t1, x0, t3
    addw a6, x0, t0
    addw a1, x0, a7
  L_41:
  L_42:
    xori a0, a2, 7
    sltiu a0, a0, 1
    bne a0, x0, L_43
  L_44:
  L_45:
  L_46:
  L_47:
  L_48:
    addw t4, x0, a6
    addw a7, x0, a1
    addw t2, x0, t2
    addw t1, x0, t1
    addw a6, x0, t6
    addw a3, x0, a3
    addi a0, a2, 1
    addw a1, x0, a0
    beq x0, x0, L_30
  L_43:
  L_49:
  L_50:
    li a0, 1
    subw t5, a2, a0
  L_51:
    addw a6, x0, a5
    addw a7, x0, t0
    addw a3, x0, a4
    addw a1, x0, a1
    addw a2, x0, t3
  L_52:
  L_53:
    slti a0, a1, 20
    bne a0, x0, L_54
  L_55:
  L_56:
  L_57:
  L_58:
  L_59:
    addw t2, x0, a6
    addw t1, x0, a2
    addw a6, x0, a7
    addw a3, x0, a3
    addw a2, x0, t5
    li a0, 1
    subw a0, a1, a0
    addw a1, x0, a0
    beq x0, x0, L_41
  L_54:
  L_60:
  L_61:
    addi t4, a1, 3
  L_62:
    addw t1, x0, a5
    addw t2, x0, t3
    addw a2, x0, t0
    addw a3, x0, a4
  L_63:
    la a0, e
    lw a1, 0(a0)
  L_64:
    li a0, 1
    slt a0, a0, a1
    bne a0, x0, L_65
  L_66:
  L_67:
  L_68:
  L_69:
  L_70:
    addw a7, x0, a2
    addw a6, x0, t1
    addw a3, x0, a3
    addw a1, x0, t4
    addi a0, t2, 1
    addw a2, x0, a0
    beq x0, x0, L_52
  L_65:
    la a2, e
  L_71:
    la a0, e
    lw a1, 0(a0)
  L_72:
    li a0, 1
    subw t2, a1, a0
    sw t2, 0(a2)
  L_73:
    addw a6, x0, a4
    addw a2, x0, a5
    addw a7, x0, t0
  L_74:
    la a0, f
    lw a1, 0(a0)
  L_75:
    li a0, 2
    slt a0, a0, a1
    bne a0, x0, L_76
  L_77:
  L_78:
  L_79:
  L_80:
  L_81:
    addw t1, x0, a2
    addw t2, x0, t2
    addw a3, x0, a6
    addi a0, a7, 1
    addw a2, x0, a0
    beq x0, x0, L_63
  L_76:
    la a2, f
  L_82:
    la a0, f
    lw a1, 0(a0)
  L_83:
    li a0, 2
    subw a7, a1, a0
    sw a7, 0(a2)
  L_84:
    addw a3, x0, a5
    addw a1, x0, a4
  L_85:
    la a0, g
    lw a0, 0(a0)
  L_86:
    slti a0, a0, 3
    bne a0, x0, L_87
  L_88:
  L_89:
  L_90:
  L_91:
  L_92:
    addw a6, x0, a1
    addw a7, x0, a7
    li a0, 8
    subw a0, a3, a0
    addw a2, x0, a0
    beq x0, x0, L_74
  L_87:
    la a1, g
  L_93:
    la a0, g
    lw a0, 0(a0)
  L_94:
    addi a3, a0, 10
    sw a3, 0(a1)
  L_95:
    addw a2, x0, a4
  L_96:
    la a0, h
    lw a0, 0(a0)
  L_97:
    slti a0, a0, 10
    bne a0, x0, L_98
  L_99:
  L_100:
  L_101:
  L_102:
  L_103:
    addw a3, x0, a3
    li a0, 1
    subw a0, a2, a0
    addw a1, x0, a0
    beq x0, x0, L_85
  L_98:
    la a1, h
  L_104:
    la a0, h
    lw a0, 0(a0)
  L_105:
    addi a0, a0, 8
    addw a2, x0, a0
    sw a0, 0(a1)
    beq x0, x0, L_96
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
  L_106:
  L_107:
  L_108:
  L_109:
  L_110:
  L_111:
  L_112:
  L_113:
  L_114:
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
    jal ra, EightWhile
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
  .bss
  .global g
  .type g, @object
  .size g, 4
g:
  .space 4
  .global h
  .type h, @object
  .size h, 4
h:
  .space 4
  .global f
  .type f, @object
  .size f, 4
f:
  .space 4
  .global e
  .type e, @object
  .size e, 4
e:
  .space 4
  .ident "SYSYC: (made by RRVM) 0.0.1"
