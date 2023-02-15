use crate::mm::MemorySet;
use crate::page_table::{PageTable, PageTableSv39};
use crate::guest::Guest;
use alloc::collections::BTreeMap;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref SHARED_DATA: Mutex<SharedData<PageTableSv39>> = Mutex::new(
        SharedData {
            hpm: MemorySet::new_kernel(),
            guests: BTreeMap::new()
        }
    );
}

/// 多核间共享数据
pub struct SharedData<P: PageTable> {
    /// hypervisor memory
    pub hpm: MemorySet<P>,
    /// all guest structs
    pub guests: BTreeMap<usize, Guest<P>>
}

pub fn add_guest(guest: Guest<PageTableSv39>) {
    let mut sharded_data = SHARED_DATA.lock();
    let old = sharded_data.guests.insert(guest.guest_id, guest);
    core::mem::forget(old);
    drop(sharded_data);
}