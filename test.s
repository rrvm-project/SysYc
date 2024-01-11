	.file	"test.c"
	.text
	.section	.text.startup,"ax",@progbits
	.p2align 4
	.globl	main
	.type	main, @function
main:
.LFB23:
	.cfi_startproc
	endbr64
	subq	$8, %rsp
	.cfi_def_cfa_offset 16
	movq	8(%rdi), %rdi
	xorl	%eax, %eax
	call	atoi@PLT
	addq	$8, %rsp
	.cfi_def_cfa_offset 8
	movl	%eax, %edx
	cltq
	imulq	$-368140053, %rax, %rax
	shrq	$32, %rax
	addl	%edx, %eax
	sarl	$31, %edx
	sarl	$5, %eax
	subl	%edx, %eax
	ret
	.cfi_endproc
.LFE23:
	.size	main, .-main
	.ident	"GCC: (Ubuntu 11.4.0-1ubuntu1~22.04) 11.4.0"
	.section	.note.GNU-stack,"",@progbits
	.section	.note.gnu.property,"a"
	.align 8
	.long	1f - 0f
	.long	4f - 1f
	.long	5
0:
	.string	"GNU"
1:
	.align 8
	.long	0xc0000002
	.long	3f - 2f
2:
	.long	0x3
3:
	.align 8
4:
