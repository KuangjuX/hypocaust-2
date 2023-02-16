use crate::constants::layout::{TRAP_CONTEXT, GUEST_START_VA};
// use crate::{mm::MemorySet, page_table::PageTable, hypervisor::stack::{hstack_alloc, HypervisorStack}};
use crate::mm::MemorySet;
use crate::page_table::{PageTable, PhysPageNum, VirtPageNum};
use crate::hypervisor::stack::hstack_alloc;
use crate::shared::SHARED_DATA;
use crate::trap::TrapContext;

pub struct Guest<P: PageTable> {
    pub gpm: MemorySet<P>,
    pub guest_id: usize,
    pub trap_cx_ppn: PhysPageNum,
}

impl<P: PageTable> Guest<P> {
    pub fn new(guest_id: usize, gpm: MemorySet<P>) -> Self {
        // 分配 hypervisor 内核栈
        let hstack = hstack_alloc(guest_id);
        let hstack_top = hstack.get_top();
        let shared_data = SHARED_DATA.lock();
        let trap_cx_ppn = shared_data.hpm
            .translate(VirtPageNum::from(TRAP_CONTEXT >> 12))
            .unwrap()
            .ppn();
        drop(shared_data);
        let trap_ctx: &mut TrapContext = trap_cx_ppn.get_mut();
        *trap_ctx = TrapContext::initialize_context(
            GUEST_START_VA,
            0,
            gpm.token(),
            hstack_top,
            0
        );
        Self {
            guest_id,
            gpm,
            trap_cx_ppn
        }
    }
}