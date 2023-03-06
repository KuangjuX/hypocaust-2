use crate::constants::layout::{TRAP_CONTEXT, GUEST_START_VA};
use crate::hypervisor::fdt::MachineMeta;
use crate::mm::{ GuestMemorySet, MemorySet };
use crate::hypervisor::{ stack::hstack_alloc};
use vmexit::{TrapContext, trap_handler};

use self::page_table::GuestPageTable;
use self::vcpu::VCpu;
pub use sbi::SbiRet;

mod context;
mod vcpu;
mod sbi;
pub mod vmexit;


pub struct Guest<G: GuestPageTable> {
    pub guest_machine: MachineMeta,
    /// guest memory set
    pub gpm: GuestMemorySet<G>,
    /// guest id
    pub guest_id: usize,
    /// virtual cpu status
    pub vcpu: VCpu
}

impl<G: GuestPageTable> Guest<G> {
    pub fn new(guest_id: usize, gpm: GuestMemorySet<G>, guest_machine: MachineMeta) -> Self {
        // 分配 hypervisor 内核栈
        let hstack = hstack_alloc(guest_id);
        let hstack_top = hstack.get_top();
        // 获取 trap context
        let trap_ctx: &mut TrapContext = unsafe{ (TRAP_CONTEXT as *mut TrapContext).as_mut().unwrap() };
        // 初始化 trap context 的环境
        // 包括入口地址/栈寄存器/satp/内核栈寄存器/trap处理地址
        *trap_ctx = TrapContext::initialize_context(
            GUEST_START_VA,
            0,
            gpm.token(),
            hstack_top,
            trap_handler as usize
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

    use crate::{mm::{MemorySet, GuestMemorySet}, page_table::translate_guest_va};
    use super::page_table::GuestPageTable;
    // use riscv_decode;

    #[allow(unused)]
    mod segment_layout {
        pub const GUEST_SEGMENT_SIZE: usize = 128 * 1024 * 1024;
    }

    pub fn gpa2hpa(va: usize, guest_id: usize) -> usize {
        va + (guest_id + 1) * segment_layout::GUEST_SEGMENT_SIZE
    }

    pub fn hpa2gpa(pa: usize, gust_id: usize) -> usize {
        pa - (gust_id + 1) * segment_layout::GUEST_SEGMENT_SIZE
    }

    pub fn two_stage_translation<G: GuestPageTable>(guest_id: usize, guest_va: usize, vsatp: usize, gpm: &GuestMemorySet<G>) -> Option<usize> {
        let guest_root = (vsatp & 0x3ff_ffff_ffff) << 12;
        if let Some(translation) = translate_guest_va::<G>(guest_id, guest_root, guest_va) {
            let guest_pa = translation.guest_pa;
            if let Some(host_va) = gpm.translate_va(guest_pa) {
                Some(host_va)
            }else{
                return None
            }
        }else{
            return None
        }
    }

    // pub fn decode_inst_at_addr<P: GuestPageTable>(guest_va: usize, gpm: &MemorySet<P>) -> (usize, Option<riscv_decode::Instruction>){
    //     if let Some(host_va) = two_stage_translation(guest_va, gpm) {
    //         hdebug!("host va: {:#x}", host_va);
    //         let i1 = unsafe{ core::ptr::read(host_va as *const u16) };
    //         let len = riscv_decode::instruction_length(i1);
    //         let inst = match len {
    //             2 => i1 as u32, 
    //             4 => unsafe{ core::ptr::read(host_va as *const u32) },
    //             _ => unreachable!()
    //         };
    //         (len, riscv_decode::decode(inst).ok())
    //     }else{
    //         (0, None)
    //     }
    // }

    pub fn decode_inst_at_addr(host_va: usize) -> (usize, Option<Instruction>) {
        let i1 = unsafe{ core::ptr::read(host_va as *const u16) };
        let len = riscv_decode::instruction_length(i1);
        let inst = match len {
            2 => i1 as u32, 
            4 => unsafe{ core::ptr::read(host_va as *const u32) },
            _ => unreachable!()
        };
        (len, riscv_decode::decode(inst).ok())
    }
}





