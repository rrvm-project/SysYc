  .file "tmp/matmul1.c"
  .option nopic
  .attribute unaligned_access, 0
  .attribute stack_align, 16
  .text
  .global a
  .section	.sbss, "aw", @nobits
  .align 2
  .type a, @object
  .size a, 4000000
a:
  .zero 4000000
  .global b
  .align 2
  .type b, @object
  .size b, 4000000
b:
  .zero 4000000
  .global c
  .align 2
  .type c, @object
  .size c, 4000000
c:
  .zero 4000000
  .global MAX
  .section	.sdata, "aw"
  .align 2
  .type MAX, @object
  .size MAX, 4
MAX:
  .word 2147483647
  .text
  .global main
  .align 1
  .type main, @function
main:
    addi sp, sp, -96
    sd s7, 0(sp)
    sd ra, 8(sp)
    sd s6, 16(sp)
    sd s4, 24(sp)
    sd s9, 32(sp)
    sd s3, 40(sp)
    sd s8, 48(sp)
    sd s5, 56(sp)
    sd s2, 64(sp)
    sd s1, 72(sp)
    sd s10, 80(sp)
    sd s11, 88(sp)
    la a3, a
    mv s2, x0
    li a7, 1000
    li s1, 4096
    addi a6, s1, -96
    j L_1
  L_2:
    mulw s1, s2, a6
    add a0, a3, s1
    addi sp, sp, -32
    sd a6, 0(sp)
    sd a7, 8(sp)
    sd a3, 16(sp)
    call getarray
    ld a6, 0(sp)
    ld a7, 8(sp)
    ld a3, 16(sp)
    addi sp, sp, 32
    bne a0, a7, L_3
    addiw s2, s2, 1
  L_1:
    blt s2, a7, L_2
    li a0, 23
    addi sp, sp, -32
    sd a3, 0(sp)
    sd a6, 8(sp)
    sd a0, 16(sp)
    sd a7, 24(sp)
    call _sysy_starttime
    ld a3, 0(sp)
    ld a6, 8(sp)
    ld a0, 16(sp)
    ld a7, 24(sp)
    addi sp, sp, 32
    la s11, b
    addi sp, sp, -32
    sd a3, 0(sp)
    sd a6, 8(sp)
    sd a7, 16(sp)
    call __create_threads
    ld a3, 0(sp)
    ld a6, 8(sp)
    ld a7, 16(sp)
    addi sp, sp, 32
    li a1, 3
    subw a0, a1, a0
    li a2, 250
    mulw s7, a0, a2
    mv s1, x0
    li a4, 250
    j L_4
  L_3:
  L_5:
    ld s7, 0(sp)
    ld ra, 8(sp)
    ld s6, 16(sp)
    ld s4, 24(sp)
    ld s9, 32(sp)
    ld s3, 40(sp)
    ld s8, 48(sp)
    ld s5, 56(sp)
    ld s2, 64(sp)
    ld s1, 72(sp)
    ld s10, 80(sp)
    ld s11, 88(sp)
    addi sp, sp, 96
    ret
  L_6:
    mulw s1, s7, a6
    add s4, s11, s1
    slliw s5, s7, 2
    mv s3, x0
    j L_7
  L_8:
    slliw s1, s3, 2
    add s2, s4, s1
    mulw s1, s3, a6
    add s1, a3, s1
    add s1, s1, s5
    lw s1, 0(s1)
    sw s1, 0(s2)
    addiw s3, s3, 1
  L_7:
    blt s3, a7, L_8
    addiw s7, s7, 1
    mv s1, s6
  L_4:
    addiw s6, s1, 1
    blt s1, a4, L_6
    addi sp, sp, -64
    sd a0, 0(sp)
    sd a2, 8(sp)
    sd a1, 16(sp)
    sd a3, 24(sp)
    sd a6, 32(sp)
    sd a4, 40(sp)
    sd a7, 48(sp)
    call __join_threads
    ld a0, 0(sp)
    ld a2, 8(sp)
    ld a1, 16(sp)
    ld a3, 24(sp)
    ld a6, 32(sp)
    ld a4, 40(sp)
    ld a7, 48(sp)
    addi sp, sp, 64
    la a5, c
    addi sp, sp, -64
    sd a2, 0(sp)
    sd a1, 8(sp)
    sd a5, 16(sp)
    sd a3, 24(sp)
    sd a6, 32(sp)
    sd a4, 40(sp)
    sd a7, 48(sp)
    call __create_threads
    ld a2, 0(sp)
    ld a1, 8(sp)
    ld a5, 16(sp)
    ld a3, 24(sp)
    ld a6, 32(sp)
    ld a4, 40(sp)
    ld a7, 48(sp)
    addi sp, sp, 64
    subw a0, a1, a0
    mulw s9, a0, a2
    mv s1, x0
    j L_9
  L_10:
    mulw s1, s9, a6
    add s8, a3, s1
    add s7, a5, s1
    mv s6, x0
    j L_11
  L_12:
    slliw s4, s6, 2
    mv s5, x0
    mv s3, x0
    j L_13
  L_14:
    slliw s1, s3, 2
    add s1, s8, s1
    lw s2, 0(s1)
    mulw s1, s3, a6
    add s1, s11, s1
    add s1, s1, s4
    lw s1, 0(s1)
    mulw s1, s2, s1
    addw s5, s5, s1
    addiw s3, s3, 1
  L_13:
    blt s3, a7, L_14
    add s1, s7, s4
    sw s5, 0(s1)
    addiw s6, s6, 1
  L_11:
    blt s6, a7, L_12
    addiw s9, s9, 1
    mv s1, s10
  L_9:
    addiw s10, s1, 1
    blt s1, a4, L_10
    addi sp, sp, -64
    sd a7, 0(sp)
    sd a0, 8(sp)
    sd a5, 16(sp)
    sd a6, 24(sp)
    sd a1, 32(sp)
    sd a4, 40(sp)
    sd a2, 48(sp)
    call __join_threads
    ld a7, 0(sp)
    ld a0, 8(sp)
    ld a5, 16(sp)
    ld a6, 24(sp)
    ld a1, 32(sp)
    ld a4, 40(sp)
    ld a2, 48(sp)
    addi sp, sp, 64
    addi sp, sp, -48
    sd a7, 0(sp)
    sd a5, 8(sp)
    sd a6, 16(sp)
    sd a1, 24(sp)
    sd a4, 32(sp)
    sd a2, 40(sp)
    call __create_threads
    ld a7, 0(sp)
    ld a5, 8(sp)
    ld a6, 16(sp)
    ld a1, 24(sp)
    ld a4, 32(sp)
    ld a2, 40(sp)
    addi sp, sp, 48
    subw a0, a1, a0
    mulw s7, a0, a2
    mv s2, x0
    li s1, 2147483648
    addi s6, s1, -1
    j L_15
  L_16:
    mulw s1, s7, a6
    add s4, a5, s1
    mv s2, x0
    mv s3, s6
    j L_17
  L_18:
    slliw s1, s2, 2
    add s1, s4, s1
    lw s1, 0(s1)
    blt s1, s3, L_19
    mv s1, s3
    j L_20
  L_19:
  L_20:
    addiw s2, s2, 1
    mv s3, s1
  L_17:
    blt s2, a7, L_18
    mv s2, x0
    j L_21
  L_22:
    slliw s1, s2, 2
    add s1, s4, s1
    sw s3, 0(s1)
    addiw s2, s2, 1
  L_21:
    blt s2, a7, L_22
    addiw s7, s7, 1
    mv s2, s5
  L_15:
    addiw s5, s2, 1
    blt s2, a4, L_16
    addi sp, sp, -32
    sd a6, 0(sp)
    sd a5, 8(sp)
    sd a7, 16(sp)
    call __join_threads
    ld a6, 0(sp)
    ld a5, 8(sp)
    ld a7, 16(sp)
    addi sp, sp, 32
    mv s6, x0
    j L_23
  L_24:
    mulw s1, s6, a6
    add s5, a5, s1
    slliw s4, s6, 2
    mv s3, x0
    j L_25
  L_26:
    slliw s1, s3, 2
    add s2, s5, s1
    mulw s1, s3, a6
    add s1, a5, s1
    add s1, s1, s4
    lw s1, 0(s1)
    negw s1, s1
    sw s1, 0(s2)
    addiw s3, s3, 1
  L_25:
    blt s3, a7, L_26
    addiw s6, s6, 1
  L_23:
    blt s6, a7, L_24
    mv s5, x0
    mv s4, x0
    j L_27
  L_28:
    mulw s1, s4, a6
    add s3, a5, s1
    mv s2, x0
    j L_29
  L_30:
    slliw s1, s2, 2
    add s1, s3, s1
    lw s1, 0(s1)
    addw s5, s5, s1
    addiw s2, s2, 1
  L_29:
    blt s2, a7, L_30
    addiw s4, s4, 1
  L_27:
    blt s4, a7, L_28
    li a0, 92
    call _sysy_stoptime
    mv a0, s5
    call putint
    mv a0, x0
    j L_5
  .size main, .-main
  .ident "SYSYC: (made by RRVM) 1.0.0"
 

.text
.global __create_threads
.global __join_threads

	SYS_clone = 220
	CLONE_VM = 256
	SIGCHLD = 17
	__create_threads:
		li a0, 3   # addi a0, a0, -1
		ble a0, zero, .ret_0
		mv a6, a0
		li a5, 0
		mv a1, sp
		li a2, 0
		li a3, 0
		li a4, 0
	.L0_builtin:
		li a0, (CLONE_VM | SIGCHLD)
		li a7, SYS_clone
		ecall
		bne a0, zero, .ret_i
		addi a5, a5, 1
		blt a5, a6, .L0_builtin
	.ret_n:
		mv a0, a6
		j .L1_builtin
	.ret_0:
		mv a0, zero
		j .L1_builtin
	.ret_i:
		mv a0, a5
	.L1_builtin:
		jr ra

	SYS_waitid = 95
	SYS_exit = 93
	P_ALL = 0
	WEXITED = 4
	__join_threads:
		li a1, 3
		sub a0, a1, a0
		li a1, 4 # new
		mv a4, a0
		addi a5, a1, -1
		beq a4, a5, .L2_builtin
		li a0, P_ALL
		li a1, 0
		li a2, 0
		li a3, WEXITED
		li a7, SYS_waitid
		ecall
	.L2_builtin:
		beq a4, zero, .L3_builtin
		li a0, 0
		li a7, SYS_exit
		ecall
	.L3_builtin:
		jr ra

	

	__fill_zero_words:
		ble a1, zero, .L8_builtin 
		addi a1, a1, -1
		slliw a1, a1, 2
		add a2, a1, a0  # 最后一次4字节
		addi a3, a2, -1
		andi a3, a3, -8 # 最后一次8字节
		andi a4, a0, 7
		beq a4, x0, .L4_builtin

		sw x0, 0(a0)
		addi a0, a0, 4

		.L4_builtin:
			bgtu a0, a3, .L7_builtin 

		.L5_builtin:
			sd x0, 0(a0)
			addi a0, a0, 8
			ble a0, a3, .L5_builtin

		.L7_builtin:
			bgtu a0, a2, .L8_builtin # 如果不够最后一次4字节
			sw x0, 0(a0)
			addi a0, a0, 4

		.L8_builtin:
			jr ra

		

