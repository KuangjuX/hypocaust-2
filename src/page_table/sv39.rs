use crate::guest::page_table::GuestPageTable;
use crate::hyp_alloc::{ FrameTracker, frame_alloc };

use super::{ PhysPageNum, VirtPageNum, PageTable, PageTableLevel, PTEFlags, PageTableEntry, PteWrapper, PageWalk };

use alloc::vec::Vec;
use alloc::vec;

#[derive(Clone)]
pub struct PageTableSv39 {
    pub root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>
}

impl PageTableSv39 {
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }

    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }
}



impl GuestPageTable for PageTableSv39 {
    /// 新建 guest 根目录页表,需要分配 16 KiB 的内存
    /// 并且 16 KiB 内存对齐
    fn new_guest() -> Self {
        let mut frames = vec![];
        let mut root_frame = frame_alloc().unwrap();
        while root_frame.ppn.0 & 0x3 != 0 {
            hdebug!("page {:#x} was allocated, but is does not follow 16KiB buundary.", root_frame.ppn.0);
            frames.push(root_frame);
            root_frame = frame_alloc().unwrap();
        }
        hdebug!("Guest root page table: {:#x}", root_frame.ppn.0);
        let root_ppn = root_frame.ppn;
        frames.push(root_frame);
        for _ in 0..3 {
            frames.push(frame_alloc().unwrap());
        }
        Self {
            root_ppn: root_ppn,
            frames
        }
    }
}

impl PageTable for PageTableSv39 {
    fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTableSv39 {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }
    /// Temporarily used to get arguments from user space.
    fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }

    fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }

    #[allow(unused)]
    fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    
    #[allow(unused)]
    fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }

    fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }

    fn translate_va(&self, va: usize) -> Option<usize> {
        let offset = va & 0xfff;
        let vpn = VirtPageNum::from(va >> 12);
        if let Some(pte) = self.translate(vpn) {
            Some(pte.ppn().0 << 12 + offset)
        }else{
            None
        }
    }

    fn walk_page_table<R: Fn(usize) -> usize>(root: usize, va: usize, read_pte: R) -> Option<PageWalk> {
        let mut path = Vec::new();
        let mut page_table = root;
        for level in 0..3 {
            let pte_index = (va >> (30 - 9 * level)) & 0x1ff;
            let pte_addr = page_table + pte_index * 8;
            let pte = read_pte(pte_addr);
            htracking!("ppn: {:#x}", pte & 0x3ff_ffff_ffff);
            let level = match level {
                0 => PageTableLevel::Level1GB,
                1 => PageTableLevel::Level2MB,
                2 => PageTableLevel::Level4KB,
                _ => unreachable!(),
            };
            let pte = PageTableEntry{ bits: pte};
            path.push(PteWrapper{ addr: pte_addr, pte, level});

            if !pte.is_valid() || (pte.writable() && !pte.readable()){ return None; }
            else if pte.readable() | pte.executable() {
                let pa = match level {
                    PageTableLevel::Level4KB => ((pte.bits >> 10) << 12) | (va & 0xfff),
                    PageTableLevel::Level2MB => ((pte.bits >> 19) << 21) | (va & 0x1fffff),
                    PageTableLevel::Level1GB => ((pte.bits >> 28) << 30) | (va & 0x3fffffff),
                };
                return Some(super::PageWalk { path, pa});
            }else{
                page_table = (pte.bits >> 10) << 12;
            }
        }
        None
    }

}