use alloc::vec;
use alloc::vec::Vec;
use crate::config::PAGE_SIZE;
use crate::mm::address::{PhysPageNum, VirtAddr, VirtPageNum};
use crate::mm::frame_alloc;
use crate::mm::page_table::{PTEFlags, PageTable};
use crate::mm::vpn_range::VpnRange;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical,               // 恒等映射
    Framed,                  // 分配新帧
    Fixed(PhysPageNum),      // 映射到指定物理帧
}

pub struct MapArea {
    pub vpn_range: VpnRange,
    pub start_va: VirtAddr,  // 精确的起始虚拟地址（可能非页对齐）
    pub map_type: MapType,
    pub map_perm: PTEFlags,
    data_frames: Vec<PhysPageNum>,
}

impl MapArea {
    pub fn new(start_va: VirtAddr, end_va: VirtAddr, map_type: MapType, flags: PTEFlags) -> Self {
        Self {
            vpn_range: VpnRange::new(start_va.floor(), end_va.ceil()),
            start_va,
            map_type,
            map_perm: flags,
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
            start_va: va,
            map_type,
            map_perm: flags,
            data_frames: vec![frame],
        }
    }

    pub fn from_another(another: &MapArea) -> Self {
        Self {
            vpn_range: VpnRange::new(another.vpn_range.get_start(), another.vpn_range.get_end()),
            start_va: another.start_va,
            data_frames: Vec::new(),
            // ★ Fixed 映射不能跨地址空间共享，必须分配新帧
            map_type: match another.map_type {
                MapType::Fixed(_) => MapType::Framed,
                other => other,
            },
            map_perm: another.map_perm,
        }
    }

    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range.iter() {
            self.map_one(page_table, vpn);
        }
    }

    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range.iter() {
            page_table.unmap(vpn);
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
        page_table.map(vpn, ppn, self.map_perm);
    }

    pub fn copy_data(&mut self, page_table: &PageTable, data: &[u8]) {
        assert_eq!(self.map_type, MapType::Framed);
        let start_va_val = self.start_va.0;
        let len = data.len();

        // 逐段拷贝，保证每次写入不跨越物理页边界
        // （相邻的虚拟页不一定映射到相邻的物理帧）
        let mut offset: usize = 0;
        while offset < len {
            let cur_va = VirtAddr(start_va_val + offset);
            let page_off = cur_va.page_offset();
            // 当前页还能写入的字节数（不超过页边界）
            let chunk_len = core::cmp::min(PAGE_SIZE - page_off, len - offset);

            if let Some(pa) = page_table.translate_va(cur_va) {
                let dst = unsafe {
                    core::slice::from_raw_parts_mut(pa.0 as *mut u8, chunk_len)
                };
                dst.copy_from_slice(&data[offset..offset + chunk_len]);
            } else {
                panic!(
                    "copy_data: translate_va failed at va={:#x} (offset={}, len={})",
                    cur_va.0, offset, len
                );
            }
            offset += chunk_len;
        }
    }
}
