mod address;
mod pte;
mod sv39;

use alloc::vec::Vec;

pub use pte::{ PTEFlags, PageTableEntry };
pub use address::{ PhysPageNum, VirtPageNum, PhysAddr, VirtAddr, StepByOne, VPNRange, PPNRange };
pub use sv39::PageTableSv39;

use crate::guest::page_table::GuestPageTable;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PageTableLevel {
    Level4KB,
    Level2MB,
    Level1GB
}

#[derive(Debug)]
pub struct PteWrapper {
    pub addr: usize,
    pub pte: PageTableEntry,
    pub level: PageTableLevel
}

#[derive(Debug)]
pub struct PageWalk {
    pub path: Vec<PteWrapper>,
    pub pa: usize
}

#[derive(Debug)]
pub struct AddressTranslation {
    pub pte: PageTableEntry,
    pub pte_addr: usize,
    pub guest_pa: usize,
    pub level: PageTableLevel,
    pub page_walk: PageWalk
}



pub trait PageTable: Clone {
    /// build new bare page table
    fn new() -> Self;
    /// build page table from
    fn from_token(satp: usize) -> Self;
    /// map virt page into phys page
    fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags);
    /// unmap virt page
    fn unmap(&mut self, vpn: VirtPageNum);
    /// page walk and renturn all walked ptes
    fn walk_page_table<R: Fn(usize) -> usize>(root: usize, va: usize, read_pte: R) -> Option<PageWalk>;
    /// translate virt page into physical page
    fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry>;
    /// translate virt address into physical address
    fn translate_va(&self, va: usize) -> Option<usize>;
    /// get page table root token
    fn token(&self) -> usize;
}

pub fn translate_guest_va<P: GuestPageTable>(_guest_id: usize, root: usize, guest_va: usize) -> Option<AddressTranslation> {
    P::walk_page_table(root, guest_va, |va| {
        // let pa = gpa2hpa(va, guest_id);
        let pa = va;
        unsafe{ core::ptr::read(pa as *const usize) }
    }).map(|t| {
        AddressTranslation { 
            pte: t.path[t.path.len() - 1].pte,
            pte_addr: t.path[t.path.len() - 1].addr,
            level: t.path[t.path.len() - 1].level,
            guest_pa: t.pa,
            page_walk: t
        }
    })
}