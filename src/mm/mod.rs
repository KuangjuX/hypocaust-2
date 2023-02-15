mod memory_set;

pub use memory_set::{MemorySet, MapArea, remap_test};

use crate::{shared::SHARED_DATA, page_table::PageTableSv39};

pub fn enable_paging(gpm: &MemorySet<PageTableSv39>) {
    let sharded_data = SHARED_DATA.lock();
    sharded_data.hpm.map_gpm(gpm);
    sharded_data.hypervisor_memory_set.activate();
    drop(sharded_data);
    hdebug!("Hypervisor enable paging!");
}