use crate::{mm::MemorySet, page_table::{PageTable, PageTableSv39}};
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref SHARED_DATA: Mutex<SharedData<PageTableSv39>> = Mutex::new(
        SharedData {
            hypervisor_memory_set: MemorySet::new_kernel()
        }
    );
}

/// 多核间共享数据
pub struct SharedData<P: PageTable> {
    pub hypervisor_memory_set: MemorySet<P>
}