use core::arch::{asm, global_asm};

use crate::constants::layout::{GUEST_DTB_ADDR, TRAMPOLINE, TRAP_CONTEXT};
use crate::device_emu::plic::is_plic_access;
use crate::guest::page_table::GuestPageTable;
use crate::guest::pmap::{decode_inst, two_stage_translation};
use crate::hypervisor::{HostVmm, HOST_VMM};
use crate::page_table::{PageTable, PageTableSv39};
use crate::{VmmError, VmmResult};

use riscv::register::scause::{Exception, Interrupt, Trap};
use riscv::register::{
    hgatp, htinst, htval, hvip, scause, sepc, sie, sscratch, stval, stvec, vsatp, vstvec,
};

pub use super::context::TrapContext;
use super::pmap::fast_two_stage_translation;
use super::sbi::sbi_vs_handler;

global_asm!(include_str!("trap.S"));

/// initialize CSR `stvec` as the entry of `__alltraps`
pub fn trap_init() {
    set_kernel_trap_entry();
}

/// enable timer interrupt in sie CSR
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

pub fn disable_timer_interrupt() {
    unsafe {
        sie::clear_stimer();
    }
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
        stvec::write(TRAMPOLINE, stvec::TrapMode::Direct);
    }
}

fn privileged_inst_handler(_ctx: &mut TrapContext) -> VmmResult {
    todo!()
}

pub fn guest_page_fault_handler<P: PageTable, G: GuestPageTable>(
    host_vmm: &mut HostVmm<P, G>,
    ctx: &mut TrapContext,
) -> VmmResult {
    let addr = htval::read() << 2;
    if is_plic_access(addr) {
        let mut inst = htinst::read();
        if inst == 0 {
            // If htinst does not provide information about the trap,
            // we must read the instruction from guest's memory manually
            let inst_addr = ctx.sepc;
            // let gpm = &host_vmm.guests[host_vmm.guest_id].as_ref().unwrap().gpm;
            if let Some(host_inst_addr) = fast_two_stage_translation::<PageTableSv39>(
                host_vmm.guest_id,
                inst_addr,
                vsatp::read().bits(),
            ) {
                inst = unsafe { core::ptr::read(host_inst_addr as *const usize) };
            } else {
                error!("inst addr: {:#x}", inst_addr);
                return Err(VmmError::TranslationError);
            }
        } else if inst == 0x3020 || inst == 0x3000 {
            // TODO: we should reinject this in the guest as a fault access
            error!("fault on 1st stage page table walk");
            return Err(VmmError::PseudoInst);
        } else {
            // If htinst is valid and is not a pseudo instructon make sure
            // the opcode is valid even if it was a compressed instruction,
            // but before save the real instruction size.
        }
        let (len, inst) = decode_inst(inst);
        if let Some(inst) = inst {
            host_vmm.handle_plic_access(ctx, addr, inst)?;
            ctx.sepc += len;
        } else {
            return Err(VmmError::DecodeInstError);
        }
        Ok(())
    } else {
        error!("addr: {:#x}, sepc: {:#x}", addr, ctx.sepc);
        Err(VmmError::DeviceNotFound)
        // todo: handle other device
    }
}

/// handle interrupt request(current only external interrupt)
pub fn handle_irq<P: PageTable, G: GuestPageTable>(
    host_vmm: &mut HostVmm<P, G>,
    _ctx: &mut TrapContext,
) {
    let host_plic = host_vmm.host_plic.as_mut().unwrap();
    // get current guest context id
    let context_id = 2 * host_vmm.guest_id + 1;
    let claim_and_complete_addr = host_plic.base_addr + 0x0020_0004 + 0x1000 * context_id;
    let irq = unsafe { core::ptr::read(claim_and_complete_addr as *const u32) };
    host_plic.claim_complete[context_id] = irq;

    // set external interrupt pending, which trigger guest interrupt
    unsafe { hvip::set_vseip() };

    // set irq pending in host vmm
    host_vmm.irq_pending = true;
}

/// forward exception by setting `vsepc` & `vscause`
pub fn forward_exception(ctx: &mut TrapContext) {
    unsafe {
        asm!(
            "csrw vsepc, {sepc}",
            "csrw vscause, {scause}",
            sepc = in(reg) ctx.sepc,
            scause = in(reg) scause::read().bits()
        )
    }
    ctx.sepc = vstvec::read().bits();
}

pub fn handle_internal_vmm_error(err: VmmError) {
    panic!("err: {:?}", err);
}

#[no_mangle]
#[allow(unreachable_code)]
pub unsafe fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let ctx = (TRAP_CONTEXT as *mut TrapContext).as_mut().unwrap();
    let scause = scause::read();
    let host_vmm = HOST_VMM.get_mut().unwrap();
    let mut host_vmm = host_vmm.lock();
    let mut err = None;
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            panic!("U-mode/VU-mode env call from VS-mode?");
        }
        Trap::Exception(Exception::VirtualSupervisorEnvCall) => {
            if let Err(vmm_err) = sbi_vs_handler(ctx) {
                err = Some(vmm_err);
            }
            ctx.sepc += 4;
        }
        Trap::Exception(Exception::VirtualInstruction) => {
            if let Err(vmm_err) = privileged_inst_handler(ctx) {
                err = Some(vmm_err);
            }
        }
        Trap::Exception(Exception::InstructionGuestPageFault) => {
            let host_vmm = unsafe { HOST_VMM.get().unwrap().lock() };
            let guest_id = host_vmm.guest_id;
            let gpm = &host_vmm.guests[guest_id].as_ref().unwrap().gpm;
            if let Some(host_va) =
                two_stage_translation(guest_id, ctx.sepc, vsatp::read().bits(), gpm)
            {
                error!("host va: {:#x}", host_va);
            } else {
                error!("Fail to translate exception pc.");
            }
            panic!(
                "InstructionGuestPageFault: sepc -> {:#x}, hgatp -> {:#x}",
                ctx.sepc,
                hgatp::read().bits()
            );
        }
        Trap::Exception(Exception::LoadGuestPageFault)
        | Trap::Exception(Exception::StoreGuestPageFault) => {
            if let Err(vmm_err) = guest_page_fault_handler(&mut host_vmm, ctx) {
                err = Some(vmm_err);
            }
            host_vmm.guest_page_falut += 1;
            if host_vmm.guest_page_falut % 1000 == 0 {
                trace!(
                    "guest page fault: {}, addr: {:#x}",
                    host_vmm.guest_page_falut,
                    htval::read() << 2
                );
            }
        }
        Trap::Interrupt(Interrupt::SupervisorExternal) => {
            handle_irq(&mut host_vmm, ctx);
            host_vmm.external_irq += 1;
            // htracking!("external irq: {}", host_vmm.external_irq);
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            // set guest timer interrupt pending
            hvip::set_vstip();
            // disable timer interrupt
            sie::clear_stimer();
            host_vmm.timer_irq += 1;
        }
        _ => forward_exception(ctx),
    }
    drop(host_vmm);
    if let Some(err) = err {
        // TODO: handler vmm error
        handle_internal_vmm_error(err)
    }
    switch_to_guest()
}

pub unsafe fn hart_entry_1() -> ! {
    set_user_trap_entry();
    // get guest context
    let ctx = (TRAP_CONTEXT as *mut TrapContext).as_mut().unwrap();

    // hgatp: set page table for guest physical address translation
    if riscv::register::hgatp::read().bits() != ctx.hgatp {
        let hgatp = riscv::register::hgatp::Hgatp::from_bits(ctx.hgatp);
        hgatp.write();
        core::arch::riscv64::hfence_gvma_all();
        assert_eq!(hgatp.bits(), riscv::register::hgatp::read().bits());
    }
    hart_entry_2()
}

/// first enter guest, pass dtb
#[naked]
pub unsafe extern "C" fn hart_entry_2() -> ! {
    core::arch::asm!(
        "fence.i",
        "li a0, {trap_context}",
        "csrw sscratch, a0",
        "mv sp, a0",
        "ld t0, 32*8(sp)",
        "ld t1, 33*8(sp)",
        "csrw sstatus, t0",
        "csrw sepc, t1",
        "ld t0, 37*8(sp)",
        "csrw hstatus, t0",
        "ld x1, 1*8(sp)",
        "ld x3, 3*8(sp)",
        "ld x5, 5*8(sp)",
        "ld x6, 6*8(sp)",
        "ld x7, 7*8(sp)",
        "ld x8, 8*8(sp)",
        "ld x9, 9*8(sp)",
        "ld x10, 10*8(sp)",
        "ld x11, 11*8(sp)",
        "ld x12, 12*8(sp)",
        "ld x13, 13*8(sp)",
        "ld x14, 14*8(sp)",
        "ld x15, 15*8(sp)",
        "ld x16, 16*8(sp)",
        "ld x17, 17*8(sp)",
        "ld x18, 18*8(sp)",
        "ld x19, 19*8(sp)",
        "ld x20, 20*8(sp)",
        "ld x21, 21*8(sp)",
        "ld x22, 22*8(sp)",
        "ld x23, 23*8(sp)",
        "ld x24, 24*8(sp)",
        "ld x25, 25*8(sp)",
        "ld x26, 26*8(sp)",
        "ld x27, 27*8(sp)",
        "ld x28, 28*8(sp)",
        "ld x29, 29*8(sp)",
        "ld x30, 30*8(sp)",
        "ld x31, 31*8(sp)",
        "ld sp, 2*8(sp)",
        "li a1, {guest_dtb}",
        "sret",
        trap_context = const TRAP_CONTEXT,
        guest_dtb = const GUEST_DTB_ADDR,
        options(noreturn)
    )
}

#[no_mangle]
/// set the new addr of __restore asm function in TRAMPOLINE page,
/// set the reg a0 = trap_cx_ptr, reg a1 = phy addr of usr page table,
/// finally, jump to new addr of __restore asm function
pub unsafe fn switch_to_guest() -> ! {
    set_user_trap_entry();
    // get guest context
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
    let scause = scause::read();
    let sepc = sepc::read();
    match scause.cause() {
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            let stval = stval::read();
            panic!(
                "scause: {:?}, sepc: {:#x}, stval: {:#x}",
                scause.cause(),
                _trap_cx.sepc,
                stval
            );
        }
        _ => {
            panic!(
                "scause: {:?}, spec: {:#x}, stval: {:#x}",
                scause.cause(),
                sepc,
                stval::read()
            )
        }
    }
}
