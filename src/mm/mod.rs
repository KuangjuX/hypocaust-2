mod memory_set;

pub use memory_set::{MemorySet, MapArea, remap_test, MapPermission};
use crate::hypervisor::HOST_VMM;

pub fn enable_paging() {
    let host_vmm = unsafe{ HOST_VMM.get().unwrap().lock() };
    host_vmm.hpm.activate();
    drop(host_vmm);
    hdebug!("Hypervisor enable paging!");
}