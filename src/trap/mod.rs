use core::arch::{ global_asm, asm };

use crate::constants::layout::{ TRAMPOLINE, TRAP_CONTEXT };
use crate::guest::pmap::{two_stage_translation, decode_inst_at_addr};
use crate::sbi::leagcy::SBI_SET_TIMER;
use crate::hypervisor::{HOST_VMM, is_plic_access};
use crate::{ VmmError, VmmResult };
use crate::sbi::{SBI_CONSOLE_PUTCHAR, console_putchar, SBI_CONSOLE_GETCHAR, console_getchar, set_timer};

use riscv::register::{ stvec, sscratch, scause, sepc, stval, sie, hgatp, vsatp, htval, htinst };
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


fn sbi_handler(ctx: &mut TrapContext) -> VmmResult {
    match ctx.x[17] {
        SBI_CONSOLE_PUTCHAR => console_putchar(ctx.x[10]),
        SBI_CONSOLE_GETCHAR => ctx.x[10] = console_getchar(),
        SBI_SET_TIMER => set_timer(ctx.x[10]),
        _ => { return Err(VmmError::Unimplemented) }
    }
    Ok(())
}

fn privileged_inst_handler(_ctx: &mut TrapContext) -> VmmResult {
    todo!()
}


pub fn guest_page_fault_handler(ctx: &mut TrapContext) -> VmmResult {
    let addr = htval::read() << 2;
    if is_plic_access(addr) {
        // panic!("PLIC Access!");
        let inst = htinst::read();
        if inst == 0 {
            // If htinst does not provide information about the trap,
            // we must read the instruction from guest's memory manually
            let inst_addr = ctx.sepc;
            let host_vmm = unsafe{ HOST_VMM.get().unwrap().lock() };
            let gpm = &host_vmm.guests.get(&host_vmm.guest_id).unwrap().gpm;
            if let Some(host_inst_addr) = two_stage_translation(
                host_vmm.guest_id, 
                inst_addr, 
                vsatp::read().bits(), 
                gpm
            ) {
                let (len, inst) = decode_inst_at_addr(host_inst_addr);
                if let Some(_inst) = inst {
                    ctx.sepc += len;
                    // todo!()
                }else{
                    return Err(VmmError::DecodeInstError)
                }
            }else{
                return Err(VmmError::TranslationError)
            }
        }else if inst == 0x3020 || inst == 0x3000 {
            // TODO: we should reinject this in the guest as a fault access
            herror!("fault on 1st stage page table walk");
            return Err(VmmError::PseudoInst)
        }else{
            // If htinst is valid and is not a pseudo instructon make sure
            // the opcode is valid even if it was a compressed instruction,
            // but before save the real instruction size.
            todo!()
        }
        // return Ok(())
        todo!()
    }else{
        Err(VmmError::DeviceNotFound)
    }
}


#[no_mangle]
pub unsafe fn trap_handler() -> ! {
    let ctx = (TRAP_CONTEXT as *mut TrapContext).as_mut().unwrap();
    let scause = scause::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            panic!("U-mode/VU-mode env call from VS-mode?");
        },
        Trap::Exception(Exception::VirtualSupervisorEnvCall) => {
            if let Err(err) = sbi_handler(ctx) {
                panic!("err: {:?}", err);
            }
            ctx.sepc += 4;
        },
        Trap::Exception(Exception::VirtualInstruction) => {
            if let Err(err) = privileged_inst_handler(ctx) {
                panic!("err: {:?}", err);
            }
        },
        Trap::Exception(Exception::IllegalInstruction) => {
            // Invalid instruction, read/write csr
            panic!("read/write CSR");
        },
        Trap::Exception(Exception::InstructionGuestPageFault) => { 
            let host_vmm = unsafe{ HOST_VMM.get().unwrap().lock() };
            let guest_id = host_vmm.guest_id;
            let gpm = &host_vmm.guests.get(&guest_id).unwrap().gpm;
            if let Some(host_va) = two_stage_translation(guest_id, ctx.sepc, vsatp::read().bits(), gpm) {
                herror!("host va: {:#x}", host_va);
            }else{
                herror!("Fail to translate exception pc.");
            }
            panic!(
                "InstructionGuestPageFault: sepc -> {:#x}, hgatp -> {:#x}", 
                ctx.sepc, hgatp::read().bits()
            );
    },
    Trap::Exception(Exception::LoadGuestPageFault) | Trap::Exception(Exception::StoreGuestPageFault) => {
        match guest_page_fault_handler(ctx) {
            Ok(()) => {}
            Err(VmmError::DeviceNotFound) => {
                herror!("Device not found!");
                todo!()
            },
            Err(VmmError::DecodeInstError) => {
                herror!("Decode failed at addr {:#x}", ctx.sepc);
            }
            _ => unimplemented!()
        }
    },
        _ => panic!("scause: {:?}, sepc: {:#x}", scause.cause(), ctx.sepc)
    }
    switch_to_guest()
}

#[no_mangle]
/// set the new addr of __restore asm function in TRAMPOLINE page,
/// set the reg a0 = trap_cx_ptr, reg a1 = phy addr of usr page table,
/// finally, jump to new addr of __restore asm function
pub unsafe fn switch_to_guest() -> ! {
    set_user_trap_entry();
    // 获取上下文切换环境
    let ctx = (TRAP_CONTEXT as *mut TrapContext).as_mut().unwrap();

    // hgatp: set page table for guest physical address translation
    if riscv::register::hgatp::read().bits() != ctx.hgatp {
        let hgatp = riscv::register::hgatp::Hgatp::from_bits(ctx.hgatp);
        hgatp.write(); 
        core::arch::riscv64::hfence_gvma_all();
        assert_eq!(hgatp.bits(), riscv::register::hgatp::read().bits());
    }

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