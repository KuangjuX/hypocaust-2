pub mod stack {
    use crate::{constants::{
        PAGE_SIZE, KERNEL_STACK_SIZE,
        layout::TRAP_CONTEXT
    }, shared::SHARED_DATA, mm::MapPermission};
    pub struct HypervisorStack(pub usize);

    pub fn hstack_position(guest_id: usize) -> (usize, usize) {
        let top = TRAP_CONTEXT - guest_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
        let bottom = top - KERNEL_STACK_SIZE;
        (bottom, top)
    }

    pub fn hstack_alloc(guest_id: usize) -> HypervisorStack {
        let (hstack_bottom, hstack_top) = hstack_position(guest_id);
        let mut sharded_data = SHARED_DATA.lock();
        sharded_data.hpm.insert_framed_area(
            hstack_bottom.into(),
            hstack_top.into(),
            MapPermission::R | MapPermission::W
        );
        HypervisorStack(guest_id)
    }

    impl HypervisorStack {
        pub fn get_top(&self) -> usize {
            let (_, hstack_top) = hstack_position(self.0);
            hstack_top
        }
    }

}