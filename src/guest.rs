use crate::{mm::MemorySet, page_table::PageTable, hypervisor::stack::{hstack_alloc, HypervisorStack}};

pub struct Guest<P: PageTable> {
    pub gpm: MemorySet<P>,
    pub guest_id: usize,
    pub hstack: HypervisorStack
}

impl<P: PageTable> Guest<P> {
    pub fn new(guest_id: usize, gpm: MemorySet<P>) -> Self {
        // 分配 hypervisor 内核栈
        let hstack = hstack_alloc(guest_id);
        Self {
            guest_id,
            gpm,
            hstack
        }
    }
}