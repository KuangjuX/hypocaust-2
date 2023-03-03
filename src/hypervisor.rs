pub mod stack {
    use crate::{constants::{
        PAGE_SIZE, KERNEL_STACK_SIZE,
        layout::TRAP_CONTEXT
    }, mm::MapPermission};
    use crate::mm::MemorySet;
    use super::HOST_VMM;
    pub struct HypervisorStack(pub usize);

    pub fn hstack_position(guest_id: usize) -> (usize, usize) {
        let top = TRAP_CONTEXT - guest_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
        let bottom = top - KERNEL_STACK_SIZE;
        (bottom, top)
    }

    pub fn hstack_alloc(guest_id: usize) -> HypervisorStack {
        let (hstack_bottom, hstack_top) = hstack_position(guest_id);
        hdebug!("allocated hstack: [{:#x}: {:#x})",hstack_bottom, hstack_top);
        let mut host_vmm = unsafe{ HOST_VMM.get_mut().unwrap().lock() };
        host_vmm.hpm.insert_framed_area(
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

    pub uart: Option<Device>,

    pub clint: Option<Device>,

    pub plic: Option<Device>
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

        // probe uart device
        for node in fdt.find_all_nodes("/soc/uart") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let base_addr = reg.starting_address as usize;
                let size = reg.size.unwrap();
                hdebug!("UART addr: {:#x}, size: {:#x}", base_addr, size);
                meta.uart = Some(Device { base_address: base_addr, size});
            }
        }

        // probe clint(core local interrupter)
        for node in fdt.find_all_nodes("/soc/clint") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let base_addr = reg.starting_address as usize;
                let size = reg.size.unwrap();
                hdebug!("CLINT addr: {:#x}, size: {:#x}", base_addr, size);
                meta.clint = Some(Device { base_address: base_addr, size});
            }
        }

        // probe plic
        for node in fdt.find_all_nodes("/soc/plic") {
            if let Some(reg) = node.reg().and_then(|mut reg| reg.next()) {
                let base_addr = reg.starting_address as usize;
                let size = reg.size.unwrap();
                hdebug!("PLIC addr: {:#x}, size: {:#x}", base_addr, size);
                meta.plic = Some(Device { base_address: base_addr, size});
            }
        }

        meta
    }
}


}


use arrayvec::ArrayVec;
use riscv::register::{ hvip, sie };
use spin::{ Once, Mutex };
use crate::constants::MAX_GUESTS;
use crate::constants::csr::{hedeleg, hideleg, hcounteren};
use crate::device_emu::plic::PlicState;
use crate::guest::{ page_table::GuestPageTable, Guest };
use crate::page_table::{ PageTable, PageTableSv39 };
use crate::mm::HostMemorySet;

use self::fdt::MachineMeta;


pub static mut HOST_VMM: Once<Mutex<HostVmm<PageTableSv39, PageTableSv39>>> = Once::new();

pub struct HostVmm<P: PageTable, G: GuestPageTable> {
    pub host_machine: MachineMeta,
    /// hypervisor memory
    pub hpm: HostMemorySet<P>,
    /// all guest structs
    pub guests: ArrayVec<Option<Guest<G>>, MAX_GUESTS>,
    /// current run guest id(single core)
    pub guest_id: usize,
    /// hypervisor emulated plic
    pub host_plic: Option<PlicState>,

    pub irq_pending: bool,
}

pub fn add_guest_queue(guest: Guest<PageTableSv39>) {
    let host_vmm = unsafe{ HOST_VMM.get_mut().unwrap() };
    let mut host_vmm = host_vmm.lock();
    let guest_id = guest.guest_id;
    assert!(guest_id < MAX_GUESTS);
    host_vmm.guests[guest_id] = Some(guest);
}


pub unsafe fn init_vmm(hpm: HostMemorySet<PageTableSv39>, host_machine: MachineMeta) {
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

    // When the hypervisor is initialized, it is necessary to write the `hcounteren` register to all 1, because it is possible to read the `time` register in VU mode or VS mode.(refs: The counter-enable register `hcounteren` is a 32-bit register that controls the availability of the hardware performance monitoring counters to the guest virtual machine.  
    // When the CY, TM, IR, or HPMn bit in the hcounteren register is clear, attempts to read the
    // cycle, time, instret, or hpmcountern register while V=1 will cause a virtual instruction exception
    // if the same bit in mcounteren is 1. When one of these bits is set, access to the corresponding register
    // is permitted when V=1, unless prevented for some other reason. In VU-mode, a counter is not
    // readable unless the applicable bits are set in both `hcounteren` and `scounteren`.  
    // `hcounteren` must be implemented. However, any of the bits may be read-only zero, indicating
    // reads to the corresponding counter will cause an exception when V=1. Hence, they are effectively
    // WARL fields.) 
    hcounteren::write(0xffff_ffff);

    // enable all interupts
    sie::set_sext();
    sie::set_ssoft();
    sie::set_stimer();

    

    // initialize HOST_VMM
    HOST_VMM.call_once(|| {
        let mut guests: ArrayVec<Option<Guest<PageTableSv39>>, MAX_GUESTS> = ArrayVec::new_const();
        for _ in 0..MAX_GUESTS{
            guests.push(None)
        }

        let host_plic;
        if let Some(plic) = host_machine.clone().plic {
            host_plic = Some(PlicState::new(plic.base_address));
        }else{
            host_plic = None;
        }
        Mutex::new(
            HostVmm { 
                host_machine,
                hpm,
                guests,
                guest_id: 0,
                host_plic,
                irq_pending: false
            }
        )
    });

    hdebug!("Initialize hypervisor environment");

}