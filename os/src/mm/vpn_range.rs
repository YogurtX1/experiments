use crate::mm::address::VirtPageNum;

#[derive(Clone)]
pub struct VpnRange {
    pub start: VirtPageNum,
    pub end: VirtPageNum,
}

impl VpnRange {
    pub fn new(start: VirtPageNum, end: VirtPageNum) -> Self {
        Self { start, end }
    }

    pub fn get_start(&self) -> VirtPageNum {
        self.start
    }

    pub fn get_end(&self) -> VirtPageNum {
        self.end
    }

    pub fn iter(&self) -> impl Iterator<Item = VirtPageNum> {
        (self.start.0..self.end.0).map(VirtPageNum)
    }
}
