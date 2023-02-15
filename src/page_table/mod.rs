mod address;
mod pte;

use alloc::vec::Vec;

pub use pte::{ PTEFlags, PageTableEntry };
pub use address::{ PhysPageNum, VirtPageNum, PhysAddr, VirtAddr };

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


pub trait PageTable: Clone {
    /// build new bare page table
    fn new() -> Self;
    /// build page table from
    fn from_token(satp: usize) -> Self;
    /// map virt page into phys page
    fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags);
    /// unmap virt page
    fn unmap(&mut self, vpn: VirtPageNum);
    
}