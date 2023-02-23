use crate::guest::page_table::GuestPageTable;
use crate::mm::MemorySet;
use crate::page_table::{PageTable, PageTableSv39};
use crate::guest::Guest;
use alloc::collections::BTreeMap;
use spin::{ Mutex, Once };


pub static mut SHARED_DATA: Once<Mutex<SharedData<PageTableSv39>>> = Once::new();

/// 多核间共享数据
pub struct SharedData<P: PageTable + GuestPageTable> {
    /// hypervisor memory
    pub hpm: MemorySet<P>,
    /// all guest structs
    pub guests: BTreeMap<usize, Guest<P>>,
    pub guest_id: usize
}

pub fn add_guest(guest: Guest<PageTableSv39>) {
    let mut sharded_data = unsafe{ SHARED_DATA.get_mut().unwrap().lock() };
    let old = sharded_data.guests.insert(guest.guest_id, guest);
    core::mem::forget(old);
    drop(sharded_data);
}
