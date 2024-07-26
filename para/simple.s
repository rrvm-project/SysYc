  .file "test.c"
  .option nopic
  .attribute unaligned_access, 0
  .attribute stack_align, 16
  .text
  .global A
  .section	.sbss, "aw", @nobits
  .align 2
  .type A, @object
  .size A, 400
A:
  .zero 400
  .global B
  .align 2
  .type B, @object
  .size B, 400
B:
  .zero 400

  .text
  .align 1
  .type main, @function
  .global main

main:
    addi sp, sp, -64
    sd s2, 0(sp)
    sd s1, 8(sp)
    sd ra, 16(sp)
    sd s11, 24(sp)
    sd s4, 32(sp)
    sd s6, 40(sp)
    sd s3, 48(sp)
    

    li a0, 4
    call __create_threads
    mv s11, a0
# no stack operation from here

    la s6, A
    slliw s3, a0, 2
    add s6, s3, s6

    addi a0, a0, 100

    sd a0, 0(s6)


# till here
    mv a0, s11
    li a1, 4
    call __join_threads

    la s6, A
    ld a0, 0(s6)
    call putint
    li a0, 10
    call putch
    ld a0, 4(s6)
    call putint
    li a0, 10
    call putch
    ld a0, 8(s6)
    call putint
    li a0, 10
    call putch
    ld a0, 12(s6)
    call putint
    li a0, 10
    call putch

    ld s2, 0(sp)
    ld s1, 8(sp)
    ld ra, 16(sp)
    ld s11, 24(sp)
    ld s4, 32(sp)
    ld s6, 40(sp)
    ld s3, 48(sp)

    addi sp, sp, 64
    li a0, 22
    ret
  .size main, .-main
  .ident "SYSYC: (made by RRVM) 1.0.0"

.text
.global __create_threads
.global __join_threads
/*
For system call ABI, see https://man7.org/linux/man-pages/man2/syscall.2.html
*/

/*
Raw system call interface varies on different architectures for clone,
but the manual page (https://man7.org/linux/man-pages/man2/clone.2.html) didn't
mention risc-v. By looking into the kernel source, I figure out that it is
long clone(unsigned long flags, void *stack,
                     int *parent_tid, unsigned long tls,
                     int *child_tid);

int __create_threads(int n) {
    --n;
    if (n <= 0) {
        return 0;
    }
    for (int i = 0; i < n; ++i) {
        int pid = clone(CLONE_VM | SIGCHLD, sp, 0, 0, 0);
        if (pid != 0) {
            return i;
        }
    }
    return n;
}
*/
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
.L0:
    li a0, (CLONE_VM | SIGCHLD)
    li a7, SYS_clone
    ecall
    bne a0, zero, .ret_i
    addi a5, a5, 1
    blt a5, a6, .L0
.ret_n:
    mv a0, a6
    j .L1
.ret_0:
    mv a0, zero
    j .L1
.ret_i:
    mv a0, a5
.L1:
    jr ra

/*
Note that it depends on an inconsistent feature between linux and POSIX,
see section BUGS at https://man7.org/linux/man-pages/man2/wait.2.html
But since it already depends on so many features of linux, like the raw
syscall number, so never mind.
void __join_threads(int i, int n) {
    --n;
    if (i != n) {
        waitid(P_ALL, 0, NULL, WEXITED);
    }
    if (i != 0) {
        _exit(0);
    }
}
*/
SYS_waitid = 95
SYS_exit = 93
P_ALL = 0
WEXITED = 4
__join_threads:
    mv a4, a0
    addi a5, a1, -1
    beq a4, a5, .L2
    li a0, P_ALL
    li a1, 0
    li a2, 0
    li a3, WEXITED
    li a7, SYS_waitid
    ecall
.L2:
    beq a4, zero, .L3
    li a0, 0
    li a7, SYS_exit
    ecall
.L3:
    jr ra
