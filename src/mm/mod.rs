mod memory_set;

pub use memory_set::{MemorySet, MapArea, remap_test, MapPermission};

use crate::shared::SHARED_DATA;

pub fn enable_paging() {
    let sharded_data = unsafe{ SHARED_DATA.get().unwrap().lock() };
    // sharded_data.hpm.map_gpm(gpm);
    sharded_data.hpm.activate();
    drop(sharded_data);
    hdebug!("Hypervisor enable paging!");
}