//! SBI call wrappers

use core::arch::asm;


pub const SBI_CONSOLE_PUTCHAR: usize = 1;
pub const SBI_CONSOLE_GETCHAR: usize = 2;

pub mod leagcy {
    pub const SBI_SET_TIMER: usize = 0;
}

pub const SBI_SUCCESS: usize = 0;
pub const SBI_ERR_FAILUER: isize = -1;
pub const SBI_ERR_NOT_SUPPORTED: isize = -2;
pub const SBI_ERR_INAVLID_PARAM: isize = -3;
pub const SBI_ERR_DENIED: isize = -4;
pub const SBI_ERR_INVALID_ADDRESS: isize = -5;
pub const SBI_ERR_ALREADY_AVAILABLE: isize = -6; 

pub const SBI_EXTID_BASE: usize = 0x10;
pub const SBI_GET_SBI_SPEC_VERSION_FID: usize = 0;
pub const SBI_GET_SBI_IMPL_ID_FID: usize = 1;
pub const SBI_GET_SBI_IMPL_VERSION_FID: usize = 2;
pub const SBI_PROBE_EXTENSION_FID: usize = 3;
pub const SBI_GET_MVENDORID_FID: usize = 4;
pub const SBI_GET_MARCHID_FID: usize = 5;
pub const SBI_GET_MIMPID_FID: usize = 6;

pub const SBI_EXTID_TIME: usize = 0x54494D45;
pub const SBI_SET_TIMER_FID: usize = 0x0;

pub const SBI_EXTID_IPI: usize = 0x735049;
pub const SBI_SEND_IPI_FID: usize = 0x0;

pub const SBI_EXTID_HSM: usize = 0x48534D;
pub const SBI_HART_START_FID: usize = 0;
pub const SBI_HART_STOP_FID: usize = 1;
pub const SBI_HART_STATUS_FID: usize = 2;

pub const SBI_EXTID_RFNC: usize = 0x52464E43;
pub const SBI_REMOTE_FENCE_I_FID: usize = 0;
pub const SBI_REMOTE_SFENCE_VMA_FID: usize = 1;
pub const SBI_REMOTE_SFENCE_VMA_ASID_FID: usize = 2;
pub const SBI_REMOTE_HFENCE_GVMA_FID: usize = 3;
pub const SBI_REMOTE_HFENCE_GVMA_VMID_FID: usize = 4;
pub const SBI_REMOTE_HFENCE_VVMA_FIDL: usize = 5;
pub const SBI_REMOTE_HFENCE_VVMA_ASID_FID: usize = 6;


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


