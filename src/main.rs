#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(core_intrinsics)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)] 
#![deny(warnings)]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(stdsimd)]

#[macro_use]
extern crate bitflags;

extern crate alloc;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod sbi;
mod lang_items;
mod detect;
mod page_table;
mod constants;
mod hyp_alloc;
mod sync;
mod shared;
mod trap;
mod mm;
mod guest;
mod hypervisor;


use constants::layout::TRAMPOLINE;
use constants::PAGE_SIZE;
use riscv::register::{ hedeleg, hideleg, hvip, stvec };

use crate::mm::MemorySet;
use crate::constants::layout::GUEST_DEFAULT_SIZE;
use crate::page_table::{PageTableSv39, VirtPageNum};
use crate::guest::Guest;
use crate::shared::add_guest;
use crate::trap::switch_to_guest;

#[link_section = ".initrd"]
#[cfg(feature = "embed_guest_kernel")]
static GUEST: [u8;include_bytes!("../guest.bin").len()] = 
 *include_bytes!("../guest.bin");

 #[cfg(not(feature = "embed_guest_kernel"))]
 static GUEST: [u8; 0] = [];

/// hypervisor boot stack size
const BOOT_STACK_SIZE: usize = 16 * PAGE_SIZE;

#[link_section = ".bss.stack"]
/// hypocaust boot stack
static BOOT_STACK: [u8; BOOT_STACK_SIZE] = [0u8; BOOT_STACK_SIZE];

#[link_section = ".text.entry"]
#[export_name = "_start"]
#[naked]
/// hypocaust entrypoint
pub unsafe extern "C" fn start() -> ! {
    core::arch::asm!(
        // prepare stack
        "la sp, {boot_stack}",
        "li t2, {boot_stack_size}",
        "addi t3, a0, 1",
        "mul t2, t2, t3",
        "add sp, sp, t2",
        // enter hentry
        "call hentry",
        boot_stack = sym BOOT_STACK,
        boot_stack_size = const BOOT_STACK_SIZE,
        options(noreturn)
    )
}

/// clear BSS segment
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

unsafe fn initialize_hypervisor() {
    // hedeleg: delegate some synchronous exceptions
    // Instruction address misaligned
    hedeleg::set_ex0();
    // breakpoint
    hedeleg::set_ex3();
    // Environment call from U-mode or VU-mode 
    hedeleg::set_ex8();
    // Instruction page fault 
    hedeleg::set_ex12();
    // Load page fault 
    hedeleg::set_ex13();
    // Store/AMO page fault 
    hedeleg::set_ex15();

    // hideleg: delegate all interrupts
    hideleg::set_eip();
    hideleg::set_sip();
    hideleg::clear_tip();

    // hvip: clear all interrupts
    hvip::clear_vseip();
    hvip::clear_vssip();
    hvip::clear_vstip();

    // stvec: set handler
    stvec::write(TRAMPOLINE, stvec::TrapMode::Direct);
    assert_eq!(stvec::read().bits(), TRAMPOLINE);

    hdebug!("Initialize hypervisor environment");

}




#[no_mangle]
fn hentry(hart_id: usize, dtb: usize) -> ! {
    if hart_id == 0 {
        clear_bss();
        hdebug!("Hello Hypocaust-2!");
        hdebug!("hart id: {}, dtb: {:#x}", hart_id, dtb);
        // detect h extension
        if sbi_rt::probe_extension(sbi_rt::Hsm).is_unavailable() {
            panic!("no HSM extension exist on current SBI environment");
        }
        if !detect::detect_h_extension() {
            panic!("no RISC-V hypervisor H extension on current environment")
        }
        hdebug!("Hypocaust-2 > running with hardware RISC-V H ISA acceration!");
        unsafe{ initialize_hypervisor() };
        // initialize heap
        hyp_alloc::heap_init();

        // create guest memory set
        let mut gpm = MemorySet::<PageTableSv39>::new_guest(&GUEST, GUEST_DEFAULT_SIZE);
        // hypervisor enable paging
        mm::enable_paging(&gpm);
        // trap init
        trap::init();
        // memory translation test
        mm::remap_test();
        // initialize guest memory
        gpm.initialize_gpm();
        hdebug!("{:#x} -> {:#x}", TRAMPOLINE >> 12, gpm.translate(VirtPageNum::from(TRAMPOLINE >> 12)).unwrap().ppn().0);
        // 创建 guest
        let guest = Guest::new(0, gpm);
        add_guest(guest);

        // 切换上下文并跳转到 guest 执行
        unsafe{ switch_to_guest() }
    }else{
        unreachable!()
    }
}
