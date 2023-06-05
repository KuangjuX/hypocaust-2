use crate::constants::layout::{GUEST_START_VA, TRAP_CONTEXT};
use crate::hypervisor::fdt::MachineMeta;
use crate::hypervisor::stack::hstack_alloc;
use crate::mm::{GuestMemorySet, MemorySet};
use vmexit::{trap_handler, TrapContext};

use self::page_table::GuestPageTable;
use self::vcpu::VCpu;
pub use sbi::SbiRet;

mod context;
mod sbi;
mod vcpu;
pub mod vmexit;

pub struct Guest<G: GuestPageTable> {
    pub guest_machine: MachineMeta,
    /// guest memory set
    pub gpm: GuestMemorySet<G>,
    /// guest id
    pub guest_id: usize,
    /// virtual cpu status
    pub vcpu: VCpu,
}

impl<G: GuestPageTable> Guest<G> {
    pub fn new(guest_id: usize, gpm: GuestMemorySet<G>, guest_machine: MachineMeta) -> Self {
        // 分配 hypervisor 内核栈
        let hstack = hstack_alloc(guest_id);
        let hstack_top = hstack.get_top();
        // 获取 trap context
        let trap_ctx: &mut TrapContext =
            unsafe { (TRAP_CONTEXT as *mut TrapContext).as_mut().unwrap() };
        // 初始化 trap context 的环境
        // 包括入口地址/栈寄存器/satp/内核栈寄存器/trap处理地址
        *trap_ctx = TrapContext::initialize_context(
            GUEST_START_VA,
            0,
            gpm.token(),
            hstack_top,
            trap_handler as usize,
        );
        Self {
            guest_id,
            gpm,
            guest_machine,
            vcpu: VCpu::new(guest_id),
        }
    }

    pub fn run(&mut self) {
        todo!()
    }
}

pub mod page_table {
    use crate::page_table::PageTable;

    pub trait GuestPageTable: PageTable {
        fn new_guest() -> Self;
    }
}

pub mod pmap {
    use riscv_decode::Instruction;

    use super::page_table::GuestPageTable;
    use crate::{
        mm::{GuestMemorySet, MemorySet},
        page_table::translate_guest_va,
    };
    // use riscv_decode;

    #[allow(unused)]
    mod segment_layout {
        pub const GUEST_SEGMENT_SIZE: usize = 128 * 1024 * 1024;
    }

    pub fn gpa2hpa(va: usize, guest_id: usize) -> usize {
        va + guest_id * segment_layout::GUEST_SEGMENT_SIZE
    }

    pub fn hpa2gpa(pa: usize, guest_id: usize) -> usize {
        pa - guest_id * segment_layout::GUEST_SEGMENT_SIZE
    }

    pub fn two_stage_translation<G: GuestPageTable>(
        guest_id: usize,
        guest_va: usize,
        vsatp: usize,
        gpm: &GuestMemorySet<G>,
    ) -> Option<usize> {
        let guest_root = (vsatp & 0x3ff_ffff_ffff) << 12;
        let guest_pa;
        if guest_root != 0 {
            if let Some(translation) = translate_guest_va::<G>(guest_id, guest_root, guest_va) {
                guest_pa = translation.guest_pa;
                // htracking!("guest pa: {:#x}", guest_pa);
            } else {
                return None;
            }
        } else {
            guest_pa = guest_va;
        }
        gpm.translate_va(guest_pa)
    }

    pub fn fast_two_stage_translation<G: GuestPageTable>(
        guest_id: usize,
        guest_va: usize,
        vsatp: usize,
    ) -> Option<usize> {
        let guest_root = (vsatp & 0x3ff_ffff_ffff) << 12;
        let guest_pa;
        if guest_root != 0 {
            if let Some(translation) = translate_guest_va::<G>(guest_id, guest_root, guest_va) {
                guest_pa = translation.guest_pa;
                // htracking!("guest pa: {:#x}", guest_pa);
            } else {
                return None;
            }
        } else {
            guest_pa = guest_va;
        }
        Some(guest_pa)
    }

    pub fn decode_inst_at_addr(host_va: usize) -> (usize, Option<Instruction>) {
        let i1 = unsafe { core::ptr::read(host_va as *const u16) };
        let len = riscv_decode::instruction_length(i1);
        let inst = match len {
            2 => i1 as u32,
            4 => unsafe { core::ptr::read(host_va as *const u32) },
            _ => unreachable!(),
        };
        (len, riscv_decode::decode(inst).ok())
    }

    /// decode risc-v instruction, return (inst len, inst)
    pub fn decode_inst(inst: usize) -> (usize, Option<Instruction>) {
        let i1 = inst as u16;
        let len = riscv_decode::instruction_length(i1);
        let inst = match len {
            2 => i1 as u32,
            4 => inst as u32,
            _ => unreachable!(),
        };
        (len, riscv_decode::decode(inst).ok())
    }
}
