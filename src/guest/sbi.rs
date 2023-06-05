use super::vmexit::TrapContext;
use crate::constants::riscv_regs::GprIndex;
use crate::sbi::leagcy::SBI_SET_TIMER;
use crate::sbi::{
    console_getchar, console_putchar, set_timer, SBI_CONSOLE_GETCHAR, SBI_CONSOLE_PUTCHAR,
    SBI_ERR_NOT_SUPPORTED, SBI_EXTID_BASE, SBI_EXTID_RFNC, SBI_EXTID_TIME, SBI_GET_MARCHID_FID,
    SBI_GET_MIMPID_FID, SBI_GET_MVENDORID_FID, SBI_GET_SBI_IMPL_ID_FID,
    SBI_GET_SBI_IMPL_VERSION_FID, SBI_GET_SBI_SPEC_VERSION_FID, SBI_PROBE_EXTENSION_FID,
    SBI_SET_TIMER_FID, SBI_SUCCESS,
};
use crate::VmmResult;
use sbi_rt::{
    get_marchid, get_mimpid, get_mvendorid, get_sbi_impl_id, get_sbi_impl_version,
    pmu_counter_get_info, pmu_counter_stop, pmu_num_counters, remote_fence_i, remote_sfence_vma,
};
use sbi_spec::pmu::{EID_PMU, PMU_COUNTER_GET_INFO, PMU_COUNTER_STOP, PMU_NUM_COUNTERS};
use sbi_spec::rfnc::{REMOTE_FENCE_I, REMOTE_SFENCE_VMA};

use riscv::register::{hvip, sie};
pub struct SbiRet {
    error: usize,
    value: usize,
}

#[inline(always)]
pub(crate) fn sbi_call_1(eid: usize, fid: usize, arg0: usize) -> SbiRet {
    let (error, value);
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") eid,
            in("a6") fid,
            inlateout("a0") arg0 => error,
            lateout("a1") value,
        );
    }
    SbiRet { error, value }
}

#[allow(unused_assignments)]
pub fn sbi_vs_handler(ctx: &mut TrapContext) -> VmmResult {
    let ext_id: usize = ctx.x[GprIndex::A7 as usize];
    let fid: usize = ctx.x[GprIndex::A6 as usize];
    let mut sbi_ret = SbiRet {
        error: SBI_SUCCESS,
        value: 0,
    };
    match ext_id {
        SBI_EXTID_BASE => sbi_ret = sbi_base_handler(fid, ctx),
        SBI_EXTID_TIME => sbi_ret = sbi_time_handler(ctx.x[GprIndex::A0 as usize], fid),
        SBI_CONSOLE_PUTCHAR => sbi_ret = sbi_console_putchar_handler(ctx.x[GprIndex::A0 as usize]),
        SBI_CONSOLE_GETCHAR => sbi_ret = sbi_console_getchar_handler(),
        SBI_SET_TIMER => sbi_ret = sbi_legacy_set_time(ctx.x[GprIndex::A0 as usize]),
        SBI_EXTID_RFNC => sbi_ret = sbi_rfnc_handler(fid, &ctx.x),
        EID_PMU => sbi_ret = sbi_pmu_handler(fid, &ctx.x),
        _ => panic!("Unsupported SBI call id {:#x}", ext_id),
    }
    ctx.x[GprIndex::A0 as usize] = sbi_ret.error;
    ctx.x[GprIndex::A1 as usize] = sbi_ret.value;

    Ok(())
}

pub fn sbi_base_handler(fid: usize, ctx: &TrapContext) -> SbiRet {
    let mut sbi_ret = SbiRet {
        error: SBI_SUCCESS,
        value: 0,
    };
    match fid {
        SBI_GET_SBI_SPEC_VERSION_FID => sbi_ret = sbi_call_1(SBI_EXTID_BASE, fid, 0),
        SBI_GET_SBI_IMPL_ID_FID => sbi_ret.value = get_sbi_impl_id(),
        SBI_GET_SBI_IMPL_VERSION_FID => sbi_ret.value = get_sbi_impl_version(),
        SBI_PROBE_EXTENSION_FID => {
            let extension = ctx.x[GprIndex::A0 as usize];
            sbi_ret = sbi_call_1(SBI_EXTID_BASE, fid, extension)
        }
        SBI_GET_MVENDORID_FID => sbi_ret.value = get_mvendorid(),
        SBI_GET_MARCHID_FID => sbi_ret.value = get_marchid(),
        SBI_GET_MIMPID_FID => sbi_ret.value = get_mimpid(),
        _ => panic!("sbi base handler fid: {}", fid),
    }
    sbi_ret
}

pub fn sbi_console_putchar_handler(c: usize) -> SbiRet {
    console_putchar(c);
    SbiRet {
        error: SBI_SUCCESS,
        value: 0,
    }
}

pub fn sbi_console_getchar_handler() -> SbiRet {
    let c = console_getchar();
    SbiRet {
        error: SBI_SUCCESS,
        value: c,
    }
}

pub fn sbi_time_handler(stime: usize, fid: usize) -> SbiRet {
    let mut sbi_ret = SbiRet {
        error: SBI_SUCCESS,
        value: 0,
    };
    if fid != SBI_SET_TIMER_FID {
        sbi_ret.error = SBI_ERR_NOT_SUPPORTED as usize;
        return sbi_ret;
    }

    // htracking!("set timer: {}", stime);
    set_timer(stime);
    unsafe {
        // clear guest timer interrupt pending
        hvip::clear_vstip();
        // enable timer interrupt
        sie::set_stimer();
    }
    sbi_ret
}

#[allow(clippy::needless_late_init)]
pub fn sbi_rfnc_handler(fid: usize, args: &[usize]) -> SbiRet {
    let sbi_ret;
    match fid {
        REMOTE_FENCE_I => {
            sbi_ret = remote_fence_i(args[GprIndex::A0 as usize], args[GprIndex::A1 as usize])
        }
        REMOTE_SFENCE_VMA => {
            sbi_ret = remote_sfence_vma(
                args[GprIndex::A0 as usize],
                args[GprIndex::A1 as usize],
                args[GprIndex::A2 as usize],
                args[GprIndex::A3 as usize],
            )
        }
        _ => todo!("sbi rfnc handler fid: {}", fid),
    }
    SbiRet {
        error: sbi_ret.error,
        value: sbi_ret.value,
    }
}

#[allow(clippy::needless_late_init)]
fn sbi_pmu_handler(fid: usize, args: &[usize]) -> SbiRet {
    let sbi_ret: sbi_rt::SbiRet;
    match fid {
        PMU_NUM_COUNTERS => {
            sbi_ret = sbi_rt::SbiRet {
                error: SBI_SUCCESS,
                value: pmu_num_counters(),
            }
        }
        PMU_COUNTER_GET_INFO => sbi_ret = pmu_counter_get_info(args[GprIndex::A0 as usize]),
        PMU_COUNTER_STOP => {
            sbi_ret = pmu_counter_stop(
                args[GprIndex::A0 as usize],
                args[GprIndex::A1 as usize],
                args[GprIndex::A2 as usize],
            )
        }
        _ => todo!("sbi pmu handler fid: {}", fid),
    }
    SbiRet {
        error: sbi_ret.error,
        value: sbi_ret.value,
    }
}

pub fn sbi_legacy_set_time(stime: usize) -> SbiRet {
    let sbi_ret = SbiRet {
        error: SBI_SUCCESS,
        value: 0,
    };
    set_timer(stime);
    unsafe {
        // clear guest timer interrupt pending
        hvip::clear_vstip();
        // enable timer interrupt
        sie::set_stimer();
    }
    sbi_ret
}
