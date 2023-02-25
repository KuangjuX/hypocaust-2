mod memory_set;

pub use memory_set::{HostMemorySet, GuestMemorySet, MapArea, remap_test, MapPermission};

use core::arch::asm;
use memory_set::MapType;
use crate::guest::page_table::GuestPageTable;
use crate::page_table::{VirtAddr, PageTable, VirtPageNum, PageTableEntry, PhysAddr, PTEFlags};
use crate::constants::layout::TRAMPOLINE;
use crate::hypervisor::HOST_VMM;

pub fn enable_paging() {
    let host_vmm = unsafe{ HOST_VMM.get().unwrap().lock() };
    host_vmm.hpm.activate();
    drop(host_vmm);
    hdebug!("Hypervisor enable paging!");
}

pub trait MemorySet<P: PageTable>{
    fn token(&self) -> usize;
    fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    );
    fn push(
        &mut self, 
        map_area: MapArea<P>, 
        data: Option<&[u8]>
    );

    fn map_trampoline(&mut self);
    fn activate(&self);
    fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry>;
    fn translate_va(&self, va: usize) -> Option<usize>;
}

impl<P: PageTable> MemorySet<P> for HostMemorySet<P> {
    fn token(&self) -> usize {
        self.page_table.token()
    }

    /// Assume that no conflicts.
    fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        self.push(
            MapArea::new(start_va, end_va,  None, None, MapType::Framed, permission),
            None,
        );
    }

    /// 将内存区域 push 到页表中，并映射内存区域
    fn push(&mut self, mut map_area: MapArea<P>, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&mut self.page_table, data);
        }
        self.areas.push(map_area);
    }

    /// Mention that trampoline is not collected by areas.
    fn map_trampoline(&mut self) {
        extern "C" {
            fn strampoline();
        }
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }

    /// 激活根页表
    fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            asm!(
                "csrw satp, {hgatp}",
                "sfence.vma",
                hgatp = in(reg) satp
            );
        }
    }
    
    /// 将虚拟页号翻译成页表项
    fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }

    fn translate_va(&self, va: usize) -> Option<usize> {
        self.page_table.translate_va(va)
    }
}

impl<P: GuestPageTable> MemorySet<P> for GuestMemorySet<P> {
    fn token(&self) -> usize {
        self.page_table.token()
    }

    /// Assume that no conflicts.
    fn insert_framed_area(
        &mut self,
        start_va: VirtAddr,
        end_va: VirtAddr,
        permission: MapPermission,
    ) {
        self.push(
            MapArea::new(start_va, end_va,  None, None, MapType::Framed, permission),
            None,
        );
    }

    /// 将内存区域 push 到页表中，并映射内存区域
    fn push(&mut self, mut map_area: MapArea<P>, data: Option<&[u8]>) {
        map_area.map(&mut self.page_table);
        if let Some(data) = data {
            map_area.copy_data(&mut self.page_table, data);
        }
        self.areas.push(map_area);
    }

    /// Mention that trampoline is not collected by areas.
    fn map_trampoline(&mut self) {
        extern "C" {
            fn strampoline();
        }
        self.page_table.map(
            VirtAddr::from(TRAMPOLINE).into(),
            PhysAddr::from(strampoline as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );
    }

    /// 激活根页表
    fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            asm!(
                "csrw satp, {hgatp}",
                "sfence.vma",
                hgatp = in(reg) satp
            );
        }
    }
    
    /// 将虚拟页号翻译成页表项
    fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.page_table.translate(vpn)
    }

    fn translate_va(&self, va: usize) -> Option<usize> {
        self.page_table.translate_va(va)
    }
}
