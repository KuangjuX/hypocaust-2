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

#[macro_use]
mod console;
mod sbi;
mod lang_items;
mod detect;




const BOOT_STACK_SIZE: usize = 16 * 4096;

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


#[no_mangle]
fn hentry(hart_id: usize, dtb: usize) -> ! {
    if hart_id == 0 {
        hdebug!("Hello Hypocaust-2!");
        hdebug!("hart id: {}, dtb: {:#x}", hart_id, dtb);
        if sbi_rt::probe_extension(sbi_rt::Hsm).is_unavailable() {
            panic!("no HSM extension exist under current SBI environment");
        }else{
            hdebug!("HSM extension is available under current SBI environment");
        }
        if !detect::detect_h_extension() {
            panic!("no RISC-V hypervisor H extension current environment")
        }else{
            hdebug!("RISC-V hypervisor H extension is available on current environment");
        }
        hdebug!("Hypocaust-2 > running with hardware RISC-V H OSA acceration");
        unreachable!()
    }else{
        unreachable!()
    }
}
