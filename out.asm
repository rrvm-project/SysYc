  .file "./project-eval/testcases/performance/instruction-combining-3.sy"
  .option nopic
  .attribute unaligned_access, 0
  .attribute stack_align, 16
  .text
  .global loopCount
  .section	.sbss, "aw", @nobits
  .align 2
  .type loopCount, @object
  .size loopCount, 4
loopCount:
  .zero 4
  .text
  .global main
  .align 1
  .type func, @function
func:
    addi sp, sp, -16
    sd s1, 0(sp)
    li s1, 10000
    addw s1, a0, s1
    subw a1, s1, a1
    mv a0, a1
    ld s1, 0(sp)
    addi sp, sp, 16
    ret
  .size func, .-func
  .align 1
  .type main, @function
main:
    addi sp, sp, -80
    sd s8, 0(sp)
    sd s4, 8(sp)
    sd s5, 16(sp)
    sd s2, 24(sp)
    sd ra, 32(sp)
    sd s9, 40(sp)
    sd s7, 48(sp)
    sd s6, 56(sp)
    sd s1, 64(sp)
    sd s3, 72(sp)
    la s9, loopCount
    call getint
    sw a0, 0(s9)
    li s5, 0
    li a0, 10015
    call _sysy_starttime
    li s9, 0
    j L_1
  L_2:
    li s1, 0
    li s4, 0
    j L_3
  L_4:
    mv s1, s7
  L_3:
    addw s7, s1, s5
    slti s6, s4, 60
    addiw s4, s4, 1
    li s2, 2290649225
    li s8, 2147549182
    mul s3, s2, s1
    srliw s2, s1, 31
    sub s3, s3, s2
    srai s3, s3, 37
    addw s3, s3, s2
    li s2, 536854529
    addw s3, s9, s3
    mul s1, s8, s3
    srliw s8, s3, 31
    sub s1, s1, s8
    srai s1, s1, 60
    addw s1, s1, s8
    mul s1, s1, s2
    subw s1, s3, s1
    bne s6, x0, L_4
    mv s9, s1
    mv s5, a0
  L_1:
    addiw a0, s5, 1
    la s8, loopCount
    lw s7, 0(s8)
    slt s6, s5, s7
    bne s6, x0, L_2
    li a0, 10030
    call _sysy_stoptime
    mv a0, s9
    call putint
    li a0, 10
    call putch
    li a0, 0
    ld s8, 0(sp)
    ld s4, 8(sp)
    ld s5, 16(sp)
    ld s2, 24(sp)
    ld ra, 32(sp)
    ld s9, 40(sp)
    ld s7, 48(sp)
    ld s6, 56(sp)
    ld s1, 64(sp)
    ld s3, 72(sp)
    addi sp, sp, 80
    ret
  .size main, .-main
 

.text
.global __create_threads
.global __join_threads

	SYS_clone = 220
	CLONE_VM = 256
	SIGCHLD = 17
	__create_threads:
		addi a0, a0, -1
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

		


 .ident "SYSYC: (made by RRVM) 1.0.0"
