use super::{read, yield_};

const STDIN: usize = 0;

pub fn getchar() -> u8 {
    let mut c = [0u8; 1];
    loop {
        if read(STDIN, &mut c) > 0 {
            return c[0];
        }
        // 无输入时主动 yield，避免 CPU 空转
        yield_();
    }
}
