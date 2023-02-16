use core::arch::{ global_asm, asm };

use crate::constants::layout::{ TRAMPOLINE, TRAP_CONTEXT };
use crate::sbi::{SBI_CONSOLE_PUTCHAR, console_putchar, SBI_CONSOLE_GETCHAR, console_getchar};

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


fn sbi_handler(ctx: &mut TrapContext) {
    match ctx.x[17] {
        SBI_CONSOLE_PUTCHAR => console_putchar(ctx.x[10]),
        SBI_CONSOLE_GETCHAR => ctx.x[10] = console_getchar(),
        _ => unimplemented!()
    }
}


#[no_mangle]
pub fn trap_handler() -> ! {
    let ctx = unsafe{ (TRAP_CONTEXT as *mut TrapContext).as_mut().unwrap() };
    let scause = scause::read();
    match scause.cause() {
        scause::Trap::Exception(scause::Exception::UserEnvCall) => {
            panic!("U-mode/VU-mode env call from VS-mode?");
        },
        scause::Trap::Exception(scause::Exception::VirtualSupervisorEnvCall) => {
            sbi_handler(ctx);
            ctx.sepc += 4;
        }
        _ => unimplemented!()
    }
    unsafe{ switch_to_guest() }
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