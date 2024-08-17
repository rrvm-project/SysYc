  .file "./project-eval/testcases/performance/fft0.sy"
  .option nopic
  .attribute unaligned_access, 0
  .attribute stack_align, 16
  .text
  .global d
  .section	.sbss, "aw", @nobits
  .align 2
  .type d, @object
  .size d, 4
d:
  .zero 4
  .global temp
  .align 2
  .type temp, @object
  .size temp, 8388608
temp:
  .zero 8388608
  .global a
  .align 2
  .type a, @object
  .size a, 8388608
a:
  .zero 8388608
  .global b
  .align 2
  .type b, @object
  .size b, 8388608
b:
  .zero 8388608
  .global c
  .align 2
  .type c, @object
  .size c, 8388608
c:
  .zero 8388608
  .global mod
  .section	.sdata, "aw"
  .align 2
  .type mod, @object
  .size mod, 4
mod:
  .word 998244353
  .global maxlen
  .align 2
  .type maxlen, @object
  .size maxlen, 4
maxlen:
  .word 2097152
  .text
  .global main
  .align 1
  .type multiply, @function
multiply:
    addi sp, sp, -48
    sd s2, 0(sp)
    sd s3, 8(sp)
    sd s5, 16(sp)
    sd s1, 24(sp)
    sd ra, 32(sp)
    sd s4, 40(sp)
    mv s5, a1
    mv s1, a0
    li a1, 0
    beq s5, a1, L_1
    li s2, 1
    beq s5, s2, L_2
    srliw s4, s5, 31
    sub a1, s5, s4
    srai a1, a1, 1
    addw a1, a1, s4
    mv a0, s1
    call multiply
    addw a1, a0, a0
    li s3, 2309898240
    addi s3, s3, 136
    mul a0, s3, a1
    srliw s4, a1, 31
    sub a0, a0, s4
    srai a0, a0, 61
    addw a0, a0, s4
    li s4, 998244352
    addi s4, s4, 1
    mul a0, a0, s4
    subw a0, a1, a0
    srliw a1, s5, 31
    add a1, a1, s5
    andi a1, a1, -2
    sub s5, s5, a1
    beq s5, s2, L_3
    j L_4
  L_1:
    mv a0, x0
  L_4:
    ld s2, 0(sp)
    ld s3, 8(sp)
    ld s5, 16(sp)
    ld s1, 24(sp)
    ld ra, 32(sp)
    ld s4, 40(sp)
    addi sp, sp, 48
    ret
  L_2:
    li s3, 2309898240
    addi s5, s3, 136
    mul a0, s5, s1
    srliw s2, s1, 31
    sub a0, a0, s2
    srai a0, a0, 61
    addw a0, a0, s2
    li s2, 998244352
    addi s4, s2, 1
    mul a0, a0, s4
    subw a0, s1, a0
    j L_4
  L_3:
    addw s1, a0, s1
    mul a0, s3, s1
    srliw s3, s1, 31
    sub a0, a0, s3
    srai a0, a0, 61
    addw a0, a0, s3
    mul a0, a0, s4
    subw a0, s1, a0
    j L_4
  .size multiply, .-multiply
  .align 1
  .type power, @function
power:
    addi sp, sp, -32
    sd s1, 0(sp)
    sd s2, 8(sp)
    sd ra, 16(sp)
    mv s2, a1
    mv s1, a0
    li a1, 0
    beq s2, a1, L_5
    srliw a0, s2, 31
    sub a1, s2, a0
    srai a1, a1, 1
    addw a1, a1, a0
    mv a0, s1
    call power
    mv a1, a0
    mv a0, a1
    call multiply
    srliw a1, s2, 31
    add a1, a1, s2
    andi a1, a1, -2
    sub a1, s2, a1
    li s2, 1
    beq a1, s2, L_6
    j L_7
  L_5:
    li a0, 1
  L_7:
    ld s1, 0(sp)
    ld s2, 8(sp)
    ld ra, 16(sp)
    addi sp, sp, 32
    ret
  L_6:
    mv a1, s1
    call multiply
    j L_7
  .size power, .-power
  .align 1
  .type fft, @function
fft:
    addi sp, sp, -96
    sd s3, 0(sp)
    sd s1, 8(sp)
    sd s11, 16(sp)
    sd s4, 24(sp)
    sd s8, 32(sp)
    sd s6, 40(sp)
    sd ra, 48(sp)
    sd s7, 56(sp)
    sd s2, 64(sp)
    sd s9, 72(sp)
    sd s10, 80(sp)
    sd s5, 88(sp)
    mv a6, a3
    mv a5, a1
    mv a4, a0
    li s1, 1
    beq a2, s1, L_8
    la s6, temp
    mv s4, x0
    li s5, 0
    li s9, 1
    li s1, 2309898240
    addi s11, s1, 136
    li s1, 998244352
    addi s10, s1, 1
    j L_9
  L_8:
    li a0, 1
    j L_10
  L_11:
    srliw s1, s4, 31
    add s1, s1, s4
    andi s1, s1, -2
    sub s1, s4, s1
    beq s1, s5, L_12
    srliw s1, a2, 31
    sub s3, a2, s1
    srai s3, s3, 1
    addw s3, s3, s1
    srliw s2, s4, 31
    sub s1, s4, s2
    srai s1, s1, 1
    addw s1, s1, s2
    addw s1, s3, s1
    slliw s1, s1, 2
    add s2, s6, s1
    addw s1, s4, a5
    slliw s1, s1, 2
    add s1, a4, s1
    lw s1, 0(s1)
    sw s1, 0(s2)
    j L_13
  L_12:
    srliw s2, s4, 31
    sub s1, s4, s2
    srai s1, s1, 1
    addw s1, s1, s2
    slliw s1, s1, 2
    add s2, s6, s1
    addw s1, s4, a5
    slliw s1, s1, 2
    add s1, a4, s1
    lw s1, 0(s1)
    sw s1, 0(s2)
  L_13:
    addiw s4, s4, 1
  L_9:
    blt s4, a2, L_11
    mv s3, x0
    j L_14
  L_15:
    addw s2, a5, s7
    slliw s1, s2, 2
    add s5, a4, s1
    lw s4, 0(s5)
    addw s1, s2, s8
    slliw s1, s1, 2
    add s6, a4, s1
    lw a1, 0(s6)
    addi sp, sp, -32
    sd a6, 0(sp)
    sd a5, 8(sp)
    sd a4, 16(sp)
    mv a0, s9
    call multiply
    ld a6, 0(sp)
    ld a5, 8(sp)
    ld a4, 16(sp)
    addi sp, sp, 32
    addw s2, s4, a0
    mul s1, s11, s2
    srliw s3, s2, 31
    sub s1, s1, s3
    srai s1, s1, 61
    addw s1, s1, s3
    mul s1, s1, s10
    subw s1, s2, s1
    sw s1, 0(s5)
    subw s1, s4, a0
    addw s2, s1, s10
    mul s1, s11, s2
    srliw s3, s2, 31
    sub s1, s1, s3
    srai s1, s1, 61
    addw s1, s1, s3
    mul s1, s1, s10
    subw s1, s2, s1
    sw s1, 0(s6)
    addi sp, sp, -32
    sd a6, 0(sp)
    sd a5, 8(sp)
    sd a4, 16(sp)
    mv a0, s9
    mv a1, a6
    call multiply
    ld a6, 0(sp)
    ld a5, 8(sp)
    ld a4, 16(sp)
    addi sp, sp, 32
    mv s9, a0
    addiw s7, s7, 1
  L_16:
    blt s7, s8, L_15
    mv a0, x0
  L_10:
    ld s3, 0(sp)
    ld s1, 8(sp)
    ld s11, 16(sp)
    ld s4, 24(sp)
    ld s8, 32(sp)
    ld s6, 40(sp)
    ld ra, 48(sp)
    ld s7, 56(sp)
    ld s2, 64(sp)
    ld s9, 72(sp)
    ld s10, 80(sp)
    ld s5, 88(sp)
    addi sp, sp, 96
    ret
  L_17:
    addw s1, a5, s3
    slliw s1, s1, 2
    add s2, a4, s1
    slliw s1, s3, 2
    add s1, s6, s1
    lw s1, 0(s1)
    sw s1, 0(s2)
    addiw s3, s3, 1
  L_14:
    blt s3, a2, L_17
    srliw s1, a2, 31
    sub s8, a2, s1
    srai s8, s8, 1
    addw s8, s8, s1
    addi sp, sp, -32
    sd a6, 0(sp)
    sd a5, 8(sp)
    sd a4, 16(sp)
    mv a0, a6
    mv a1, a6
    call multiply
    ld a6, 0(sp)
    ld a5, 8(sp)
    ld a4, 16(sp)
    addi sp, sp, 32
    mv s1, a0
    addi sp, sp, -32
    sd a6, 0(sp)
    sd a5, 8(sp)
    sd a4, 16(sp)
    mv a0, a4
    mv a1, a5
    mv a2, s8
    mv a3, s1
    call fft
    ld a6, 0(sp)
    ld a5, 8(sp)
    ld a4, 16(sp)
    addi sp, sp, 32
    addw a1, a5, s8
    addi sp, sp, -32
    sd a6, 0(sp)
    sd a5, 8(sp)
    sd a4, 16(sp)
    mv a0, a4
    mv a2, s8
    mv a3, s1
    call fft
    ld a6, 0(sp)
    ld a5, 8(sp)
    ld a4, 16(sp)
    addi sp, sp, 32
    mv s7, x0
    j L_16
  .size fft, .-fft
  .align 1
  .type main, @function
main:
    addi sp, sp, -96
    sd s6, 0(sp)
    sd s3, 8(sp)
    sd s1, 16(sp)
    sd s11, 24(sp)
    sd s8, 32(sp)
    sd s9, 40(sp)
    sd s7, 48(sp)
    sd ra, 56(sp)
    sd s5, 64(sp)
    sd s4, 72(sp)
    sd s2, 80(sp)
    sd s10, 88(sp)
    la s11, a
    mv a0, s11
    call getarray
    mv s1, a0
    la s7, b
    mv a0, s7
    call getarray
    addw s1, s1, a0
    addiw s10, s1, -1
    li s9, 1
    j L_18
  L_19:
    slliw s9, s9, 1
  L_18:
    blt s9, s10, L_19
    li s8, 998244352
    addi s5, s8, 0
    divw s4, s5, s9
    li s6, 3
    mv a0, s6
    mv a1, s4
    call power
    mv s1, a0
    mv a0, s11
    mv a1, x0
    mv a2, s9
    mv a3, s1
    call fft
    mv a0, s7
    mv a1, x0
    mv a2, s9
    mv a3, s1
    call fft
    mv s3, x0
    j L_20
  L_21:
    slliw s1, s3, 2
    add s2, s11, s1
    lw a0, 0(s2)
    add s1, s7, s1
    lw a1, 0(s1)
    call multiply
    sw a0, 0(s2)
    addiw s3, s3, 1
  L_20:
    blt s3, s9, L_21
    subw a1, s5, s4
    mv a0, s6
    call power
    mv a3, a0
    mv a0, s11
    mv a1, x0
    mv a2, s9
    call fft
    mv s4, x0
    addi s3, s8, -1
    j L_22
  L_23:
    slliw s1, s4, 2
    add s2, s11, s1
    lw s1, 0(s2)
    mv a0, s9
    mv a1, s3
    call power
    mv a1, a0
    mv a0, s1
    call multiply
    sw a0, 0(s2)
    addiw s4, s4, 1
  L_22:
    blt s4, s9, L_23
    mv a0, s10
    mv a1, s11
    call putarray
    mv a0, x0
    ld s6, 0(sp)
    ld s3, 8(sp)
    ld s1, 16(sp)
    ld s11, 24(sp)
    ld s8, 32(sp)
    ld s9, 40(sp)
    ld s7, 48(sp)
    ld ra, 56(sp)
    ld s5, 64(sp)
    ld s4, 72(sp)
    ld s2, 80(sp)
    ld s10, 88(sp)
    addi sp, sp, 96
    ret
  .size main, .-main
  .ident "SYSYC: (made by RRVM) 1.0.0"
