#![no_std]
#![no_main]
#![feature(panic_info_message)]
// #![deny(warnings)]

#[macro_use]
mod sbi;
mod lang_items;
mod console;
mod boards;

use core::arch::global_asm;

global_asm!(include_str!("asm/entry.asm"));




/// clear BSS segment
pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}


#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("[kernel] Hello, Guest Kernel!");
    unsafe{ 
        csrw_test();
        csrr_test(); 
    }
    panic!("panic in rust_main.")
}

pub unsafe fn csrw_test() {
    core::arch::asm!(
        "li t0, 0xdeaf",
        "csrw sscratch, t0"
    );
    println!("[kernel] csrw_test passed!");
}

pub unsafe fn csrr_test() {
    let mut x = 0;
    core::arch::asm!(
        "csrr {}, sscratch",
        out(reg) x
    );
    assert_eq!(x, 0xdeaf);
    println!("[kernel] csrr_test passed!");
}