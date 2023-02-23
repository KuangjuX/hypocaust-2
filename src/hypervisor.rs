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
        let mut sharded_data = unsafe{ SHARED_DATA.get_mut().unwrap().lock() };
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

pub mod fdt {
///! ref: https://github.com/mit-pdos/RVirt/blob/HEAD/src/fdt.rs

use arrayvec::ArrayVec;
use fdt::Fdt;

#[derive(Clone, Debug)]
pub struct Device {
    pub base_address: usize,
    pub size: usize
}

#[derive(Clone, Debug, Default)]
pub struct MachineMeta{
    pub physical_memory_offset: usize,
    pub physical_memory_size: usize,

    pub virtio: ArrayVec<Device, 16>,

    pub test_finisher_address: Option<Device>,
}

impl MachineMeta {
    pub fn parse(dtb: usize) -> Self {
        let fdt = unsafe{ Fdt::from_ptr(dtb as *const u8) }.unwrap();
        let memory = fdt.memory();
        let mut meta = MachineMeta::default();
        for region in memory.regions() {
            meta.physical_memory_offset = region.starting_address as usize;
            meta.physical_memory_size = region.size.unwrap();
        }
        // probe virtio mmio device
        for node in fdt.find_all_nodes("/soc/virtio_mmio") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let paddr = reg.starting_address as usize;
                let size = reg.size.unwrap();
                let vaddr = paddr;
                unsafe{
                    let header = vaddr as *const u32;
                    let device_id_addr = header.add(2);
                    let device_id = core::ptr::read_volatile(device_id_addr);
                    if device_id != 0 {
                        hdebug!("virtio mmio addr: {:#x}, size: {:#x}", paddr, size);
                        meta.virtio.push(
                            Device { base_address: paddr, size }
                        )
                    }
                }
            }
        }
        meta.virtio.sort_unstable_by_key(|v| v.base_address);

        // probe virt test
        for node in fdt.find_all_nodes("/soc/test") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let base_addr = reg.starting_address as usize;
                let size = reg.size.unwrap();
                hdebug!("test addr: {:#x}, size: {:#x}", base_addr, size);
                meta.test_finisher_address = Some(Device { base_address: base_addr, size});
            }
        }
        meta
    }
}


}

use riscv::register::hvip;
use crate::constants::csr::{hedeleg, hideleg, hcounteren};

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

    hcounteren::write(0xffff_ffff);

    hdebug!("Initialize hypervisor environment");

}