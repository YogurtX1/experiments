use crate::config::{PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT_USER, USER_STACK_SIZE};
use crate::mm::address::{PhysPageNum, VirtAddr, VirtPageNum};
use crate::mm::page_table::{PTEFlags, PageTable};
use crate::mm::frame_alloc;
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

    pub fn new_bare() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: Vec::new(),
        }
    }

    pub fn token(&self) -> usize {
        self.satp()
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

    pub fn insert_framed_area(&mut self, start_va: VirtAddr, end_va: VirtAddr, permission: PTEFlags) {
        self.push(
            MapArea::new(start_va, end_va, MapType::Framed, permission),
            None,
        );
    }

    pub fn remove_area_with_start_vpn(&mut self, start_vpn: VirtPageNum) {
        if let Some((idx, area)) = self.areas.iter_mut().enumerate()
            .find(|(_, area)| area.vpn_range.get_start() == start_vpn) {
            area.unmap(&mut self.page_table);
            self.areas.remove(idx);
        }
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

    /// 重用全局 trampoline 物理帧为用户空间映射 trampoline
    pub fn map_trampoline_global(&mut self) {
        let trampoline_frame = crate::mm::KERNEL_SPACE
            .exclusive_access()
            .translate(TRAMPOLINE.into())
            .expect("trampoline not found in kernel page table");
        self.map_trampoline(trampoline_frame, TRAMPOLINE);
    }

    pub fn recycle_data_pages(&mut self) {
        self.areas.clear();
    }

    /// 从 ELF 数据构建完整的用户地址空间
    /// 返回 (memory_set, user_sp, entry_point)
    pub fn from_elf(elf_data: &[u8]) -> (Self, usize, usize) {
        let mut memory_set = Self::new_bare();

        // 映射 trampoline
        memory_set.map_trampoline_global();

        // 解析 ELF 程序头获取 PT_LOAD 段
        let elf = ElfFile::new(elf_data).expect("Invalid ELF file");
        let entry_point = elf.header.pt2.e_entry as usize;
        crate::println!(
            "[from_elf] entry_point={:#x}, phnum={}, phentsize={}",
            entry_point, elf.header.phnum, elf.header.phentsize
        );

        let mut max_end_vpn = VirtPageNum(0);
        for ph in elf.program_iter() {
            if ph.p_type != 1 {
                // PT_LOAD = 1
                continue;
            }
            let vaddr = ph.p_vaddr as usize;
            let memsz = ph.p_memsz as usize;
            let filesz = ph.p_filesz as usize;
            let offset = ph.p_offset as usize;
            let flags = ph.p_flags;

            // ★ 诊断: 打印 PT_LOAD 段信息
            crate::println!(
                "[from_elf] PT_LOAD: p_offset={:#x}, p_vaddr={:#x}, filesz={:#x}, memsz={:#x}, flags={:#x}",
                offset, vaddr, filesz, memsz, flags
            );

            let start_va: VirtAddr = vaddr.into();
            let end_va: VirtAddr = (vaddr + memsz).into();
            let end_vpn = end_va.ceil();

            let mut map_perm = PTEFlags::U;
            if flags & 0x4 != 0 { map_perm |= PTEFlags::R; }
            if flags & 0x2 != 0 { map_perm |= PTEFlags::W; }
            if flags & 0x1 != 0 { map_perm |= PTEFlags::X; }

            let data = &elf_data[offset..offset + filesz];
            // ★ 诊断: 打印数据段前 16 字节
            let preview_len = core::cmp::min(16, data.len());
            crate::print!("[from_elf] first {} bytes at offset={:#x}: ", preview_len, offset);
            for b in &data[..preview_len] {
                crate::print!("{:02x} ", b);
            }
            crate::println!();
            memory_set.push(
                MapArea::new(start_va, end_va, MapType::Framed, map_perm),
                Some(data),
            );

            if end_vpn.0 > max_end_vpn.0 {
                max_end_vpn = end_vpn;
            }
        }

        // 映射用户栈
        let max_end_va: usize = max_end_vpn.0 * PAGE_SIZE;
        let user_stack_bottom: usize = max_end_va;
        let user_stack_top: usize = user_stack_bottom + USER_STACK_SIZE;
        memory_set.push(
            MapArea::new(
                user_stack_bottom.into(),
                user_stack_top.into(),
                MapType::Framed,
                PTEFlags::R | PTEFlags::W | PTEFlags::U,
            ),
            None,
        );

        // 映射 TrapContext 页面
        let trap_cx_frame = frame_alloc().expect("from_elf: alloc trap_context frame failed");
        memory_set.push(
            MapArea::new_one_page(
                TRAP_CONTEXT_USER.into(),
                trap_cx_frame,
                MapType::Fixed(trap_cx_frame),
                PTEFlags::R | PTEFlags::W,
            ),
            None,
        );

        // ★ 验证关键地址映射
        let trap_cx_vpn: VirtPageNum = TRAP_CONTEXT_USER.into();
        match memory_set.translate(trap_cx_vpn) {
            Some(ppn) => crate::println!(
                "[from_elf] TRAP_CONTEXT_USER={:#x} -> ppn={:?} OK",
                TRAP_CONTEXT_USER, ppn
            ),
            None => crate::println!(
                "[from_elf] ERROR: TRAP_CONTEXT_USER={:#x} NOT MAPPED!",
                TRAP_CONTEXT_USER
            ),
        }
        let tramp_vpn: VirtPageNum = TRAMPOLINE.into();
        match memory_set.translate(tramp_vpn) {
            Some(ppn) => crate::println!(
                "[from_elf] TRAMPOLINE={:#x} -> ppn={:?} OK",
                TRAMPOLINE, ppn
            ),
            None => crate::println!(
                "[from_elf] ERROR: TRAMPOLINE={:#x} NOT MAPPED!",
                TRAMPOLINE
            ),
        }

        (memory_set, user_stack_top, entry_point)
    }

    /// 从现有用户地址空间复制一份新地址空间 (用于 fork)
    pub fn from_existed_user(user_space: &MemorySet) -> MemorySet {
        let mut memory_set = Self::new_bare();
        // map trampoline
        memory_set.map_trampoline_global();
        // copy data sections/trap_context/user_stack
        for area in user_space.areas.iter() {
            let new_area = MapArea::from_another(area);
            memory_set.push(new_area, None);
            // copy data from another space
            for vpn in area.vpn_range.iter() {
                let src_ppn = user_space.translate(vpn).unwrap();
                let dst_ppn = memory_set.translate(vpn).unwrap();
                dst_ppn.get_bytes_array().copy_from_slice(src_ppn.get_bytes_array());
            }
        }
        memory_set
    }
}

// ─── ELF 解析 (内联，避免外部依赖在 no_std 中的问题) ──────
struct ElfFile<'a> {
    data: &'a [u8],
    pub header: Elf64Header,
}

struct Elf64Header {
    pub pt2: Elf64EhdrPt2,
    phoff: u64,
    pub phentsize: u16,
    pub phnum: u16,
}

struct Elf64EhdrPt2 {
    pub e_entry: u64,
}

#[allow(dead_code)]
struct ProgramHeader {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_filesz: u64,
    p_memsz: u64,
}

impl ProgramHeader {
    fn read(data: &[u8], offset: usize) -> Self {
        Self {
            p_type: u32_from_le(&data[offset..offset+4]),
            p_flags: u32_from_le(&data[offset+4..offset+8]),
            p_offset: u64_from_le(&data[offset+8..offset+16]),
            p_vaddr: u64_from_le(&data[offset+16..offset+24]),
            p_filesz: u64_from_le(&data[offset+32..offset+40]),
            p_memsz: u64_from_le(&data[offset+40..offset+48]),
        }
    }
}

impl<'a> ElfFile<'a> {
    fn new(data: &'a [u8]) -> Option<Self> {
        if data.len() < 64 { return None; }
        if &data[0..4] != b"\x7fELF" { return None; }
        // ELF64 little-endian check
        if data[4] != 2 || data[5] != 1 { return None; }

        let e_entry = u64_from_le(&data[24..32]);
        let e_phoff = u64_from_le(&data[32..40]);
        let e_phentsize = u16_from_le(&data[54..56]);
        let e_phnum = u16_from_le(&data[56..58]);

        Some(Self {
            data,
            header: Elf64Header {
                pt2: Elf64EhdrPt2 { e_entry },
                phoff: e_phoff,
                phentsize: e_phentsize,
                phnum: e_phnum,
            },
        })
    }

    fn program_iter(&self) -> ProgramHeaderIter {
        ProgramHeaderIter {
            data: self.data,
            phoff: self.header.phoff as usize,
            phentsize: self.header.phentsize,
            phnum: self.header.phnum,
            current: 0,
        }
    }
}

struct ProgramHeaderIter<'a> {
    data: &'a [u8],
    phoff: usize,
    phentsize: u16,
    phnum: u16,
    current: u16,
}

impl<'a> Iterator for ProgramHeaderIter<'a> {
    type Item = ProgramHeader;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.phnum {
            return None;
        }
        let offset = self.phoff + self.current as usize * self.phentsize as usize;
        self.current += 1;
        Some(ProgramHeader::read(self.data, offset))
    }
}

fn u16_from_le(data: &[u8]) -> u16 {
    u16::from_le_bytes([data[0], data[1]])
}

fn u32_from_le(data: &[u8]) -> u32 {
    u32::from_le_bytes([data[0], data[1], data[2], data[3]])
}

fn u64_from_le(data: &[u8]) -> u64 {
    u64::from_le_bytes([data[0], data[1], data[2], data[3],
                        data[4], data[5], data[6], data[7]])
}
