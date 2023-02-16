use core::arch::{ global_asm, asm };

use crate::constants::layout::{ TRAMPOLINE, TRAP_CONTEXT };

use riscv::register::{ stvec, sscratch, scause, sepc, stval, sie };
use riscv::register::scause::{ Trap, Exception };

mod context;
pub use context::TrapContext;

global_asm!(include_str!("trap.S"));

/// initialize CSR `stvec` as the entry of `__alltraps`
pub fn init() {
    set_kernel_trap_entry();
}

/// enable timer interrupt in sie CSR
pub fn enable_timer_interrupt() {
    unsafe { sie::set_stimer(); }
}

pub fn disable_timer_interrupt() {
    unsafe{ sie::clear_stimer(); }
}

fn set_kernel_trap_entry() {
    extern "C" {
        fn __alltraps();
        fn __alltraps_k();
    }
    let __alltraps_k_va = __alltraps_k as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        stvec::write(__alltraps_k_va, stvec::TrapMode::Direct);
        sscratch::write(trap_from_kernel as usize);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, stvec::TrapMode::Direct);
    }
}

// #[link_section = ".text.trampoline"]
// #[export_name = "vstrap_entry"]
// #[naked]
// pub unsafe extern "C" fn vstrap_entry() -> ! {
//     asm!(
//         ".align 4",
//         // 保存栈指针到 `sscratch` 寄存器
//         "csrw sscratch, sp",
//         // 设置栈指针
//         "li sp, {trap_context}",
//         // 由 guest 切换到 hypervisor, 存储上下文
//         "sd ra, 1*8(sp)",
//         "sd gp, 3*8(sp)",
//         "sd tp, 4*8(sp)",
//         "sd t0, 5*8(sp)",
//         "sd t1, 6*8(sp)",
//         "sd t2, 7*8(sp)",
//         "sd s0, 8*8(sp)",
//         "sd s1, 9*8(sp)",
//         "sd a0, 10*8(sp)",
//         "sd a1, 11*8(sp)",
//         "sd a2, 12*8(sp)",
//         "sd a3, 13*8(sp)",
//         "sd a4, 14*8(sp)",
//         "sd a5, 15*8(sp)",
//         "sd a6, 16*8(sp)",
//         "sd a7, 17*8(sp)",
//         "sd s2, 18*8(sp)",
//         "sd s3, 19*8(sp)",
//         "sd s4, 20*8(sp)",
//         "sd s5, 21*8(sp)",
//         "sd s6, 22*8(sp)",
//         "sd s7, 23*8(sp)",
//         "sd s8, 24*8(sp)",
//         "sd s9, 25*8(sp)",
//         "sd s10, 26*8(sp)",
//         "sd s11, 27*8(sp)",
//         "sd t3, 28*8(sp)",
//         "sd t4, 29*8(sp)",
//         "sd t5, 30*8(sp)",
//         "sd t6, 31*8(sp)",
        
//         // 跳转到异常处理函数
//         "jal ra, trap_handler",
//         // 恢复栈指针
//         "li sp, {trap_context}",

//         // 恢复上下文
//         "ld ra, 1*8(sp)",
//         "ld gp, 3*8(sp)",
//         "ld tp, 4*8(sp)",
//         "ld t0, 5*8(sp)",
//         "ld t1, 6*8(sp)",
//         "ld t2, 7*8(sp)",
//         "ld s0, 8*8(sp)",
//         "ld s1, 9*8(sp)",
//         "ld a0, 10*8(sp)",
//         "ld a1, 11*8(sp)",
//         "ld a2, 12*8(sp)",
//         "ld a3, 13*8(sp)",
//         "ld a4, 14*8(sp)",
//         "ld a5, 15*8(sp)",
//         "ld a6, 16*8(sp)",
//         "ld a7, 17*8(sp)",
//         "ld s2, 18*8(sp)",
//         "ld s3, 19*8(sp)",
//         "ld s4, 20*8(sp)",
//         "ld s5, 21*8(sp)",
//         "ld s6, 22*8(sp)",
//         "ld s7, 23*8(sp)",
//         "ld s8, 24*8(sp)",
//         "ld s9, 25*8(sp)",
//         "ld s10, 26*8(sp)",
//         "ld s11, 27*8(sp)",
//         "ld t3, 28*8(sp)",
//         "ld t4, 29*8(sp)",
//         "ld t5, 30*8(sp)",
//         "ld t6, 31*8(sp)",

//         // 恢复栈指针并且返回
//         "csrr sp. sscratch",
//         "sret",

//         trap_context = const TRAP_CONTEXT,
//         options(noreturn),
//     );
// }

#[no_mangle]
pub fn trap_handler() -> ! {
    let trap_ctx = unsafe{ (TRAP_CONTEXT as *mut TrapContext).as_mut().unwrap() };
    let scause = scause::read();
    panic!("trap handler sepc: {:#x}, stval: {:#x}, scause: {:?}", trap_ctx.sepc, stval::read(), scause.cause());
}

#[no_mangle]
/// set the new addr of __restore asm function in TRAMPOLINE page,
/// set the reg a0 = trap_cx_ptr, reg a1 = phy addr of usr page table,
/// finally, jump to new addr of __restore asm function
pub unsafe fn switch_to_guest() -> ! {
    set_user_trap_entry();
    // 获取上下文切换环境
    let trap_ctx = (TRAP_CONTEXT as *mut TrapContext).as_ref().unwrap();

    // hgatp: set page table for guest physical address translation
    if riscv::register::hgatp::read().bits() != trap_ctx.hgatp {
        let hgatp = riscv::register::hgatp::Hgatp::from_bits(trap_ctx.hgatp);
        hgatp.write(); 
        core::arch::riscv64::hfence_gvma_all();
        assert_eq!(hgatp.bits(), riscv::register::hgatp::read().bits());
    }
    // hstatus: handle SPV change the virtualization mode to 0 after sret
    riscv::register::hstatus::set_spv();

    // sstatus: handle SPP to 1 to change the privilege to S-Mode after sret
    riscv::register::sstatus::set_spp(riscv::register::sstatus::SPP::Supervisor);
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",             // jump to new addr of __restore asm function
            restore_va = in(reg) restore_va,
            in("a0") TRAP_CONTEXT,           // a0 = virt addr of Trap Context
            in("a1") trap_ctx.hgatp,        // a1 = phy addr of usr page table
            options(noreturn)
        );
    }
}


#[no_mangle]
pub fn trap_from_kernel(_trap_cx: &TrapContext) -> ! {
    let scause= scause::read();
    let sepc = sepc::read();
    match scause.cause() {
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::LoadFault) | Trap::Exception(Exception::LoadPageFault)=> {
            let stval = stval::read();
            panic!("scause: {:?}, sepc: {:#x}, stval: {:#x}", scause.cause(), _trap_cx.sepc, stval);
        },
        _ => { panic!("scause: {:?}, spec: {:#x}, stval: {:#x}", scause.cause(), sepc, stval::read())}
    }
}