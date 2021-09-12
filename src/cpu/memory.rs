const MEMORY_MAX: usize = 0x10000;
const RAM_TOP: usize = 0x800;
const MIRROR_TOP: usize = 0x2000;

pub struct Memory {
    ram: [u8; MEMORY_MAX],
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            ram: [0; MEMORY_MAX],
        }
    }

    pub fn write(&mut self, address: usize, value: u8) {
        self.ram[address] = value;
    }

    pub fn read(&self, address: usize) -> u8 {
        if address < MIRROR_TOP {
            return self.ram[address % RAM_TOP];
        }
        self.ram[address]
    }

    pub fn load(&mut self, address: usize, data: &[u8]) {
        self.ram[address..address + data.len()].copy_from_slice(data);
    }
}
