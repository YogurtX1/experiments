use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct PhysPageNum(pub usize);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct VirtPageNum(pub usize);

impl From<usize> for PhysAddr {
    fn from(v: usize) -> Self { Self(v) }
}

impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self { Self(v) }
}

impl From<usize> for PhysPageNum {
    fn from(v: usize) -> Self { Self(v) }
}

impl From<usize> for VirtPageNum {
    /// 从 usize 构造 VirPageNum，截取 Sv39 规范的 27 位 VPN
    /// VA[63:39] 的符号扩展由 CPU 自动处理
    fn from(v: usize) -> Self {
        Self((v >> PAGE_SIZE_BITS) & ((1 << 27) - 1))
    }
}

impl From<PhysAddr> for usize {
    fn from(v: PhysAddr) -> Self { v.0 }
}

impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self { v.0 }
}

impl From<PhysPageNum> for usize {
    fn from(v: PhysPageNum) -> Self { v.0 }
}

impl From<VirtPageNum> for usize {
    fn from(v: VirtPageNum) -> Self { v.0 }
}

impl PhysAddr {
    pub fn floor(&self) -> PhysPageNum {
        PhysPageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> PhysPageNum {
        PhysPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }
    /// PhysAddr 的页内偏移
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
}

impl VirtAddr {
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }
    pub fn ceil(&self) -> VirtPageNum {
        VirtPageNum((self.0 + PAGE_SIZE - 1) / PAGE_SIZE)
    }
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

impl From<PhysAddr> for PhysPageNum {
    fn from(v: PhysAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}

impl From<PhysPageNum> for PhysAddr {
    fn from(v: PhysPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        assert_eq!(v.page_offset(), 0);
        v.floor()
    }
}

impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

impl VirtPageNum {
    /// 返回 Sv39 页表遍历索引: [VPN[2], VPN[1], VPN[0]]
    /// - idx[0] (root/level-2 PTE)  = VA[38:30]
    /// - idx[1] (level-1 PTE)        = VA[29:21]
    /// - idx[2] (level-0 PTE/leaf)   = VA[20:12]
    pub fn indexes(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx: [usize; 3] = [0; 3];
        // 从高位到低位填充: idx[2] = VPN[0], idx[1] = VPN[1], idx[0] = VPN[2]
        for i in (0..3).rev() {
            idx[i] = vpn & 0x1ff;
            vpn >>= 9;
        }
        idx
    }
}

impl PhysPageNum {
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: usize = self.0 << PAGE_SIZE_BITS;
        unsafe { core::slice::from_raw_parts_mut(pa as *mut u8, PAGE_SIZE) }
    }

    pub fn get_mut<T>(&self) -> &'static mut T {
        let pa: usize = self.0 << PAGE_SIZE_BITS;
        unsafe { &mut *(pa as *mut T) }
    }
}
