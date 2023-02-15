use core::arch::{ global_asm, asm };

use crate::constants::layout::TRAMPOLINE;

use riscv::register::{ stvec, sscratch, scause, sepc, stval, sie };
use riscv::register::scause::{ Trap, Exception};

mod context;
pub use context::TrapContext;

global_asm!(include_str!("trap.S"));

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

#[no_mangle]
/// set the new addr of __restore asm function in TRAMPOLINE page,
/// set the reg a0 = trap_cx_ptr, reg a1 = phy addr of usr page table,
/// finally, jump to new addr of __restore asm function
pub fn trap_return() -> ! {
    set_user_trap_entry();
    // let user_satp = hypervisor.current_user_token();
    // extern "C" {
    //     fn __alltraps();
    //     fn __restore();
    // }
    // let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    // unsafe {
    //     asm!(
    //         "fence.i",
    //         "jr {restore_va}",             // jump to new addr of __restore asm function
    //         restore_va = in(reg) restore_va,
    //         in("a0") trap_cx_ptr,      // a0 = virt addr of Trap Context
    //         in("a1") user_satp,        // a1 = phy addr of usr page table
    //         options(noreturn)
    //     );
    // }
    unreachable!()
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