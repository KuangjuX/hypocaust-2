.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .globl __alltraps_k
    .globl __restore_k
    .align 2

__alltraps:
    # 交换 sp 和 sscratch 寄存器,此时 sp 寄存器是 TrapContext 的地址,
    # sp 是 guest 地址
    csrrw sp, sscratch, sp
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they have been saved in TrapContext
    csrr t0, sstatus
    csrr t1, sepc
    # 将 sstatus 和 sepc 存储在 32*8(trap ctx) 和 33*8(trap ctx) 的位置
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # 将 guest stack 寄存器保存  
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # 加载 hgatp 到 t0 寄存器
    ld t0, 34*8(sp)
    # 加载 hypervisor trap handler 地址到 t1 寄存器
    ld t1, 36*8(sp)
    # 存储 hstatus 寄存器
    csrr t0, hstatus
    sd t0, 37*8(sp)
    # 切换栈寄存器
    ld sp, 35*8(sp)
    # 由 VS guest 跳转到 HS hypervisor, 不需要切换页表
    # 跳转到 trap 处理函数
    jr t1

# a0: trap context addr; a1: hgatp
__restore:
    # 注意,此时不需要进行页表切换,此时仍在 HS 态,使用 satp 进行 MMU 地址翻译
    # 将 sscratch 和 sp 全部写入 a0(Trap Context)
    csrw sscratch, a0
    mv sp, a0
    # now sp points to TrapContext in user space, start restoring based on it
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # 恢复 hstatus 寄存器
    ld t0, 37*8(sp)
    csrw hstatus, t0
    # restore general purpose registers except x0/sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # back to user stack
    ld sp, 2*8(sp)
    sret

    .align 2
__alltraps_k:
    addi sp, sp, -34*8 
    sd x1, 1*8(sp)
    sd x3, 3*8(sp)
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    mv a0, sp
    csrr t2, sscratch
    jalr t2

__restore_k:
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    addi sp, sp, 34*8
    sret