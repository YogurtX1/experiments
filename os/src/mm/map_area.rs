use alloc::vec;
use alloc::vec::Vec;
use crate::mm::address::{PhysPageNum, VirtAddr, VirtPageNum};
use crate::mm::frame_alloc;
use crate::mm::page_table::{PTEFlags, PageTable};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical,               // 恒等映射
    Framed,                  // 分配新帧
    Fixed(PhysPageNum),      // 映射到指定物理帧
}

pub struct MapArea {
    pub vpn_range: VpnRange,
    pub map_type: MapType,
    pub flags: PTEFlags,
    data_frames: Vec<PhysPageNum>,
}

#[derive(Clone)]
pub struct VpnRange {
    pub start: VirtPageNum,
    pub end: VirtPageNum,
}

impl VpnRange {
    pub fn new(start: VirtPageNum, end: VirtPageNum) -> Self {
        Self { start, end }
    }

    pub fn iter(&self) -> impl Iterator<Item = VirtPageNum> {
        (self.start.0..self.end.0).map(VirtPageNum)
    }
}

impl MapArea {
    pub fn new(start_va: VirtAddr, end_va: VirtAddr, map_type: MapType, flags: PTEFlags) -> Self {
        Self {
            vpn_range: VpnRange::new(start_va.floor(), end_va.ceil()),
            map_type,
            flags,
            data_frames: Vec::new(),
        }
    }

    /// 创建单页 MapArea (用于 trampoline、TrapContext 等固定映射)
    pub fn new_one_page(
        va: VirtAddr,
        frame: PhysPageNum,
        map_type: MapType,
        flags: PTEFlags,
    ) -> Self {
        let start = va.floor();
        let end = VirtPageNum(start.0 + 1);
        Self {
            vpn_range: VpnRange::new(start, end),
            map_type,
            flags,
            data_frames: vec![frame],
        }
    }

    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range.iter() {
            self.map_one(page_table, vpn);
        }
    }

    fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn: PhysPageNum = match self.map_type {
            MapType::Identical => PhysPageNum(vpn.0),
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                self.data_frames.push(frame);
                frame
            }
            MapType::Fixed(ppn) => ppn,
        };
        page_table.map(vpn, ppn, self.flags);
    }

    pub fn copy_data(&mut self, page_table: &PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let dst_start = self.vpn_range.start.0 * crate::config::PAGE_SIZE;
        let len = data.len();

        for offset in (0..len).step_by(crate::config::PAGE_SIZE) {
            let chunk = &data[offset..core::cmp::min(offset + crate::config::PAGE_SIZE, len)];
            let va = VirtAddr(dst_start + offset);
            if let Some(pa) = page_table.translate_va(va) {
                let dst = unsafe {
                    core::slice::from_raw_parts_mut(pa.0 as *mut u8, chunk.len())
                };
                dst.copy_from_slice(chunk);
            }
        }
    }
}
