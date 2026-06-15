use crate::config::CLOCK_FREQ;
use riscv::register::time;

const TICKS_PER_SEC: usize = 100;
const MICRO_PER_SEC: usize = 1_000_000;

pub fn get_time() -> usize {
    time::read()
}

pub fn set_next_trigger() {
    let time = get_time();
    let next = time + CLOCK_FREQ / TICKS_PER_SEC;
    crate::sbi::set_timer(next as u64);
}

pub fn get_time_ms() -> usize {
    get_time() / (CLOCK_FREQ / MICRO_PER_SEC)
}
