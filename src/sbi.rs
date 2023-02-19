//! SBI call wrappers

use core::arch::asm;


pub const SBI_CONSOLE_PUTCHAR: usize = 1;
pub const SBI_CONSOLE_GETCHAR: usize = 2;

pub mod leagcy {
    pub const SBI_SET_TIMER: usize = 0;
}


#[inline(always)]
/// general sbi call
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "li x16, 0",
            "ecall",
            inlateout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x17") which,
        );
    }
    ret
}

/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

/// use sbi call to getchar from console (qemu uart handler)
pub fn console_getchar() -> usize {
    sbi_call(SBI_CONSOLE_GETCHAR, 0, 0, 0)
}

pub fn set_timer(stime: usize) {
    sbi_rt::set_timer(stime as u64);
}

/// use sbi call to shutdown the kernel
pub fn shutdown() -> ! {
    sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::SystemFailure);
    unreachable!()
}