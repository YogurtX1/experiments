use crate::mm::address::{PhysAddr, PhysPageNum, VirtAddr, VirtPageNum};
use crate::config::PAGE_SIZE;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use core::fmt::{self, Debug};

bitflags::bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        Self {
            bits: (ppn.0 << 10) | (flags.bits() as usize),
        }
    }
    pub fn empty() -> Self {
        Self { bits: 0 }
    }
    pub fn ppn(&self) -> PhysPageNum {
        PhysPageNum((self.bits >> 10) & ((1usize << 44) - 1))
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.bits as u8)
    }
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
}

impl Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PTE")
            .field("ppn", &self.ppn())
            .field("flags", &self.flags())
            .field("raw", &format_args!("{:#x}", self.bits))
            .finish()
    }
}

pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);
    let mut vec = Vec::new();
    let mut start = ptr as usize;
    let end = start + len;
    while start < end {
        let va = VirtAddr(start);
        let page_end = (start / PAGE_SIZE + 1) * PAGE_SIZE;
        let chunk_len = core::cmp::min(page_end, end) - start;
        let pa = page_table.translate_va(va).expect("translated_byte_buffer: unmapped");
        vec.push(unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, chunk_len) });
        start += chunk_len;
    }
    vec
}

pub fn translated_str(token: usize, ptr: *const u8) -> String {
    let page_table = PageTable::from_token(token);
    let mut string = String::new();
    let mut va = ptr as usize;
    loop {
        let ch: u8 = *(page_table.translate_va(VirtAddr::from(va)).unwrap().get_mut());
        if ch == 0 {
            break;
        } else {
            string.push(ch as char);
            va += 1;
        }
    }
    string
}

pub fn translated_refmut<T>(token: usize, ptr: *mut T) -> &'static mut T {
    let page_table = PageTable::from_token(token);
    let va = ptr as usize;
    page_table.translate_va(VirtAddr::from(va)).unwrap().get_mut()
}

pub struct PageTable {
    pub root_ppn: PhysPageNum,
    frames: Vec<PhysPageNum>,
}

impl PageTable {
    pub fn new() -> Self {
        let frame = crate::mm::frame_alloc().unwrap();
        let bytes = frame.get_bytes_array();
        bytes.fill(0);

        Self {
            root_ppn: frame,
            frames: vec![frame],
        }
    }

    pub fn satp(&self) -> usize {
        (8usize << 60) | self.root_ppn.0
    }

    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }

    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;

        for (i, &idx) in idxs.iter().enumerate() {
            let ptes: &mut [PageTableEntry] =
                unsafe { core::slice::from_raw_parts_mut(ppn.get_mut::<PageTableEntry>(), 512) };
            if i == 2 {
                return Some(&mut ptes[idx]);
            }
            if !ptes[idx].is_valid() {
                // 中间级页表必须 2MB 对齐（Sv39 非叶 PTE 使用 PPN[2:1] 共 35 bits）
                let frame = crate::mm::alloc_pte_frame()?;
                let bytes = frame.get_bytes_array();
                bytes.fill(0);
                ptes[idx] = PageTableEntry::new(frame, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = ptes[idx].ppn();
        }
        None
    }

    /// 使用 2MB 大页映射一个 2MB 对齐的区域 (直接写到 level-1 PTE)
    /// vpn 和 ppn 的低 9 位必须为 0
    pub fn map_huge(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        assert!(
            vpn.0 & 0x1FF == 0 && ppn.0 & 0x1FF == 0,
            "map_huge: vpn({:#x}) or ppn({:#x}) not 2MB-aligned",
            vpn.0, ppn.0
        );
        let idxs = vpn.indexes();
        let root_ppn = self.root_ppn;

        // Level-2: 找到或创建 level-1 页表（必须 2MB 对齐）
        let root_ptes: &mut [PageTableEntry] =
            unsafe { core::slice::from_raw_parts_mut(root_ppn.get_mut::<PageTableEntry>(), 512) };
        if !root_ptes[idxs[0]].is_valid() {
            let frame = crate::mm::alloc_pte_frame().expect("map_huge: alloc level-1 frame");
            frame.get_bytes_array().fill(0);
            root_ptes[idxs[0]] = PageTableEntry::new(frame, PTEFlags::V);
            self.frames.push(frame);
        }

        // Level-1: 设置为 2MB 大页叶子条目
        let l1_ppn = root_ptes[idxs[0]].ppn();
        let l1_ptes: &mut [PageTableEntry] =
            unsafe { core::slice::from_raw_parts_mut(l1_ppn.get_mut::<PageTableEntry>(), 512) };
        assert!(
            !l1_ptes[idxs[1]].is_valid(),
            "map_huge: level-1 entry {} already valid",
            idxs[1]
        );

        // Sv39 2MB 大页编码: ppn[2] 在 bits[53:28], ppn[1] 在 bits[27:19], ppn[0] = 0
        let ppn_val = ppn.0;
        let huge_bits = ((ppn_val >> 18) << 28)
            | (((ppn_val >> 9) & 0x1FF) << 19)
            | (flags.bits() | PTEFlags::V.bits()) as usize;
        l1_ptes[idxs[1]] = PageTableEntry { bits: huge_bits };
    }

    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        if pte.is_valid() {
            // 页面已被映射（例如内核各段的边界未页对齐导致重叠），跳过
            return;
        }
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }

    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is not mapped", vpn);
        *pte = PageTableEntry::empty();
    }

    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;

        for (i, &idx) in idxs.iter().enumerate() {
            let ptes: &[PageTableEntry] =
                unsafe { core::slice::from_raw_parts(ppn.get_mut::<PageTableEntry>(), 512) };
            if !ptes[idx].is_valid() {
                return None;
            }
            if i == 2 {
                return Some(ptes[idx]);
            }
            ppn = ptes[idx].ppn();
        }
        None
    }

    pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.translate(va.floor()).map(|pte| {
            PhysAddr(PhysAddr::from(pte.ppn()).0 + va.page_offset())
        })
    }
}
