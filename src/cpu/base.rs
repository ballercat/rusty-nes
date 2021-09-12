use super::memory::Memory;

#[derive(Copy, Clone)]
pub struct State {
    pub a: u8,
    pub pc: usize,
    pub x: u8,
    pub y: u8,
    pub status: u8,
}

pub struct Processor {
    pub mem: Memory,
    pub state: State,
    pub cycles: u32,
}
