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
  .type multiply_calc_coef, @function
multiply_calc_coef:
    addi sp, sp, -96
    sd s6, 0(sp)
    sd s8, 8(sp)
    sd s10, 16(sp)
    sd s3, 24(sp)
    sd s9, 32(sp)
    sd s4, 40(sp)
    sd s1, 48(sp)
    sd s5, 56(sp)
    sd s2, 64(sp)
    sd s7, 72(sp)
    sd s11, 80(sp)
    sd ra, 88(sp)
    mv s8, a1
    mv s10, a0
    li s1, 0
    beq s8, s1, L_1
    li s11, 1
    beq s8, s11, L_2
    srliw s1, s8, 31
    sub a1, s8, s1
    srai a1, a1, 1
    addw a1, a1, s1
    mv a0, s10
    call multiply_calc_coef
    lw s3, 0(s10)
    li s1, 4
    add s9, s10, s1
    lw s1, 0(s9)
    mul s2, s3, x0
    add s1, s1, s2
    addw s2, s3, s3
    addw s3, s1, s1
    li s1, 2309898240
    addi s7, s1, 136
    mul s4, s7, s2
    srliw s1, s2, 31
    sub s4, s4, s1
    srai s4, s4, 61
    addw s4, s4, s1
    li s1, 998244352
    addi s6, s1, 1
    mul s4, s4, s6
    subw s4, s2, s4
    mul s5, s7, s3
    srliw s1, s3, 31
    sub s5, s5, s1
    srai s5, s5, 61
    addw s5, s5, s1
    mul s5, s5, s6
    subw s5, s3, s5
    srliw s1, s8, 31
    add s1, s1, s8
    andi s1, s1, -2
    sub s1, s8, s1
    beq s1, s11, L_3
    sw s4, 0(s10)
    sw s5, 0(s9)
    j L_4
  L_1:
    li s1, 4
    add s1, s10, s1
    sw x0, 0(s1)
    sw x0, 0(s10)
  L_4:
    ld s6, 0(sp)
    ld s8, 8(sp)
    ld s10, 16(sp)
    ld s3, 24(sp)
    ld s9, 32(sp)
    ld s4, 40(sp)
    ld s1, 48(sp)
    ld s5, 56(sp)
    ld s2, 64(sp)
    ld s7, 72(sp)
    ld s11, 80(sp)
    ld ra, 88(sp)
    addi sp, sp, 96
    ret
  L_2:
    li s1, 1
    sw s1, 0(s10)
    li s1, 4
    add s1, s10, s1
    sw x0, 0(s1)
    j L_4
  L_3:
    addiw s1, s4, 1
    mul s3, s7, s1
    srliw s2, s1, 31
    sub s3, s3, s2
    srai s3, s3, 61
    addw s3, s3, s2
    mul s3, s3, s6
    subw s3, s1, s3
    mul s1, s7, s5
    srliw s2, s5, 31
    sub s1, s1, s2
    srai s1, s1, 61
    addw s1, s1, s2
    mul s1, s1, s6
    subw s1, s5, s1
    sw s3, 0(s10)
    sw s1, 0(s9)
    j L_4
  .size multiply_calc_coef, .-multiply_calc_coef
  .align 1
  .type power, @function
power:
    addi sp, sp, -64
    sd s5, 0(sp)
    sd s3, 8(sp)
    sd s1, 16(sp)
    sd s7, 24(sp)
    sd s2, 32(sp)
    sd ra, 40(sp)
    sd s4, 48(sp)
    sd s6, 56(sp)
    mv s5, a1
    mv s6, a0
    li s1, 0
    addi sp, sp, -16
    mv s3, sp
    addi sp, sp, -16
    mv s7, sp
    beq s5, s1, L_5
    srliw s1, s5, 31
    sub a1, s5, s1
    srai a1, a1, 1
    addw a1, a1, s1
    mv a0, s6
    call power
    mv s2, a0
    mv a0, s3
    mv a1, s2
    call multiply_calc_coef
    lw s1, 0(s3)
    mul s2, s1, s2
    li s4, 4
    add s1, s3, s4
    lw s1, 0(s1)
    add s2, s2, s1
    li s1, 998244352
    addi s3, s1, 1
    rem s2, s2, s3
    srliw s1, s5, 31
    add s1, s1, s5
    andi s1, s1, -2
    sub s1, s5, s1
    li s5, 1
    beq s1, s5, L_6
    addi sp, sp, 32
    mv a0, s2
    j L_7
  L_5:
    addi sp, sp, 32
    li a0, 1
  L_7:
    ld s5, 0(sp)
    ld s3, 8(sp)
    ld s1, 16(sp)
    ld s7, 24(sp)
    ld s2, 32(sp)
    ld ra, 40(sp)
    ld s4, 48(sp)
    ld s6, 56(sp)
    addi sp, sp, 64
    ret
  L_6:
    mv a0, s7
    mv a1, s6
    call multiply_calc_coef
    lw s1, 0(s7)
    mul s2, s1, s2
    add s1, s7, s4
    lw s1, 0(s1)
    add s1, s2, s1
    rem a0, s1, s3
    addi sp, sp, 32
    j L_7
  .size power, .-power
  .align 1
  .type fft, @function
fft:
    addi sp, sp, -96
    sd s7, 0(sp)
    sd s9, 8(sp)
    sd s10, 16(sp)
    sd s4, 24(sp)
    sd ra, 32(sp)
    sd s8, 40(sp)
    sd s3, 48(sp)
    sd s1, 56(sp)
    sd s6, 64(sp)
    sd s5, 72(sp)
    sd s11, 80(sp)
    sd s2, 88(sp)
    mv t3, a3
    mv t2, a1
    mv a7, a0
    li s1, 1
    addi sp, sp, -16
    mv s7, sp
    addi sp, sp, -16
    mv t1, sp
    addi sp, sp, -16
    mv t0, sp
    beq a2, s1, L_8
    la s6, temp
    li s1, 4
    add s8, s7, s1
    add a6, t1, s1
    add a5, t0, s1
    mv s4, x0
    li s5, 0
    li s1, 998244352
    addi a4, s1, 1
    li s10, 1
    li s1, 2309898240
    addi s11, s1, 136
    j L_9
  L_8:
    addi sp, sp, 48
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
    addw s1, s4, t2
    slliw s1, s1, 2
    add s1, a7, s1
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
    addw s1, s4, t2
    slliw s1, s1, 2
    add s1, a7, s1
    lw s1, 0(s1)
    sw s1, 0(s2)
  L_13:
    addiw s4, s4, 1
  L_9:
    blt s4, a2, L_11
    mv s3, x0
    j L_14
  L_15:
    addw s2, t2, s8
    slliw s1, s2, 2
    add s6, a7, s1
    lw s5, 0(s6)
    addw s1, s2, s9
    slliw s1, s1, 2
    add s7, a7, s1
    lw a1, 0(s7)
    addi sp, sp, -64
    sd a7, 0(sp)
    sd a4, 8(sp)
    sd t2, 16(sp)
    sd t0, 24(sp)
    sd t1, 32(sp)
    sd a6, 40(sp)
    sd a5, 48(sp)
    sd t3, 56(sp)
    mv a0, t1
    call multiply_calc_coef
    ld a7, 0(sp)
    ld a4, 8(sp)
    ld t2, 16(sp)
    ld t0, 24(sp)
    ld t1, 32(sp)
    ld a6, 40(sp)
    ld a5, 48(sp)
    ld t3, 56(sp)
    addi sp, sp, 64
    lw s1, 0(t1)
    mul s1, s1, s10
    lw s2, 0(a6)
    add s1, s1, s2
    rem s3, s1, a4
    addw s2, s5, s3
    mul s1, s11, s2
    srliw s4, s2, 31
    sub s1, s1, s4
    srai s1, s1, 61
    addw s1, s1, s4
    mul s1, s1, a4
    subw s1, s2, s1
    sw s1, 0(s6)
    subw s1, s5, s3
    addw s2, s1, a4
    mul s1, s11, s2
    srliw s3, s2, 31
    sub s1, s1, s3
    srai s1, s1, 61
    addw s1, s1, s3
    mul s1, s1, a4
    subw s1, s2, s1
    sw s1, 0(s7)
    addi sp, sp, -64
    sd t1, 0(sp)
    sd a7, 8(sp)
    sd a5, 16(sp)
    sd t0, 24(sp)
    sd a6, 32(sp)
    sd a4, 40(sp)
    sd t3, 48(sp)
    sd t2, 56(sp)
    mv a0, t0
    mv a1, t3
    call multiply_calc_coef
    ld t1, 0(sp)
    ld a7, 8(sp)
    ld a5, 16(sp)
    ld t0, 24(sp)
    ld a6, 32(sp)
    ld a4, 40(sp)
    ld t3, 48(sp)
    ld t2, 56(sp)
    addi sp, sp, 64
    lw s1, 0(t0)
    mul s1, s1, s10
    lw s2, 0(a5)
    add s1, s1, s2
    rem s10, s1, a4
    addiw s8, s8, 1
  L_16:
    blt s8, s9, L_15
    addi sp, sp, 48
    mv a0, x0
  L_10:
    ld s7, 0(sp)
    ld s9, 8(sp)
    ld s10, 16(sp)
    ld s4, 24(sp)
    ld ra, 32(sp)
    ld s8, 40(sp)
    ld s3, 48(sp)
    ld s1, 56(sp)
    ld s6, 64(sp)
    ld s5, 72(sp)
    ld s11, 80(sp)
    ld s2, 88(sp)
    addi sp, sp, 96
    ret
  L_17:
    addw s1, t2, s3
    slliw s1, s1, 2
    add s2, a7, s1
    slliw s1, s3, 2
    add s1, s6, s1
    lw s1, 0(s1)
    sw s1, 0(s2)
    addiw s3, s3, 1
  L_14:
    blt s3, a2, L_17
    srliw s1, a2, 31
    sub s9, a2, s1
    srai s9, s9, 1
    addw s9, s9, s1
    addi sp, sp, -64
    sd a7, 0(sp)
    sd t2, 8(sp)
    sd t0, 16(sp)
    sd t1, 24(sp)
    sd a5, 32(sp)
    sd t3, 40(sp)
    sd a6, 48(sp)
    sd a4, 56(sp)
    mv a0, s7
    mv a1, t3
    call multiply_calc_coef
    ld a7, 0(sp)
    ld t2, 8(sp)
    ld t0, 16(sp)
    ld t1, 24(sp)
    ld a5, 32(sp)
    ld t3, 40(sp)
    ld a6, 48(sp)
    ld a4, 56(sp)
    addi sp, sp, 64
    lw s1, 0(s7)
    mul s2, s1, t3
    lw s1, 0(s8)
    add s1, s2, s1
    rem s1, s1, a4
    addi sp, sp, -64
    sd a7, 0(sp)
    sd t2, 8(sp)
    sd t0, 16(sp)
    sd t1, 24(sp)
    sd a5, 32(sp)
    sd t3, 40(sp)
    sd a6, 48(sp)
    sd a4, 56(sp)
    mv a0, a7
    mv a1, t2
    mv a2, s9
    mv a3, s1
    call fft
    ld a7, 0(sp)
    ld t2, 8(sp)
    ld t0, 16(sp)
    ld t1, 24(sp)
    ld a5, 32(sp)
    ld t3, 40(sp)
    ld a6, 48(sp)
    ld a4, 56(sp)
    addi sp, sp, 64
    addw a1, t2, s9
    addi sp, sp, -64
    sd a6, 0(sp)
    sd a7, 8(sp)
    sd t2, 16(sp)
    sd t0, 24(sp)
    sd t1, 32(sp)
    sd a5, 40(sp)
    sd t3, 48(sp)
    sd a4, 56(sp)
    mv a0, a7
    mv a2, s9
    mv a3, s1
    call fft
    ld a6, 0(sp)
    ld a7, 8(sp)
    ld t2, 16(sp)
    ld t0, 24(sp)
    ld t1, 32(sp)
    ld a5, 40(sp)
    ld t3, 48(sp)
    ld a4, 56(sp)
    addi sp, sp, 64
    mv s8, x0
    j L_16
  .size fft, .-fft
  .align 1
  .type main, @function
main:
    addi sp, sp, -96
    sd s2, 0(sp)
    sd s6, 8(sp)
    sd s4, 16(sp)
    sd s8, 24(sp)
    sd s10, 32(sp)
    sd ra, 40(sp)
    sd s9, 48(sp)
    sd s1, 56(sp)
    sd s5, 64(sp)
    sd s11, 72(sp)
    sd s3, 80(sp)
    sd s7, 88(sp)
    la t1, a
    addi sp, sp, -16
    sd t1, 0(sp)
    mv a0, t1
    call getarray
    ld t1, 0(sp)
    addi sp, sp, 16
    mv s1, a0
    la s11, b
    addi sp, sp, -16
    sd t1, 0(sp)
    mv a0, s11
    call getarray
    ld t1, 0(sp)
    addi sp, sp, 16
    addi sp, sp, -16
    mv s10, sp
    addi sp, sp, -16
    mv t0, sp
    addw s1, s1, a0
    addiw a7, s1, -1
    li a6, 1
    j L_18
  L_19:
    slliw a6, a6, 1
  L_18:
    blt a6, a7, L_19
    li a5, 998244352
    addi s8, a5, 0
    divw s7, s8, a6
    li s9, 3
    addi sp, sp, -48
    sd t0, 0(sp)
    sd t1, 8(sp)
    sd a6, 16(sp)
    sd a5, 24(sp)
    sd a7, 32(sp)
    mv a0, s9
    mv a1, s7
    call power
    ld t0, 0(sp)
    ld t1, 8(sp)
    ld a6, 16(sp)
    ld a5, 24(sp)
    ld a7, 32(sp)
    addi sp, sp, 48
    mv s1, a0
    addi sp, sp, -48
    sd t0, 0(sp)
    sd t1, 8(sp)
    sd a6, 16(sp)
    sd a5, 24(sp)
    sd a7, 32(sp)
    mv a0, t1
    mv a1, x0
    mv a2, a6
    mv a3, s1
    call fft
    ld t0, 0(sp)
    ld t1, 8(sp)
    ld a6, 16(sp)
    ld a5, 24(sp)
    ld a7, 32(sp)
    addi sp, sp, 48
    addi sp, sp, -48
    sd t0, 0(sp)
    sd t1, 8(sp)
    sd a6, 16(sp)
    sd a5, 24(sp)
    sd a7, 32(sp)
    mv a0, s11
    mv a1, x0
    mv a2, a6
    mv a3, s1
    call fft
    ld t0, 0(sp)
    ld t1, 8(sp)
    ld a6, 16(sp)
    ld a5, 24(sp)
    ld a7, 32(sp)
    addi sp, sp, 48
    li s6, 4
    add s5, s10, s6
    mv s4, x0
    addi a4, a5, 1
    j L_20
  L_21:
    slliw s1, s4, 2
    add s3, t1, s1
    lw s2, 0(s3)
    add s1, s11, s1
    lw a1, 0(s1)
    addi sp, sp, -48
    sd a7, 0(sp)
    sd a4, 8(sp)
    sd t1, 16(sp)
    sd t0, 24(sp)
    sd a5, 32(sp)
    sd a6, 40(sp)
    mv a0, s10
    call multiply_calc_coef
    ld a7, 0(sp)
    ld a4, 8(sp)
    ld t1, 16(sp)
    ld t0, 24(sp)
    ld a5, 32(sp)
    ld a6, 40(sp)
    addi sp, sp, 48
    lw s1, 0(s10)
    mul s2, s1, s2
    lw s1, 0(s5)
    add s1, s2, s1
    rem s1, s1, a4
    sw s1, 0(s3)
    addiw s4, s4, 1
  L_20:
    blt s4, a6, L_21
    subw a1, s8, s7
    addi sp, sp, -48
    sd t0, 0(sp)
    sd a5, 8(sp)
    sd a4, 16(sp)
    sd t1, 24(sp)
    sd a6, 32(sp)
    sd a7, 40(sp)
    mv a0, s9
    call power
    ld t0, 0(sp)
    ld a5, 8(sp)
    ld a4, 16(sp)
    ld t1, 24(sp)
    ld a6, 32(sp)
    ld a7, 40(sp)
    addi sp, sp, 48
    mv a3, a0
    addi sp, sp, -48
    sd t0, 0(sp)
    sd t1, 8(sp)
    sd a5, 16(sp)
    sd a4, 24(sp)
    sd a6, 32(sp)
    sd a7, 40(sp)
    mv a0, t1
    mv a1, x0
    mv a2, a6
    call fft
    ld t0, 0(sp)
    ld t1, 8(sp)
    ld a5, 16(sp)
    ld a4, 24(sp)
    ld a6, 32(sp)
    ld a7, 40(sp)
    addi sp, sp, 48
    add s6, t0, s6
    mv s5, x0
    addi s4, a5, -1
    j L_22
  L_23:
    slliw s1, s5, 2
    add s3, t1, s1
    lw s2, 0(s3)
    addi sp, sp, -48
    sd a4, 0(sp)
    sd t0, 8(sp)
    sd t1, 16(sp)
    sd a6, 24(sp)
    sd a7, 32(sp)
    mv a0, a6
    mv a1, s4
    call power
    ld a4, 0(sp)
    ld t0, 8(sp)
    ld t1, 16(sp)
    ld a6, 24(sp)
    ld a7, 32(sp)
    addi sp, sp, 48
    mv a1, a0
    addi sp, sp, -48
    sd a4, 0(sp)
    sd t0, 8(sp)
    sd t1, 16(sp)
    sd a6, 24(sp)
    sd a7, 32(sp)
    mv a0, t0
    call multiply_calc_coef
    ld a4, 0(sp)
    ld t0, 8(sp)
    ld t1, 16(sp)
    ld a6, 24(sp)
    ld a7, 32(sp)
    addi sp, sp, 48
    lw s1, 0(t0)
    mul s1, s1, s2
    lw s2, 0(s6)
    add s1, s1, s2
    rem s1, s1, a4
    sw s1, 0(s3)
    addiw s5, s5, 1
  L_22:
    blt s5, a6, L_23
    mv a0, a7
    mv a1, t1
    call putarray
    addi sp, sp, 32
    mv a0, x0
    ld s2, 0(sp)
    ld s6, 8(sp)
    ld s4, 16(sp)
    ld s8, 24(sp)
    ld s10, 32(sp)
    ld ra, 40(sp)
    ld s9, 48(sp)
    ld s1, 56(sp)
    ld s5, 64(sp)
    ld s11, 72(sp)
    ld s3, 80(sp)
    ld s7, 88(sp)
    addi sp, sp, 96
    ret
  .size main, .-main
  .ident "SYSYC: (made by RRVM) 1.0.0"
