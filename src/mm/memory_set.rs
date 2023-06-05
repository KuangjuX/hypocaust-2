//! Implementation of [`MapArea`] and [`MemorySet`].

use super::MemorySet;
use crate::constants::{
    layout::{MEMORY_END, TRAMPOLINE, TRAP_CONTEXT},
    PAGE_SIZE,
};
use crate::guest::page_table::GuestPageTable;
use crate::hyp_alloc::{frame_alloc, FrameTracker};
use crate::hypervisor::HOST_VMM;
use crate::page_table::{PPNRange, StepByOne, VPNRange};
use crate::page_table::{PTEFlags, PageTable};
use crate::page_table::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::arch::asm;
use core::marker::PhantomData;

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
    fn sinitrd();
    fn einitrd();
}

/// memory set structure, controls virtual-memory space
pub struct HostMemorySet<P: PageTable> {
    pub page_table: P,
    pub areas: Vec<MapArea<P>>,
}

pub struct GuestMemorySet<G: GuestPageTable> {
    pub page_table: G,
    pub areas: Vec<MapArea<G>>,
}

impl<P: PageTable> HostMemorySet<P> {
    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    /// Without kernel stacks.
    /// 内核虚拟地址映射
    /// 映射了内核代码段和数据段以及跳板页，没有映射内核栈
    pub fn new_host_vmm() -> Self {
        let mut hpm = Self::new_bare();
        // map trampoline
        hpm.map_trampoline();

        // 这里注意了,需要单独映射 Trap Context,因为在上下文切换时
        // 我们是不切换页表的
        hpm.push(
            MapArea::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                None,
                None,
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        // map kernel sections
        hpm.push(
            MapArea::new(
                (stext as usize).into(),
                (etext as usize).into(),
                Some((stext as usize).into()),
                Some((etext as usize).into()),
                MapType::Linear,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );

        hpm.push(
            MapArea::new(
                (srodata as usize).into(),
                (erodata as usize).into(),
                Some((srodata as usize).into()),
                Some((erodata as usize).into()),
                MapType::Linear,
                MapPermission::R,
            ),
            None,
        );

        hpm.push(
            MapArea::new(
                (sdata as usize).into(),
                (edata as usize).into(),
                Some((sdata as usize).into()),
                Some((edata as usize).into()),
                MapType::Linear,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        hpm.push(
            MapArea::new(
                (sbss_with_stack as usize).into(),
                (ebss as usize).into(),
                Some((sbss_with_stack as usize).into()),
                Some((ebss as usize).into()),
                MapType::Linear,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        hpm.push(
            MapArea::new(
                (ekernel as usize).into(),
                MEMORY_END.into(),
                Some((ekernel as usize).into()),
                Some(MEMORY_END.into()),
                MapType::Linear,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        use crate::board::MMIO;
        for (start, size) in MMIO.iter() {
            hpm.push(
                MapArea::new(
                    VirtAddr(*start),
                    VirtAddr(start + size),
                    Some(PhysAddr(*start)),
                    Some(PhysAddr(start + size)),
                    MapType::Linear,
                    MapPermission::R | MapPermission::W,
                ),
                None,
            );
        }
        hpm
    }

    /// 激活根页表
    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            asm!(
                "csrw satp, {satp}",
                "sfence.vma",
                satp = in(reg) satp
            );
        }
    }

    pub fn map_guest(&mut self, start_pa: usize, gpm_size: usize) {
        // map dtb
        self.push(
            MapArea::new(
                VirtAddr(0x9000_0000),
                VirtAddr(0x9020_0000),
                Some(PhysAddr(0x9000_0000)),
                Some(PhysAddr(0x9020_0000)),
                MapType::Linear,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
        self.push(
            MapArea::new(
                start_pa.into(),
                (start_pa + gpm_size).into(),
                Some(start_pa.into()),
                Some((start_pa + gpm_size).into()),
                MapType::Linear,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );
    }

    /// 加载客户操作系统
    pub fn map_gpm(&mut self, gpm: &GuestMemorySet<impl GuestPageTable>) {
        for area in gpm.areas.iter() {
            // 修改虚拟地址与物理地址相同
            let ppn_range = area.ppn_range.unwrap();
            let start_pa: PhysAddr = ppn_range.get_start().into();
            let end_pa: PhysAddr = ppn_range.get_end().into();
            let start_va: usize = start_pa.into();
            let end_va: usize = end_pa.into();
            let new_area = MapArea::new(
                start_va.into(),
                end_va.into(),
                Some(start_pa),
                Some(end_pa),
                area.map_type,
                area.map_perm,
            );
            self.push(new_area, None);
        }
    }
}

impl<G: GuestPageTable> GuestMemorySet<G> {
    /// 为 guest page table 新建根页表
    /// 需要分配 16 KiB 对齐的页表
    pub fn new_guest_bare() -> Self {
        Self {
            page_table: GuestPageTable::new_guest(),
            areas: Vec::new(),
        }
    }

    pub fn setup_gpm() -> Self {
        use crate::board::{GUEST_BIN_ADDR, GUEST_BIN_SIZE};
        let mut gpm = Self::new_guest_bare();

        info!(
            "map guest: [{:#x}: {:#x}]",
            GUEST_BIN_ADDR,
            GUEST_BIN_ADDR + GUEST_BIN_SIZE
        );

        // map dtb
        gpm.push(
            MapArea::new(
                VirtAddr(0x9000_0000),
                VirtAddr(0x9020_0000),
                Some(PhysAddr(0x9000_0000)),
                Some(PhysAddr(0x9020_0000)),
                MapType::Linear,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );
        gpm.push(
            MapArea::new(
                VirtAddr(GUEST_BIN_ADDR),
                VirtAddr(GUEST_BIN_ADDR + GUEST_BIN_SIZE),
                Some(PhysAddr(GUEST_BIN_ADDR)),
                Some(PhysAddr(GUEST_BIN_ADDR + GUEST_BIN_SIZE)),
                MapType::Linear,
                MapPermission::R | MapPermission::W | MapPermission::U | MapPermission::X,
            ),
            None,
        );

        gpm.map_trampoline();

        use crate::board::MMIO;
        for (start, size) in MMIO.iter() {
            if *start == 0x0c00_0000 {
                gpm.push(
                    MapArea::new(
                        VirtAddr(*start),
                        VirtAddr(*start + 0x0020_0000),
                        Some(PhysAddr(*start)),
                        Some(PhysAddr(*start + 0x0020_0000)),
                        MapType::Linear,
                        MapPermission::R | MapPermission::W | MapPermission::U,
                    ),
                    None,
                );
            } else {
                gpm.push(
                    MapArea::new(
                        VirtAddr(*start),
                        VirtAddr(start + size),
                        Some(PhysAddr(*start)),
                        Some(PhysAddr(start + size)),
                        MapType::Linear,
                        MapPermission::R | MapPermission::W | MapPermission::U,
                    ),
                    None,
                );
            }

            info!("map mmio: [{:#x}: {:#x}]", *start, start + size);
        }
        gpm
    }
}

/// map area structure, controls a contiguous piece of virtual memory
#[derive(Clone)]
pub struct MapArea<P: PageTable> {
    pub vpn_range: VPNRange,
    pub ppn_range: Option<PPNRange>,
    pub data_frames: BTreeMap<VirtPageNum, FrameTracker>,
    pub map_type: MapType,
    pub map_perm: MapPermission,
    _marker: PhantomData<P>,
}

impl<P> MapArea<P>
where
    P: PageTable,
{
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        start_pa: Option<PhysAddr>,
        end_pa: Option<PhysAddr>,
        map_type: MapType,
        map_perm: MapPermission,
    ) -> Self {
        let start_vpn: VirtPageNum = start_va.floor();
        let end_vpn: VirtPageNum = end_va.ceil();
        if let (Some(start_pa), Some(end_pa)) = (start_pa, end_pa) {
            let start_ppn = start_pa.floor();
            let end_ppn = end_pa.ceil();
            return Self {
                vpn_range: VPNRange::new(start_vpn, end_vpn),
                ppn_range: Some(PPNRange::new(start_ppn, end_ppn)),
                data_frames: BTreeMap::new(),
                map_type,
                map_perm,
                _marker: PhantomData,
            };
        }
        Self {
            vpn_range: VPNRange::new(start_vpn, end_vpn),
            ppn_range: None,
            data_frames: BTreeMap::new(),
            map_type,
            map_perm,
            _marker: PhantomData,
        }
    }
    pub fn map_one(&mut self, page_table: &mut P, vpn: VirtPageNum, ppn_: Option<PhysPageNum>) {
        let ppn: PhysPageNum;
        match self.map_type {
            // 线性映射
            MapType::Linear => {
                ppn = ppn_.unwrap();
            }
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                ppn = frame.ppn;
                self.data_frames.insert(vpn, frame);
            }
        }
        let pte_flags = PTEFlags::from_bits(self.map_perm.bits).unwrap();
        page_table.map(vpn, ppn, pte_flags);
    }
    #[allow(unused)]
    pub fn unmap_one(&mut self, page_table: &mut P, vpn: VirtPageNum) {
        if self.map_type == MapType::Framed {
            self.data_frames.remove(&vpn);
        }
        page_table.unmap(vpn);
    }
    pub fn map(&mut self, page_table: &mut P) {
        let vpn_range = self.vpn_range;
        if let Some(ppn_range) = self.ppn_range {
            let ppn_start: usize = ppn_range.get_start().into();
            let ppn_end: usize = ppn_range.get_end().into();
            let vpn_start: usize = vpn_range.get_start().into();
            let vpn_end: usize = vpn_range.get_end().into();
            assert_eq!(ppn_end - ppn_start, vpn_end - vpn_start);
            let mut ppn = ppn_range.get_start();
            let mut vpn = vpn_range.get_start();
            loop {
                self.map_one(page_table, vpn, Some(ppn));
                ppn.step();
                vpn.step();
                if ppn == ppn_range.get_end() && vpn == vpn_range.get_end() {
                    break;
                }
            }
        } else {
            for vpn in self.vpn_range {
                self.map_one(page_table, vpn, None)
            }
        }
    }
    #[allow(unused)]
    pub fn unmap(&mut self, page_table: &mut P) {
        for vpn in self.vpn_range {
            self.unmap_one(page_table, vpn);
        }
    }
    /// data: start-aligned but maybe with shorter length
    /// assume that all frames were cleared before
    pub fn copy_data(&mut self, page_table: &mut P, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let mut start: usize = 0;
        let mut current_vpn = self.vpn_range.get_start();
        let len = data.len();
        loop {
            let src = &data[start..len.min(start + PAGE_SIZE)];
            let dst = &mut page_table
                .translate(current_vpn)
                .unwrap()
                .ppn()
                .get_bytes_array()[..src.len()];
            dst.copy_from_slice(src);
            start += PAGE_SIZE;
            if start >= len {
                break;
            }
            current_vpn.step();
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
/// map type for memory set: identical or framed
pub enum MapType {
    Framed,
    Linear,
}

bitflags! {
    /// map permission corresponding to that in pte: `R W X U`
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}

#[allow(unused)]
pub fn remap_test() {
    let host_vmm = unsafe { HOST_VMM.get().unwrap().lock() };
    let kernel_space = &host_vmm.hpm;
    let mid_text: VirtAddr = ((stext as usize + etext as usize) / 2).into();
    let mid_rodata: VirtAddr = ((srodata as usize + erodata as usize) / 2).into();
    let mid_data: VirtAddr = ((sdata as usize + edata as usize) / 2).into();

    assert!(!kernel_space
        .page_table
        .translate(mid_text.floor())
        .unwrap()
        .writable(),);
    assert!(!kernel_space
        .page_table
        .translate(mid_rodata.floor())
        .unwrap()
        .writable(),);
    assert!(!kernel_space
        .page_table
        .translate(mid_data.floor())
        .unwrap()
        .executable(),);
    unsafe { core::ptr::read(TRAMPOLINE as *const usize) };
    // 测试 guest ketnel
    debug!("remap test passed!");
    drop(host_vmm);
}
