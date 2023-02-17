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
        hdebug!("allocated hstack: [{:#x}: {:#x})",hstack_bottom, hstack_top);
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

use riscv::register::hvip;
use crate::constants::csr::{hedeleg, hideleg};

pub unsafe fn initialize_hypervisor() {
    // hedeleg: delegate some synchronous exceptions
    hedeleg::write(
        hedeleg::INST_ADDR_MISALIGN |
        hedeleg::BREAKPOINT | 
        hedeleg::ENV_CALL_FROM_U_OR_VU | 
        hedeleg::INST_PAGE_FAULT |
        hedeleg::LOAD_PAGE_FAULT |
        hedeleg::STORE_PAGE_FAULT
    );

    // hideleg: delegate all interrupts
    hideleg::write(
        hideleg::VSEIP |
        hideleg::VSSIP | 
        hideleg::VSTIP
    );

    // hvip: clear all interrupts
    hvip::clear_vseip();
    hvip::clear_vssip();
    hvip::clear_vstip();


    hdebug!("Initialize hypervisor environment");

}