use crate::constants::layout::{TRAP_CONTEXT, GUEST_START_VA};
// use crate::{mm::MemorySet, page_table::PageTable, hypervisor::stack::{hstack_alloc, HypervisorStack}};
use crate::mm::MemorySet;
use crate::page_table::PageTable;
use crate::hypervisor::stack::hstack_alloc;
use crate::shared::SHARED_DATA;
use crate::trap::{TrapContext, trap_handler};

pub struct Guest<P: PageTable> {
    pub gpm: MemorySet<P>,
    pub guest_id: usize
}

impl<P: PageTable> Guest<P> {
    pub fn new(guest_id: usize, gpm: MemorySet<P>) -> Self {
        // 分配 hypervisor 内核栈
        let hstack = hstack_alloc(guest_id);
        let hstack_top = hstack.get_top();
        let shared_data = SHARED_DATA.lock();
        // let trap_cx_ppn = shared_data.hpm
        //     .translate(VirtPageNum::from(TRAP_CONTEXT >> 12))
        //     .unwrap()
        //     .ppn();
        drop(shared_data);
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
            gpm
        }
    }
}