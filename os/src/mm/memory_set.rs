use crate::mm::address::{PhysPageNum, VirtPageNum};
use crate::mm::page_table::{PTEFlags, PageTable};
use crate::mm::MapArea;
use crate::mm::MapType;
use alloc::vec::Vec;

pub struct MemorySet {
    pub page_table: PageTable,
    pub areas: Vec<MapArea>,
}

impl MemorySet {
    pub fn new() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    pub fn push(&mut self, mut area: MapArea, data: Option<&[u8]>) {
        area.map(&mut self.page_table);
        if let Some(data) = data {
            area.copy_data(&self.page_table, data);
        }
        self.areas.push(area);
    }

    /// 将指定的物理帧映射到给定虚拟地址 (用于 trampoline)
    pub fn push_trampoline(&mut self, frame: PhysPageNum, va: usize) {
        let vpn: VirtPageNum = va.into();
        self.page_table.map(
            vpn,
            frame,
            PTEFlags::R | PTEFlags::X,
        );
        self.areas.push(MapArea::new_one_page(
            va.into(),
            frame,
            MapType::Framed,
            PTEFlags::R | PTEFlags::X,
        ));
    }

    pub fn satp(&self) -> usize {
        self.page_table.satp()
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PhysPageNum> {
        self.page_table.translate(vpn).map(|pte| pte.ppn())
    }

    /// 为用户地址空间映射 trampoline 页面
    pub fn map_trampoline(&mut self, trampoline_frame: PhysPageNum, trampoline_va: usize) {
        let vpn: VirtPageNum = trampoline_va.into();
        self.page_table.map(vpn, trampoline_frame, PTEFlags::R | PTEFlags::X);
        self.areas.push(MapArea::new_one_page(
            trampoline_va.into(),
            trampoline_frame,
            MapType::Framed,
            PTEFlags::R | PTEFlags::X,
        ));
    }
}
