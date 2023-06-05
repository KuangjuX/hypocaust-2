#![no_std]
#![no_main]
#![deny(warnings)]
#![allow(non_upper_case_globals, dead_code)]
#![feature(
    panic_info_message,
    alloc_error_handler,
    core_intrinsics,
    naked_functions,
    asm_const,
    stdsimd
)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;

extern crate alloc;

#[path = "boards/qemu_virt_riscv.rs"]
mod board;

#[macro_use]
mod console;
mod constants;
mod detect;
mod device_emu;
mod drivers;
mod error;
mod guest;
mod hyp_alloc;
mod hypervisor;
mod lang_items;
mod mm;
mod page_table;
mod sbi;
mod sync;

use crate::constants::layout::{GUEST_DEFAULT_SIZE, GUEST_START_PA};
use crate::constants::PAGE_SIZE;
use crate::guest::vmexit::hart_entry_1;
use crate::guest::Guest;
use crate::hypervisor::{add_guest_queue, init_vmm, HOST_VMM};
use crate::mm::{GuestMemorySet, HostMemorySet};
use crate::page_table::PageTableSv39;

pub use error::{VmmError, VmmResult};

/// hypervisor boot stack size
const BOOT_STACK_SIZE: usize = 16 * PAGE_SIZE;

#[link_section = ".bss.stack"]
/// hypervisor boot stack
static BOOT_STACK: [u8; BOOT_STACK_SIZE] = [0u8; BOOT_STACK_SIZE];

#[link_section = ".text.entry"]
#[export_name = "_start"]
#[naked]
/// hypervisor entrypoint
///
/// # Safety
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

#[no_mangle]
unsafe fn hentry(hart_id: usize, dtb: usize) -> ! {
    if hart_id == 0 {
        clear_bss();
        console::init();
        info!("Hello Hypocaust-2!");
        info!("hart id: {}, dtb: {:#x}", hart_id, dtb);
        // detect h extension
        if sbi_rt::probe_extension(sbi_rt::Hsm).is_unavailable() {
            panic!("no HSM extension exist on current SBI environment");
        }
        if !detect::detect_h_extension() {
            panic!("no RISC-V hypervisor H extension on current environment")
        }
        info!("Hypocaust-2 > running with hardware RISC-V H ISA acceration!");

        // initialize heap
        hyp_alloc::heap_init();
        // let machine = hypervisor::fdt::MachineMeta::parse(dtb);
        // parse guest fdt
        // let guest_machine = hypervisor::fdt::MachineMeta::parse(0x9000_0000);
        // initialize vmm
        let hpm = HostMemorySet::<PageTableSv39>::new_host_vmm();
        init_vmm(hpm);
        // create guest memory set
        let gpm = GuestMemorySet::<PageTableSv39>::setup_gpm();

        let mut host_vmm = HOST_VMM.get_mut().unwrap().lock();
        host_vmm.hpm.map_guest(GUEST_START_PA, GUEST_DEFAULT_SIZE);
        drop(host_vmm);
        // hypervisor enable paging
        mm::enable_paging();
        // trap init
        guest::vmexit::trap_init();
        // memory translation test
        mm::remap_test();
        // create guest struct
        let guest = Guest::new(0, gpm);
        add_guest_queue(guest);
        info!("Start to run guest......");
        hart_entry_1()
    } else {
        unreachable!()
    }
}
