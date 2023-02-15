mod memory_set;

pub use memory_set::{MemorySet, MapArea, remap_test};

use crate::shared::SHARED_DATA;

pub fn enable_paging() {
    let sharded_data = SHARED_DATA.lock();
    sharded_data.hypervisor_memory_set.activate();
    drop(sharded_data);
    hdebug!("Hypervisor enable paging!");
}